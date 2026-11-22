//! Prototype primitive presentation: CPU projection and painter ordering, GPU rasterization.
//! This is not yet a replacement for the original geomProc or ordering table.
use anyhow::{Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    sprite_render::AlphaMode2d,
};
use std::collections::BTreeMap;
use tm1_assets::{
    scene::{Scene, TextureKey},
    texture::Vram,
};

use bevy::camera::visibility::RenderLayers;

#[derive(Component)]
pub struct TaxiMesh;
#[derive(Component)]
pub struct ObjectCamera;

const TILE: usize = 256;
const COLUMNS: usize = 8;

#[derive(Resource)]
pub struct Arena {
    scene: Scene,
    floor_start: usize,
    vehicle_start: usize,
    opponent_start: usize,
    overview: bool,
    object_mesh: bool,
    pixels: Vec<u8>,
    atlas_size: [usize; 2],
    tiles: BTreeMap<(Option<TextureKey>, bool), usize>,
    mesh: Handle<Mesh>,
    center: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
}
impl Arena {
    pub fn new(
        mut scene: Scene,
        floor_start: usize,
        vehicle: Scene,
        opponent: Scene,
        vram: Vram,
        object_mesh: bool,
    ) -> Result<Self> {
        let vehicle_start = scene.polygons.len();
        scene.polygons.extend(vehicle.polygons);
        let opponent_start = scene.polygons.len();
        scene.polygons.extend(opponent.polygons);
        let mut tiles = BTreeMap::new();
        tiles.insert((None, false), 0);
        for p in &scene.polygons {
            let next = tiles.len();
            tiles.entry((p.texture, p.semi_transparent)).or_insert(next);
        }
        let width = COLUMNS * TILE;
        let height = tiles.len().div_ceil(COLUMNS) * TILE;
        ensure!(height <= 8192, "texture atlas exceeds prototype limit");
        let mut pixels = vec![0; width * height * 4];
        for ((key, semi), slot) in &tiles {
            for y in 0..TILE {
                for x in 0..TILE {
                    let color = match key {
                        Some(k) => vram.sample_primitive(*k, x as u8, y as u8, *semi)?,
                        None => [255, 255, 255, if *semi { 128 } else { 255 }],
                    };
                    let i = ((slot / COLUMNS * TILE + y) * width + slot % COLUMNS * TILE + x) * 4;
                    pixels[i..i + 4].copy_from_slice(&color);
                }
            }
        }
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for p in &scene.polygons[..vehicle_start] {
            for v in &p.vertices {
                let pos = world(v.position);
                lo = lo.min(pos);
                hi = hi.max(pos);
            }
        }
        let center = (lo + hi) * 0.5;
        let radius = (hi - lo).length() * 0.5;
        ensure!(radius.is_finite() && radius > 0., "invalid arena bounds");
        Ok(Self {
            scene,
            floor_start,
            vehicle_start,
            opponent_start,
            overview: false,
            object_mesh,
            pixels,
            atlas_size: [width, height],
            tiles,
            mesh: Handle::default(),
            center,
            radius,
            yaw: 0.5,
            pitch: 0.65,
            zoom: 1.4,
        })
    }
}
fn world(p: [f32; 3]) -> Vec3 {
    Vec3::new(p[0], p[2], -p[1])
}

