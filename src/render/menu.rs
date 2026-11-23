//! CPU composition of the 320 by 240 car-selection and credits display.
use super::*;
use tm1_assets::{
    scene::Polygon,
    screen::{HEIGHT, WIDTH},
};
use tm1_core::{drive::HierarchyTransform, math::Matrix};

pub(crate) struct ModelTransform {
    pub root: HierarchyTransform,
    pub light: LightMatrix,
}

pub(crate) fn model_transform(view: &Matrix, model: &Matrix, relative: [i32; 3]) -> ModelTransform {
    let camera = view.rotation.map(|r| r.map(i32::from));
    let rows = [
        camera[0],
        camera[2].map(|v| -3690 * v >> 12),
        camera[1].map(|v| 3690 * v >> 12),
    ];
    let planes = [
        [-3355, 2349, 0],
        [3355, 2349, 0],
        [0, 4095, 0],
        [0, 1904, -3626],
        [0, 1904, 3626],
        [0, -4095, 0],
    ];
    let normals = planes.map(|plane| {
        std::array::from_fn(|j| (0..3).map(|i| camera[i][j] * plane[i]).sum::<i32>() >> 12)
    });
    ModelTransform {
        root: HierarchyTransform {
            rows,
            normals,
            position: relative,
        }
        .child([0; 3], Some(model.rotation)),
        light: rotate_light(WORLD_LIGHT, model.rotation),
    }
}

struct Face<'a> {
    polygon: &'a Polygon,
    corners: [PacketCorner; 4],
    bucket: u32,
}

#[derive(Default)]
pub(crate) struct ModelTable<'a> {
    faces: Vec<Face<'a>>,
}

impl<'a> ModelTable<'a> {
    pub fn submit(
        &mut self,
        scene: &'a Scene,
        frame: u32,
        pose: ModelTransform,
        environment: LightEnvironment,
        lod_scale: u32,
    ) {
        // Each hierProcOne resets its scratch lists, then links packets into the same display OT.
        let mut pass = hierarchy::Pass::new(pose.root, true);
        pass.lights[0] = pose.light;
        pass.lod_scale = lod_scale;
        for node in &scene.hierarchy {
            if let tm1_assets::scene::HierarchyKind::Animation(a) = node.kind {
                pass.visits.insert(a.node, u64::from(frame));
            }
        }
        let state = hierarchy::BranchState {
            wheels: [0; 14],
            rotation: [0, 4096],
            switches: [0; 8],
            stands: 0,
            pickups: 0,
            damage: &|_| 0,
        };
        pass.traverse((), scene, 0, 0, 0, 1200, true, &state);
        let mut subdivision_count = 0;
        let mut triangles = Vec::new();
        let mut quads = Vec::new();
        for object in &pass.objects {
            let transform = pass.frame(object.slot, object.position);
            for polygon in &scene.polygons[object.polygons.clone()] {
                let n = polygon.vertices.len();
                let scale = if polygon.texture.is_some() {
                    128.
                } else {
                    255.
                };
                let mut depths = [0; 4];
                let corners: [PacketCorner; 4] = std::array::from_fn(|i| {
                    let v = &polygon.vertices[i.min(n - 1)];
                    let (offset, z) = transform.project(v.local, 228, [160, 120]);
                    depths[i] = z;
                    let mut xy = [160 + offset[0], 120 + offset[1]];
                    if let Some(extent) = polygon.billboard {
                        let [w, h] = sprite_half_size(extent, 228, z);
                        xy[0] += if i & 1 == 0 { -w } else { w };
                        xy[1] += if i < 2 { -h } else { h };
                    }
                    PacketCorner {
                        xy,
                        uv: v.uv.map(|v| v as i32),
                        rgb: environment.vertex_rgb(v, scale, pass.lights[object.slot]),
                    }
                });
                if !polygon.double_sided
                    && polygon.billboard.is_none()
                    && !packet_front_facing([corners[0].xy, corners[1].xy, corners[2].xy], false)
                {
                    continue;
                }
                let Some((raw, bucket)) = hierarchy_bucket(n, depths, polygon.fixed_ot) else {
                    continue;
                };
                if queue_subdivision(&mut subdivision_count, polygon.subdivide_near, raw) {
                    if polygon.billboard.is_none() {
                        let queue = if n == 3 { &mut triangles } else { &mut quads };
                        queue.extend(subdivided_faces(corners, n).map(|corners| Face {
                            polygon,
                            corners,
                            bucket,
                        }));
                    }
                } else {
                    self.faces.push(Face {
                        polygon,
                        corners,
                        bucket,
                    });
                }
            }
        }
        self.faces.append(&mut triangles);
        self.faces.append(&mut quads);
    }

    pub fn draw(mut self, out: &mut [u8], textures: &Vram) {
        // DrawOTagR visits descending buckets, newest packet first in every bucket.
        self.faces.sort_by_key(|face| face.bucket);
        for face in self.faces.into_iter().rev() {
            for triangle in indices(face.polygon.vertices.len()).chunks_exact(3) {
                let vertices = [triangle[0], triangle[1], triangle[2]].map(|i| face.corners[i]);
                raster_triangle(out, textures, face.polygon, vertices);
            }
        }
    }
}

