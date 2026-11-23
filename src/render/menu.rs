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

const DITHER: [[i32; 4]; 4] = [
    [-4, 0, -3, 1],
    [2, -2, 3, -1],
    [-3, 1, -4, 0],
    [3, -1, 2, -2],
];

// Modulate the native five-bit texel, quantize the foreground, then blend in VRAM precision.
fn pixel_channel(
    shade: u8,
    texture: Option<u8>,
    offset: i32,
    background: u8,
    blend: Option<u16>,
) -> u8 {
    let intensity = texture.map_or(i32::from(shade), |t| {
        i32::from(t >> 3) * i32::from(shade) >> 4
    });
    let foreground = ((intensity + offset) >> 3).clamp(0, 31) as u8;
    let bg = background >> 3;
    let value = match blend {
        Some(0) => (bg + foreground) / 2,
        Some(1) => (bg + foreground).min(31),
        Some(2) => bg.saturating_sub(foreground),
        Some(3) => (bg + foreground / 4).min(31),
        _ => foreground,
    };
    (value << 3) | (value >> 2)
}

fn raster_triangle(
    out: &mut [u8],
    textures: &Vram,
    polygon: &Polygon,
    vertices: [PacketCorner; 3],
) {
    raster_packet(
        out,
        textures,
        polygon.texture,
        polygon.semi_transparent,
        polygon.dither,
        vertices,
    );
}

pub(crate) fn draw_portrait(
    out: &mut [u8],
    textures: &Vram,
    portrait: &tm1_assets::scene::OccupantPortrait,
    xy: [[i32; 2]; 4],
) {
    let corners: [PacketCorner; 4] = std::array::from_fn(|i| PacketCorner {
        xy: xy[i],
        uv: portrait.uv[i],
        rgb: [191; 3],
    });
    for tri in [[0, 1, 2], [2, 1, 3]] {
        raster_packet(
            out,
            textures,
            Some(portrait.texture),
            true,
            false,
            tri.map(|i| corners[i]),
        );
    }
}