pub fn setup(
    mut commands: Commands,
    mut arena: ResMut<Arena>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut image = Image::new(
        Extent3d {
            width: arena.atlas_size[0] as u32,
            height: arena.atlas_size[1] as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        std::mem::take(&mut arena.pixels),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    let texture = images.add(image);
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    for polygon in &arena.scene.polygons[arena.vehicle_start..arena.opponent_start] {
        if polygon.billboard.is_some() {
            continue;
        }
        let slot = arena.tiles[&(polygon.texture, polygon.semi_transparent)];
        let indices: &[usize] = if polygon.vertices.len() == 4 {
            &[0, 1, 2, 2, 1, 3]
        } else {
            &[0, 1, 2]
        };
        for &i in indices {
            let v = &polygon.vertices[i];
            positions.push(world(v.position).to_array());
            uvs.push([
                ((slot % COLUMNS * TILE) as f32 + v.uv[0] + 0.5) / arena.atlas_size[0] as f32,
                ((slot / COLUMNS * TILE) as f32 + v.uv[1] + 0.5) / arena.atlas_size[1] as f32,
            ]);
            colors.push(v.color);
        }
    }
    let local_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    commands.spawn((
        TaxiMesh,
        Mesh2d(meshes.add(local_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(texture.clone()),
            alpha_mode: AlphaMode2d::Mask(0.5),
            ..default()
        })),
        RenderLayers::layer(1),
        NoFrustumCulling,
    ));
    commands.spawn((
        ObjectCamera,
        Camera2d,
        Camera {
            order: 1,
            is_active: arena.object_mesh,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 2. * (0.5_f32 / 0.85).atan(),
            near: arena.radius * 0.005,
            far: arena.radius * 10.,
            ..default()
        }),
        RenderLayers::layer(1),
    ));
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
    arena.mesh = meshes.add(mesh);
    commands.spawn((
        Mesh2d(arena.mesh.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(texture),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        NoFrustumCulling,
    ));
    commands.spawn(Camera2d);
    commands.spawn((
        HudLabel,
        Text2d::new(""),
        TextFont::from_font_size(15.),
        bevy::sprite::Anchor::BOTTOM_LEFT,
        Transform::from_xyz(0., 0., 1.),
    ));
}
#[derive(Component)]
pub struct HudLabel;
pub fn hud(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    mut text: Single<(&mut Text2d, &mut Transform), With<HudLabel>>,
) {
    let (label, transform) = &mut *text;
    let scale = (window.height() / 800.).clamp(0.6, 1.8);
    transform.translation = Vec3::new(
        -window.width() * 0.5 + 180. * scale,
        -window.height() * 0.5 + 21. * scale,
        1.,
    );
    transform.scale = Vec3::splat(scale);
    let state = if drive.taxi.health().0 <= 0 {
        "WRECKED   R · Restart"
    } else if drive.opponent.health().0 <= 0 {
        "ARENA CLEARED   R · Restart"
    } else if drive.paused {
        "PAUSED   P · Resume"
    } else {
        ""
    };
    let (weapon, ammo) = drive.taxi.weapon();
    let message = format!(
        "Police  {}      Taxi  {}
{}  × {}     {}",
        drive.opponent.health().0.max(0),
        drive.taxi.health().0.max(0),
        weapon,
        ammo,
        state
    );
    if label.0 != message {
        label.0 = message;
    }
}

pub fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut arena: ResMut<Arena>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        arena.overview = !arena.overview;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        arena.object_mesh = !arena.object_mesh;
    }
    let dt = time.delta_secs().min(0.1);
    if keys.pressed(KeyCode::ArrowLeft) {
        arena.yaw -= dt;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        arena.yaw += dt;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        arena.pitch = (arena.pitch + dt).min(1.5);
    }
    if keys.pressed(KeyCode::ArrowDown) {
        arena.pitch = (arena.pitch - dt).max(0.05);
    }
    if keys.pressed(KeyCode::Equal) {
        arena.zoom = (arena.zoom - dt).max(0.3);
    }
    if keys.pressed(KeyCode::Minus) {
        arena.zoom = (arena.zoom + dt).min(5.);
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub fn update(
    arena: Res<Arena>,
    drive: Res<crate::driving::Drive>,
    capture: Res<crate::smoke::Capture>,
    mut timings: Local<Vec<f64>>,
    mut taxi_entity: Single<&mut Transform, (With<TaxiMesh>, Without<ObjectCamera>)>,
    mut camera: Single<(&mut Transform, &mut Camera), With<ObjectCamera>>,
    window: Single<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let started = capture.demo.then(std::time::Instant::now);
    let direction = Vec3::new(
        arena.yaw.sin() * arena.pitch.cos(),
        arena.pitch.sin(),
        arena.yaw.cos() * arena.pitch.cos(),
    );
    let origin = world(drive.taxi.position().map(|v| v as f32));
    let angle = drive.taxi.heading() as f32 * std::f32::consts::TAU / 4096.;
    let vehicle_rotation = Quat::from_rotation_y(-angle);
    let matrix = drive.taxi.rotation();
    let opponent_origin = world(drive.opponent.position().map(|v| v as f32));
    let opponent_matrix = drive.opponent.rotation();
    let transform = |index: usize, p: [f32; 3]| {
        if index >= arena.vehicle_start {
            let (origin, matrix) = if index >= arena.opponent_start {
                (opponent_origin, opponent_matrix)
            } else {
                (origin, matrix)
            };
            origin
                + world(std::array::from_fn(|i| {
                    (0..3).map(|j| matrix[j][i] as f32 * p[j] / 4096.).sum()
                }))
        } else {
            world(p)
        }
    };
    let (eye, target) = if arena.overview {
        (
            arena.center + direction * arena.radius * arena.zoom,
            arena.center,
        )
    } else {
        (
            origin + vehicle_rotation * Vec3::new(0., 155., 215.),
            origin + Vec3::Y * 40.,
        )
    };
    camera.1.is_active = arena.object_mesh;
    if arena.object_mesh {
        *camera.0 = Transform::from_translation(eye).looking_at(target, Vec3::Y);
        let basis = |p: [f32; 3]| {
            world(std::array::from_fn(|i| {
                (0..3).map(|j| matrix[j][i] as f32 * p[j] / 4096.).sum()
            }))
        };
        taxi_entity.translation = origin;
        taxi_entity.rotation = Quat::from_mat3(&Mat3::from_cols(
            basis([1., 0., 0.]),
            basis([0., 0., 1.]),
            basis([0., -1., 0.]),
        ))
        .normalize();
    }
    let forward = (target - eye).normalize();
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward);
    let focal = window.height() * 0.85;
    let near = arena.radius * 0.005;
    let hidden = drive.contacts.hidden_pickups();
    let mut order: Vec<_> = arena
        .scene
        .polygons
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            !arena.object_mesh
                || arena.scene.polygons[*i].billboard.is_some()
                || !(arena.vehicle_start..arena.opponent_start).contains(i)
        })
        .filter(|(_, p)| p.pickup.is_none_or(|id| hidden & (1u64 << id) == 0))
        .map(|(i, p)| {
            let depth = p
                .vertices
                .iter()
                .map(|v| (transform(i, v.position) - eye).dot(forward))
                .sum::<f32>()
                / p.vertices.len() as f32;
            (i, depth)
        })
        .collect();
    // DB.c assigns the floor CS the fixed ordering-table slot 0xffe.
    let floor = arena.floor_start..arena.vehicle_start;
    order.sort_by(|a, b| {
        floor
            .contains(&b.0)
            .cmp(&floor.contains(&a.0))
            .then_with(|| b.1.total_cmp(&a.1))
    });
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    for (i, _) in order {
        let p = &arena.scene.polygons[i];
        let slot = arena.tiles[&(p.texture, p.semi_transparent)];
        let indices: &[usize] = if p.vertices.len() == 4 {
            &[0, 1, 2, 2, 1, 3]
        } else {
            &[0, 1, 2]
        };
        for triangle in indices.chunks_exact(3) {
            let vertex = |index: usize| {
                let v = &p.vertices[index];
                let delta = transform(i, v.position) - eye;
                let mut view = Vec3::new(delta.dot(right), delta.dot(up), delta.dot(forward));
                if let Some([w, h]) = p.billboard {
                    view.x += if index & 1 == 0 { -w } else { w };
                    view.y += if index < 2 { h } else { -h };
                }
                ClipVertex {
                    view,
                    uv: Vec2::from(v.uv),
                    color: Vec4::from(v.color),
                }
            };
            let corners = [
                vertex(triangle[0]),
                vertex(triangle[1]),
                vertex(triangle[2]),
            ];
            let mut clipped = [corners[0]; 4];
            let mut count = 0;
            let mut previous = corners[2];
            for current in corners {
                if (current.view.z >= near) != (previous.view.z >= near) {
                    let t = (near - previous.view.z) / (current.view.z - previous.view.z);
                    clipped[count] = previous.lerp(current, t);
                    count += 1;
                }
                if current.view.z >= near {
                    clipped[count] = current;
                    count += 1;
                }
                previous = current;
            }
            for j in 2..count {
                for v in [clipped[0], clipped[j - 1], clipped[j]] {
                    let projected = v.view.truncate() * (focal / v.view.z);
                    positions.push([projected.x, projected.y, 0.]);
                    uvs.push([
                        ((slot % COLUMNS * TILE) as f32 + v.uv.x + 0.5)
                            / arena.atlas_size[0] as f32,
                        ((slot / COLUMNS * TILE) as f32 + v.uv.y + 0.5)
                            / arena.atlas_size[1] as f32,
                    ]);
                    colors.push(v.color.to_array());
                }
            }
        }
    }
    // The untextured atlas tile is solid white. Tracers and HUD bars share
    // the arena batch and are independent of simulation stepping.
    let mut quad = |points: [Vec2; 4], tint: [f32; 4]| {
        for i in [0, 1, 2, 2, 1, 3] {
            positions.push([points[i].x, points[i].y, 0.]);
            uvs.push([
                0.5 / arena.atlas_size[0] as f32,
                0.5 / arena.atlas_size[1] as f32,
            ]);
            colors.push(tint);
        }
    };
    let mut segments = [[[0; 3]; 2]; 40];
    let count = drive.contacts.bullet_segments(&mut segments);
    for segment in &segments[..count] {
        let mut v = segment.map(|p| {
            let d = world(p.map(|n| n as f32)) - eye;
            Vec3::new(d.dot(right), d.dot(up), d.dot(forward))
        });
        if v[0].z < near && v[1].z < near {
            continue;
        }
        if (v[0].z < near) != (v[1].z < near) {
            let t = (near - v[0].z) / (v[1].z - v[0].z);
            let cross = v[0].lerp(v[1], t);
            let i = usize::from(v[1].z < near);
            v[i] = cross;
        }
        let [a, b] = v.map(|p| p.truncate() * (focal / p.z));
        let normal = (b - a).normalize_or_zero().perp();
        quad(
            [a - normal, a + normal, b - normal, b + normal],
            [1., 0.82, 0.22, 1.],
        );
    }
    let mut effects = [[0; 5]; 122];
    let count = drive.contacts.effects(&mut effects);
    for &[x, y, z, kind, ttl] in &effects[..count] {
        let delta = world([x as f32, y as f32, z as f32]) - eye;
        let depth = delta.dot(forward);
        if depth < near {
            continue;
        }
        let center = Vec2::new(delta.dot(right), delta.dot(up)) * (focal / depth);
        let (size, tint) = match kind {
            2 => (12. + (7 - ttl) as f32 * 6., [1., 0.24, 0.04, 0.82]),
            7 => (3., [0.7, 0.5, 0.3, 1.]),
            8 => (8., [0.4, 0.39, 0.38, 0.55]),
            5 => (3., [0.35, 0.65, 1., 1.]),
            6 => (16., [0.2, 0.16, 0.14, 1.]),
            3 => (6., [1., 0.4, 0.08, 1.]),
            1 => (5., [1., 0.87, 0.39, 1.]),
            _ => (7., [1., 0.78, 0.21, 0.96]),
        };
        let radius = (size * focal / depth).clamp(1., 80.);
        quad(
            [
                center - Vec2::X * radius,
                center + Vec2::Y * radius,
                center - Vec2::Y * radius,
                center + Vec2::X * radius,
            ],
            tint,
        );
    }
    let scale = (window.height() / 800.).clamp(0.6, 1.8);
    let x = -window.width() / 2. + 22. * scale;
    let y = -window.height() / 2. + 24. * scale;
    let mut bar = |y: f32, w: f32, h: f32, tint| {
        quad(
            [
                Vec2::new(x, y),
                Vec2::new(x + w, y),
                Vec2::new(x, y + h),
                Vec2::new(x + w, y + h),
            ],
            tint,
        );
    };
    let (hp, max_hp) = drive.taxi.health();
    let (op, op_max) = drive.opponent.health();
    for (row, value, tint) in [
        (0, drive.taxi.gun_heat() as f32 / 199., [1., 0.62, 0.1, 1.]),
        (1, hp as f32 / max_hp.max(1) as f32, [0.25, 0.85, 0.42, 1.]),
        (2, op as f32 / op_max.max(1) as f32, [0.95, 0.25, 0.18, 1.]),
    ] {
        let y = y + row as f32 * 12. * scale;
        bar(y, 150. * scale, 8. * scale, [0.05, 0.05, 0.05, 0.85]);
        bar(y, 150. * scale * value.clamp(0., 1.), 8. * scale, tint);
    }
    if let Some(mut mesh) = meshes.get_mut(&arena.mesh) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    if let Some(start) = started {
        crate::smoke::sample("CPU geometry preparation", &mut timings, start);
    }
}

#[derive(Clone, Copy)]
struct ClipVertex {
    view: Vec3,
    uv: Vec2,
    color: Vec4,
}
impl ClipVertex {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            view: self.view.lerp(other.view, t),
            uv: self.uv.lerp(other.uv, t),
            color: self.color.lerp(other.color, t),
        }
    }
}