fn raster_triangle(
    out: &mut [u8],
    textures: &Vram,
    polygon: &Polygon,
    vertices: [PacketCorner; 3],
) {
    let xy = vertices.map(|v| v.xy);
    if !triangle_fits_gpu(xy) {
        return;
    }
    let [p, q, r] = xy.map(|p| Vec2::new(p[0] as f32, p[1] as f32));
    let area = (q - p).perp_dot(r - p);
    if area == 0. {
        return;
    }
    let lo = p.min(q).min(r).max(Vec2::ZERO).as_ivec2();
    let hi = p
        .max(q)
        .max(r)
        .min(Vec2::new(WIDTH as f32, HEIGHT as f32))
        .as_ivec2();
    // Include the top and left edges only, including the diagonal of a blended quad.
    let inclusive = [(q, r), (r, p), (p, q)].map(|(a, b)| {
        let e = (b - a) * area.signum();
        e.y < 0. || (e.y == 0. && e.x > 0.)
    });
    for y in lo.y..hi.y {
        for x in lo.x..hi.x {
            let point = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let edges =
                [(q, r), (r, p), (p, q)].map(|(a, b)| (b - a).perp_dot(point - a) * area.signum());
            if (0..3).any(|i| edges[i] < 0. || (edges[i] == 0. && !inclusive[i])) {
                continue;
            }
            let weights = edges.map(|e| e / area.abs());
            let interpolate =
                |values: [i32; 3]| (0..3).map(|i| weights[i] * values[i] as f32).sum::<f32>();
            let mut texel = [255; 4];
            if let Some(key) = polygon.texture {
                let [u, v] = [0, 1].map(|k| interpolate(vertices.map(|c| c.uv[k])) as u8);
                let Ok(sample) = textures.sample_primitive(key, u, v, polygon.semi_transparent)
                else {
                    continue;
                };
                if sample[3] == 0 {
                    continue;
                }
                texel = sample;
            }
            let at = 4 * (y as usize * WIDTH + x as usize);
            let blend = polygon.semi_transparent && (polygon.texture.is_none() || texel[3] == 128);
            for k in 0..3 {
                let shade = interpolate(vertices.map(|c| c.rgb[k]));
                let fg = if polygon.texture.is_some() {
                    texel[k] as f32 * shade / 128.
                } else {
                    shade
                } as u8;
                let bg = out[at + k];
                out[at + k] = if blend {
                    match polygon.texture.map_or(0, |t| (t.page >> 5) & 3) {
                        0 => bg / 2 + fg / 2,
                        1 => bg.saturating_add(fg),
                        2 => bg.saturating_sub(fg),
                        _ => bg.saturating_add(fg / 4),
                    }
                } else {
                    fg
                };
            }
            out[at + 3] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tm1_assets::{disc::Disc, scene::Dmd};
    use tm1_core::drive::Vehicle;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn overlapping_showroom_cars_follow_retail_packet_links() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/CARS.DMD").unwrap();
        let db = Dmd::parse(&data).unwrap();
        let textures = Vram::from_tms(&disc.read("UADMD/CARS.TMS").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let trig = tm1_core::math::TrigTables::from_executable(&exe).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        let camera = trig.rotation_yxz([182, 0, 0]);
        let cars =
            [Vehicle::Outlaw, Vehicle::YellowJacket].map(|v| db.garage_vehicle(v as u16).unwrap());
        let mut table = ModelTable::default();
        let mut first_count = 0;
        for (i, car) in cars.iter().enumerate() {
            let rotation = trig.rotation_yxz([0, 0, 1365 + i as i16 * 280]);
            let pose = model_transform(&camera, &rotation, [80 * i as i32, 7680, -3400]);
            table.submit(car, 0, pose, LightEnvironment::GARAGE, 8);
            if i == 0 {
                first_count = table.faces.len();
            }
        }
        assert!(first_count > 10 && table.faces.len() > first_count + 10);
        let ot = 0x80010000;
        let packets = 0x80015000;
        cpu.put(ot, &[255; 4096 * 4]);
        let mut shared_bucket = false;
        for (i, face) in table.faces.iter().enumerate() {
            let head = ot + 4 * face.bucket;
            shared_bucket |= i >= first_count
                && table.faces[..first_count]
                    .iter()
                    .any(|f| f.bucket == face.bucket);
            cpu.call(0x80161a60, &[head, packets + 4 * i as u32]);
        }
        assert!(shared_bucket);
        let mut expected = vec![0; 4 * WIDTH * HEIGHT];
        let mut visited = 0;
        for bucket in (0..4096).rev() {
            let mut addr = cpu.word(ot + bucket * 4) & 0xffffff;
            while addr != 0xffffff {
                let i = ((addr - (packets & 0xffffff)) / 4) as usize;
                let face = &table.faces[i];
                for tri in indices(face.polygon.vertices.len()).chunks_exact(3) {
                    raster_triangle(
                        &mut expected,
                        &textures,
                        face.polygon,
                        [
                            face.corners[tri[0]],
                            face.corners[tri[1]],
                            face.corners[tri[2]],
                        ],
                    );
                }
                addr = cpu.word(addr) & 0xffffff;
                visited += 1;
            }
        }
        assert_eq!(visited, table.faces.len());
        assert!(expected.chunks_exact(4).filter(|p| p[3] == 255).count() > 500);
        let mut rendered = vec![0; expected.len()];
        table.draw(&mut rendered, &textures);
        assert!(rendered == expected);
    }
}