fn raster_packet(
    out: &mut [u8],
    textures: &Vram,
    texture: Option<TextureKey>,
    semi: bool,
    dither: bool,
    vertices: [PacketCorner; 3],
) {
    let xy = vertices.map(|v| v.xy);
    if !triangle_fits_gpu(xy) {
        return;
    }
    let [p, q, r] = xy.map(|v| IVec2::from_array(v));
    let area = (q - p).perp_dot(r - p);
    if area == 0 {
        return;
    }
    let lo = p.min(q).min(r).max(IVec2::ZERO);
    let hi = p.max(q).max(r).min(IVec2::new(WIDTH as i32, HEIGHT as i32));
    let edges = [(q, r), (r, p), (p, q)];
    let inclusive = edges.map(|(a, b)| {
        let step = (b - a) * area.signum();
        step.y < 0 || (step.y == 0 && step.x > 0)
    });
    // Gradients truncate to twelve fractional bits. Start at the GPU's leftmost
    // vertex with a half-unit bias so RGB and UV round at the same boundary.
    let anchor = if q.x <= p.x {
        if r.x <= q.x { 2 } else { 1 }
    } else if r.x < p.x {
        2
    } else {
        0
    };
    let origin = IVec2::from_array(xy[anchor]);
    let gradients: [_; 5] = std::array::from_fn(|component| {
        let values = vertices.map(|v| {
            if component < 3 {
                v.rgb[component]
            } else {
                v.uv[component - 3]
            }
        });
        let du = values[1] - values[0];
        let dv = values[2] - values[0];
        let slope = |numerator: i32| (i64::from(numerator) << 12) / i64::from(area);
        let dx = slope(du * (r.y - p.y) - dv * (q.y - p.y));
        let dy = slope(dv * (q.x - p.x) - du * (r.x - p.x));
        ((i64::from(values[anchor]) << 12) + 2048, dx, dy)
    });
    for y in lo.y..hi.y {
        for x in lo.x..hi.x {
            let point = IVec2::new(x, y);
            if edges.iter().zip(inclusive).any(|(&(a, b), include)| {
                let distance = (b - a).perp_dot(point - a) * area.signum();
                distance < 0 || (distance == 0 && !include)
            }) {
                continue;
            }
            let delta = point - origin;
            let attributes = gradients.map(|(start, dx, dy)| {
                ((start + dx * i64::from(delta.x) + dy * i64::from(delta.y)) >> 12) as u8
            });
            let mut texel = [255; 4];
            if let Some(key) = texture {
                let [u, v] = [attributes[3], attributes[4]];
                let Ok(sample) = textures.sample_primitive(key, u, v, semi) else {
                    continue;
                };
                if sample[3] == 0 {
                    continue;
                }
                texel = sample;
            }
            let at = 4 * (y as usize * WIDTH + x as usize);
            let blend = (semi && (texture.is_none() || texel[3] == 128))
                .then(|| texture.map_or(0, |t| (t.page >> 5) & 3));
            let offset = if dither {
                DITHER[y as usize & 3][x as usize & 3]
            } else {
                0
            };
            for k in 0..3 {
                let shade = attributes[k];
                out[at + k] =
                    pixel_channel(shade, texture.map(|_| texel[k]), offset, out[at + k], blend);
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
    fn framebuffer_blends_quantized_colors_and_preserves_raw_texels() {
        for level in 0..32u8 {
            let texel = (level << 3) | (level >> 2);
            assert_eq!(pixel_channel(128, Some(texel), 0, 0, None), texel);
        }
        assert_eq!(
            DITHER.map(|r| r.map(|d| pixel_channel(127, None, d, 0, None) >> 3)),
            [
                [15, 15, 15, 16],
                [16, 15, 16, 15],
                [15, 16, 15, 15],
                [16, 15, 16, 15]
            ]
        );
        // Both odd operands retain the carry when averaged; quarter-add truncates before adding.
        for (mode, expected) in [(0, 11), (1, 22), (2, 0), (3, 10)] {
            assert_eq!(pixel_channel(120, None, 0, 56, Some(mode)) >> 3, expected);
        }
        assert_eq!(pixel_channel(255, Some(255), 3, 255, Some(1)), 255);
        assert_eq!(pixel_channel(0, None, -4, 0, None), 0);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn triangle_edges_and_shading_share_integer_pixel_coordinates() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/CARS.DMD").unwrap();
        let textures = Vram::from_tms(&disc.read("UADMD/CARS.TMS").unwrap()).unwrap();
        let mut scene = Dmd::parse(&data)
            .unwrap()
            .garage_vehicle(Vehicle::Outlaw as u16)
            .unwrap();
        let face = &mut scene.polygons[0];
        face.texture = None;
        face.dither = false;
        face.semi_transparent = true;
        let corner = |xy, r| PacketCorner {
            xy,
            uv: [0; 2],
            rgb: [r; 3],
        };
        let a = corner([2, 2], 32);
        let b = corner([7, 2], 192);
        let c = corner([2, 5], 80);
        let d = corner([7, 5], 240);
        let mut out = vec![0; 4 * WIDTH * HEIGHT];
        raster_triangle(&mut out, &textures, face, [a, b, c]);
        for (y, end) in [(2, 7), (3, 6), (4, 4)] {
            for x in 1..8 {
                let opacity = out[4 * (y * WIDTH + x) + 3];
                assert_eq!(opacity == 255, (2..end).contains(&x), "at {x},{y}");
            }
        }
        raster_triangle(&mut out, &textures, face, [c, b, d]);
        for y in 2..5 {
            for x in 2..7 {
                let expected = (32 + 32 * (x - 2) + 16 * (y - 2)) / 16;
                assert_eq!(usize::from(out[4 * (y * WIDTH + x)] >> 3), expected);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn display_model_dither_matches_jp_draw_mode_packets() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut data = disc.read("UADMD/CARS.DMD").unwrap();
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let car = Vehicle::Outlaw as u16;
        let original = Dmd::parse(&data).unwrap().garage_vehicle(car).unwrap();
        let object = original
            .polygons
            .iter()
            .find(|p| {
                let command = data[p.source_offset + 19];
                command & 0x10 != 0 || command & 5 == 4
            })
            .unwrap()
            .object_id;
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let spad = 0x1f800000u32;
        let db = 0x80010000u32;
        let ot = db + 64;
        let packet = db + 128;
        let stub = db + 512;
        cpu.put(db, &ot.to_le_bytes());
        for enabled in 0..=1 {
            cpu.call(0x8015f02c, &[spad + 0x60 + 12 * enabled, 0, enabled, 0, 0]);
        }
        // Enter geomTess with the same record, buffer and scratchpad used by geomProc.
        cpu.put(
            stub,
            &[
                0x3c081f80u32,
                0x00a04821,
                0x00c05021,
                0x00a09821,
                0x08000000 | ((0x800f8014u32 >> 2) & 0x03ffffff),
                0,
            ]
            .map(u32::to_le_bytes)
            .concat(),
        );
        for flag in [0u8, 2] {
            let offset = (object - base) as usize + 26;
            data[offset] = (data[offset] & !2) | flag;
            let scene = Dmd::parse(&data).unwrap().garage_vehicle(car).unwrap();
            cpu.put(base, &data);
            for p in scene.polygons.iter().filter(|p| p.object_id == object) {
                cpu.put(spad + 0x78, &u32::from(flag).to_le_bytes());
                cpu.put(ot, &0xffffffu32.to_le_bytes());
                cpu.call_until(
                    stub,
                    &[db, base + p.source_offset as u32, packet],
                    0x800f8154,
                );
                let head = cpu.word(ot) & 0xffffff;
                let enabled = head != (packet & 0xffffff) && cpu.word(head + 4) & 0x200 != 0;
                if head != (packet & 0xffffff) {
                    assert_eq!(cpu.word(head) & 0xffffff, packet & 0xffffff);
                    let restore = cpu.word(packet) & 0xffffff;
                    assert_eq!(cpu.word(restore + 4) >> 24, 0xe1);
                    assert_eq!(cpu.word(restore + 4) & 0x200, 0);
                    assert_eq!(cpu.word(restore) & 0xffffff, 0xffffff);
                }
                let command = data[p.source_offset + 19];
                let eligible = if command & 4 != 0 {
                    command & 1 == 0
                } else {
                    command & 16 != 0
                };
                assert_eq!(p.dither, enabled && eligible);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn overlapping_showroom_cars_follow_retail_packet_links() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/CARS.DMD").unwrap();
        let db = Dmd::parse(&data).unwrap();
        let textures = Vram::from_tms(&disc.read("UADMD/CARS.TMS").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let trig = tm1_core::math::TrigTables;
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
