//! Arena presentation with a shared perspective depth buffer.
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
    scene::{Scene, SpriteFrame, TextureKey},
    texture::Vram,
};

use bevy::camera::visibility::RenderLayers;

#[derive(Component)]
pub struct VehicleMesh(usize);
#[derive(Component)]
pub struct BodyStage(usize);
#[derive(Component)]
pub struct WorldCamera;
#[derive(Component)]
pub struct SkyCamera;

const TILE: usize = 256;
const COLUMNS: usize = 8;

#[derive(Resource)]
pub struct Arena {
    scene: Scene,
    sky: Option<Scene>,
    missile_models: [Option<Scene>; 2],
    contrail_frames: [SpriteFrame; 8],
    explosion_frames: [SpriteFrame; 16],
    vehicle_start: usize,
    body_ranges: Vec<[std::ops::Range<usize>; 3]>,
    overview: bool,
    radar: bool,
    arrows: [SpriteFrame; 4],
    pixels: Vec<u8>,
    atlas_size: [usize; 2],
    tiles: BTreeMap<(Option<TextureKey>, bool), usize>,
    mesh: Handle<Mesh>,
    additive: Handle<Mesh>,
    solids: Handle<Mesh>,
    transparent: Handle<Mesh>,
    center: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
}
impl Arena {
    pub fn new(
        mut scene: Scene,
        sky: Option<Scene>,
        vehicle: [Scene; 3],
        opponents: Vec<[Scene; 3]>,
        missile_models: [Option<Scene>; 2],
        explosion_frames: [SpriteFrame; 16],
        contrail_frames: [SpriteFrame; 8],
        arrows: [SpriteFrame; 4],
        vram: Vram,
    ) -> Result<Self> {
        let vehicle_start = scene.polygons.len();
        let body_ranges = std::iter::once(vehicle)
            .chain(opponents)
            .map(|variants| {
                variants.map(|variant| {
                    let start = scene.polygons.len();
                    scene.polygons.extend(variant.polygons);
                    start..scene.polygons.len()
                })
            })
            .collect();
        let mut tiles = BTreeMap::new();
        tiles.insert((None, false), 0);
        for p in scene
            .polygons
            .iter()
            .chain(sky.iter().flat_map(|s| &s.polygons))
            .chain(missile_models.iter().flatten().flat_map(|s| &s.polygons))
        {
            let next = tiles.len();
            tiles.entry((p.texture, p.semi_transparent)).or_insert(next);
        }
        for frame in explosion_frames
            .iter()
            .chain(&contrail_frames)
            .chain(&arrows)
        {
            let next = tiles.len();
            tiles.entry((Some(frame.texture), true)).or_insert(next);
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
            sky,
            missile_models,
            vehicle_start,
            body_ranges,
            explosion_frames,
            contrail_frames,
            overview: false,
            radar: true,
            arrows,
            pixels,
            atlas_size: [width, height],
            tiles,
            mesh: Handle::default(),
            additive: default(),
            solids: default(),
            transparent: default(),
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
    mut additive_materials: ResMut<Assets<crate::additive::Additive>>,
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
    arena.additive = meshes.add(Batch::default().mesh());
    commands.spawn((
        Mesh2d(arena.additive.clone()),
        MeshMaterial2d(additive_materials.add(crate::additive::Additive {
            atlas: texture.clone(),
        })),
        RenderLayers::layer(1),
        NoFrustumCulling,
    ));
    let masked = materials.add(ColorMaterial {
        texture: Some(texture.clone()),
        alpha_mode: AlphaMode2d::Mask(0.5),
        ..default()
    });
    let blend = materials.add(ColorMaterial {
        texture: Some(texture),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    // The retail sky uses a fixed far ordering-table slot and follows the
    // player horizontally. A background camera gives it independent depth.
    if let Some(sky) = &arena.sky {
        let mut batch = Batch::default();
        for polygon in &sky.polygons {
            let slot = arena.tiles[&(polygon.texture, polygon.semi_transparent)];
            for &i in indices(polygon.vertices.len()) {
                let v = &polygon.vertices[i];
                batch.push(world(v.position), arena.uv(slot, v.uv.into()), v.color);
            }
        }
        commands.spawn((
            Mesh2d(meshes.add(batch.mesh())),
            MeshMaterial2d(masked.clone()),
            RenderLayers::layer(2),
            NoFrustumCulling,
        ));
        commands.spawn((
            WorldCamera,
            SkyCamera,
            Camera2d,
            Camera {
                order: -1,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: 2. * (0.5_f32 / 0.85).atan(),
                near: 1.,
                far: 100000.,
                ..default()
            }),
            RenderLayers::layer(2),
        ));
    }
    for (car, ranges) in arena.body_ranges.iter().enumerate() {
        for (stage, range) in ranges.iter().enumerate() {
            let mut batch = Batch::default();
            for polygon in &arena.scene.polygons[range.clone()] {
                if !polygon.animation.is_empty()
                    || polygon.billboard.is_some()
                    || polygon.semi_transparent
                {
                    continue;
                }
                let slot = arena.tiles[&(polygon.texture, false)];
                for &i in indices(polygon.vertices.len()) {
                    let v = &polygon.vertices[i];
                    batch.push(world(v.position), arena.uv(slot, v.uv.into()), v.color);
                }
            }
            commands.spawn((
                VehicleMesh(car),
                BodyStage(stage),
                if stage == 0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Mesh2d(meshes.add(batch.mesh())),
                MeshMaterial2d(masked.clone()),
                RenderLayers::layer(1),
                NoFrustumCulling,
            ));
        }
    }
    for (transparent, material) in [(false, masked), (true, blend.clone())] {
        let mesh = meshes.add(Batch::default().mesh());
        if transparent {
            arena.transparent = mesh.clone();
        } else {
            arena.solids = mesh.clone();
        }
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            RenderLayers::layer(1),
            NoFrustumCulling,
        ));
    }
    commands.spawn((
        WorldCamera,
        Camera2d,
        Camera {
            order: 0,
            clear_color: if arena.sky.is_some() {
                ClearColorConfig::None
            } else {
                ClearColorConfig::Default
            },

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
    arena.mesh = meshes.add(Batch::default().mesh());
    commands.spawn((
        Mesh2d(arena.mesh.clone()),
        MeshMaterial2d(blend),
        NoFrustumCulling,
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
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
    let state = if drive.player.health().0 <= 0 {
        "WRECKED   R · Restart"
    } else if drive.cleared() {
        "ARENA CLEARED   R · Restart"
    } else if drive.paused {
        "PAUSED   P · Resume"
    } else {
        ""
    };
    let (weapon, ammo) = drive.player.weapon();
    let remaining = if drive.opponents.len() > 1 {
        format!(
            "     Rivals  {}/{}",
            drive
                .opponents
                .iter()
                .filter(|ai| ai.health().0 > 0)
                .count(),
            drive.opponents.len()
        )
    } else {
        String::new()
    };
    let message = format!(
        "{}  {}      {}  {}{remaining}
{}  × {}     {}",
        drive.nearest_opponent().vehicle().name(),
        drive.nearest_opponent().health().0.max(0),
        drive.player.vehicle().name(),
        drive.player.health().0.max(0),
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
    if keys.just_pressed(KeyCode::KeyV) {
        arena.radar = !arena.radar;
    }
    if keys.just_pressed(KeyCode::Tab) {
        arena.overview = !arena.overview;
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
    mut vehicles: Query<
        (&VehicleMesh, &BodyStage, &mut Visibility, &mut Transform),
        Without<WorldCamera>,
    >,
    mut cameras: Query<(&mut Transform, Has<SkyCamera>), With<WorldCamera>>,
    window: Single<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let started = capture.demo.then(std::time::Instant::now);
    let direction = Vec3::new(
        arena.yaw.sin() * arena.pitch.cos(),
        arena.pitch.sin(),
        arena.yaw.cos() * arena.pitch.cos(),
    );
    let origin = world(drive.player.position().map(|v| v as f32));
    let angle = drive.player.heading() as f32 * std::f32::consts::TAU / 4096.;
    let vehicle_rotation = Quat::from_rotation_y(-angle);
    let matrix = drive.player.rotation();
    let poses: Vec<_> = std::iter::once((origin, matrix, drive.player.body_stage()))
        .chain(drive.opponents.iter().map(|ai| {
            (
                world(ai.position().map(|v| v as f32)),
                ai.rotation(),
                ai.body_stage(),
            )
        }))
        .collect();
    let transform = |index: usize, p: [f32; 3]| {
        if index >= arena.vehicle_start {
            let car = arena
                .body_ranges
                .partition_point(|ranges| ranges[2].end <= index);
            let (origin, matrix, _) = poses[car];
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
    for (mut camera, sky) in &mut cameras {
        *camera = Transform::from_translation(eye).looking_at(target, Vec3::Y);
        if sky {
            camera.translation -= Vec3::new(origin.x, 0., origin.z);
        }
    }
    for (vehicle, stage, mut visibility, mut entity) in &mut vehicles {
        let (origin, matrix, selected) = poses[vehicle.0];
        if stage.0 != selected {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Inherited;
        let basis = |p: [f32; 3]| {
            world(std::array::from_fn(|i| {
                (0..3).map(|j| matrix[j][i] as f32 * p[j] / 4096.).sum()
            }))
        };
        entity.translation = origin;
        entity.rotation = Quat::from_mat3(&Mat3::from_cols(
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
    let (_, stand_branches) = drive.contacts.health_stands();
    let mut screen_batch = Batch::default();
    let mut solids = Batch::default();
    let mut transparent = Batch::default();
    let mut additive = Batch::default();
    let visible = (0..arena.vehicle_start).chain(
        arena
            .body_ranges
            .iter()
            .zip(&poses)
            .flat_map(|(ranges, pose)| ranges[pose.2].clone()),
    );
    let hidden_ranges: Vec<_> = arena
        .scene
        .visibility
        .iter()
        .map(|controller| controller.hidden_polygons([eye.x, -eye.z, eye.y]))
        .collect();
    for i in visible {
        if hidden_ranges.iter().any(|range| range.contains(&i)) {
            continue;
        }
        let p = &arena.scene.polygons[i];
        if p.animation
            .iter()
            .any(|frame| frame.selected(drive.player.frame()) != frame.child)
        {
            continue;
        }
        if i >= arena.vehicle_start
            && p.animation.is_empty()
            && p.billboard.is_none()
            && !p.semi_transparent
        {
            continue;
        }
        if p.health_stand
            .is_some_and(|(slot, branch)| (stand_branches >> slot) & 1 != u16::from(branch))
        {
            continue;
        }
        if p.pickup.is_some_and(|id| hidden & (1u64 << id) != 0) {
            continue;
        }
        let slot = arena.tiles[&(p.texture, p.semi_transparent)];
        if i >= arena.vehicle_start && !p.double_sided && p.billboard.is_none() {
            // geomProc rejects negative screen-space NCLIP in the normal view.
            // The screen Y flip makes a clockwise world-space face visible.
            let [a, b, c] = [0, 1, 2].map(|j| transform(i, p.vertices[j].position));
            if (b - a).cross(c - a).dot(eye - a) > 0. {
                continue;
            }
        }
        let indices = indices(p.vertices.len());
        let batch = if p.semi_transparent && p.texture.is_some_and(|k| (k.page >> 5) & 3 == 1) {
            &mut additive
        } else if p.semi_transparent {
            &mut transparent
        } else {
            &mut solids
        };
        for &i_vertex in indices {
            let v = &p.vertices[i_vertex];
            let mut point = transform(i, v.position);
            if let Some([w, h]) = p.billboard {
                point += right * if i_vertex & 1 == 0 { -w } else { w }
                    + up * if i_vertex < 2 { h } else { -h };
            }
            batch.push(point, arena.uv(slot, v.uv.into()), v.color);
        }
    }
    let mut faces = [[0; 14]; 180];
    let count = drive.contacts.contrail_faces(&mut faces);
    for face in &faces[..count] {
        let frame = arena.contrail_frames[face[0] as usize];
        let slot = arena.tiles[&(Some(frame.texture), true)];
        let points: [Vec3; 4] = std::array::from_fn(|i| {
            world([
                face[2 + 3 * i] as f32,
                face[3 + 3 * i] as f32,
                face[4 + 3 * i] as f32,
            ])
        });
        if (points[1] - points[0])
            .cross(points[2] - points[0])
            .dot(eye - points[0])
            > 0.
        {
            continue;
        }
        let tint = [16, 8, 0].map(|shift| ((face[1] >> shift) & 255) as f32 / 128.);
        for &i in indices(4) {
            let [u, v] = frame.uv[i ^ 1];
            additive.push(
                points[i],
                arena.uv(slot, Vec2::new(u.into(), v.into())),
                [tint[0], tint[1], tint[2], 1.],
            );
        }
    }
    let mut poses = [[0; 13]; 10];
    let count = drive.contacts.missile_poses(&mut poses);
    for pose in &poses[..count] {
        let origin = world([pose[1] as f32, pose[2] as f32, pose[3] as f32]);
        let Some(model) = &arena.missile_models[usize::from(pose[0] == 312)] else {
            continue;
        };
        for p in &model.polygons {
            let batch = if p.semi_transparent && p.texture.is_some_and(|t| (t.page >> 5) & 3 == 1) {
                &mut additive
            } else if p.semi_transparent {
                &mut transparent
            } else {
                &mut solids
            };
            let slot = arena.tiles[&(p.texture, p.semi_transparent)];
            for &i in indices(p.vertices.len()) {
                let v = &p.vertices[i];
                let rotated = std::array::from_fn(|k| {
                    (0..3)
                        .map(|j| pose[4 + 3 * j + k] as f32 * v.position[j] / 4096.)
                        .sum()
                });
                batch.push(
                    origin + world(rotated),
                    arena.uv(slot, v.uv.into()),
                    v.color,
                );
            }
        }
    }
    let mut flames = [[0; 4]; 80];
    let count = drive.contacts.flame_particles(&mut flames);
    for &[x, y, z, index] in &flames[..count] {
        let center = world([x as f32, y as f32, z as f32]);
        let frame = arena.explosion_frames[index as usize];
        let radius = f32::from(frame.width) * 6. / 8.;
        let slot = arena.tiles[&(Some(frame.texture), true)];
        for &i in indices(4) {
            let offset = right * if i & 1 == 0 { -radius } else { radius }
                + up * if i < 2 { radius } else { -radius };
            let [u, v] = frame.uv[i];
            additive.push(
                center + offset,
                arena.uv(slot, Vec2::new(u.into(), v.into())),
                [1.; 4],
            );
        }
    }
    let white = arena.uv(0, Vec2::ZERO);
    let mut quad = |points: [Vec2; 4], depths: [f32; 4], colors: [[f32; 4]; 2]| {
        for &i in indices(4) {
            let xy = points[i] * (depths[i] / focal);
            transparent.push(
                eye + forward * depths[i] + right * xy.x + up * xy.y,
                white,
                colors[i / 2],
            );
        }
    };
    let mut segments = [[[0; 3]; 2]; 40];
    let count = drive.contacts.bullet_segments(&mut segments);
    let mut arcs = [[[0; 3]; 16]; 3];
    let arc_count = drive.contacts.taser_arcs(&mut arcs);
    let bullets = segments[..count]
        .iter()
        .map(|&s| (s, [[1., 0.82, 0.22, 1.]; 2]));
    let electricity = arcs[..arc_count].iter().flat_map(|arc| {
        arc.windows(2).enumerate().map(|(j, points)| {
            let colors = [j, j + 1].map(|index| {
                let brightness = (250 - index * 12) as f32 / 255.;
                [brightness, brightness, 1., 1.]
            });
            ([points[0], points[1]], colors)
        })
    });
    for (segment, tint) in bullets.chain(electricity) {
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
            [v[0].z, v[0].z, v[1].z, v[1].z],
            tint,
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
            3 | 26 => continue,
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
            [depth; 4],
            [tint; 2],
        );
    }
    let scale = (window.height() / 800.).clamp(0.6, 1.8);
    let x = -window.width() / 2. + 22. * scale;
    let y = -window.height() / 2. + 24. * scale;
    let mut bar = |y: f32, w: f32, h: f32, tint| {
        let points = [
            Vec2::new(x, y),
            Vec2::new(x + w, y),
            Vec2::new(x, y + h),
            Vec2::new(x + w, y + h),
        ];
        for &i in indices(4) {
            screen_batch.push(points[i].extend(0.), white, tint);
        }
    };
    let (hp, max_hp) = drive.player.health();
    let (op, op_max) = drive.nearest_opponent().health();
    for (row, value, tint) in [
        (
            0,
            drive.player.gun_heat() as f32 / 199.,
            [1., 0.62, 0.1, 1.],
        ),
        (1, hp as f32 / max_hp.max(1) as f32, [0.25, 0.85, 0.42, 1.]),
        (2, op as f32 / op_max.max(1) as f32, [0.95, 0.25, 0.18, 1.]),
    ] {
        let y = y + row as f32 * 12. * scale;
        bar(y, 150. * scale, 8. * scale, [0.05, 0.05, 0.05, 0.85]);
        bar(y, 150. * scale * value.clamp(0., 1.), 8. * scale, tint);
    }
    if arena.radar {
        draw_radar(&mut screen_batch, &arena, &drive, &window);
    }
    screen_batch.write(&mut meshes, &arena.mesh);
    additive.write(&mut meshes, &arena.additive);
    solids.write(&mut meshes, &arena.solids);
    // Sort all translucent surfaces together, including smoke and tracers.
    let mut triangles: Vec<_> = transparent
        .positions
        .chunks_exact(3)
        .enumerate()
        .map(|(i, triangle)| {
            let depth = triangle
                .iter()
                .map(|p| Vec3::from_array(*p).dot(forward))
                .sum::<f32>();
            (i * 3, depth)
        })
        .collect();
    triangles.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut sorted = Batch::default();
    for (first, _) in triangles {
        let range = first..first + 3;
        sorted
            .positions
            .extend_from_slice(&transparent.positions[range.clone()]);
        sorted
            .uvs
            .extend_from_slice(&transparent.uvs[range.clone()]);
        sorted.colors.extend_from_slice(&transparent.colors[range]);
    }
    sorted.write(&mut meshes, &arena.transparent);
    if let Some(start) = started {
        crate::smoke::sample("CPU geometry preparation", &mut timings, start);
    }
}

impl Arena {
    fn uv(&self, slot: usize, uv: Vec2) -> [f32; 2] {
        [
            ((slot % COLUMNS * TILE) as f32 + uv.x + 0.5) / self.atlas_size[0] as f32,
            ((slot / COLUMNS * TILE) as f32 + uv.y + 0.5) / self.atlas_size[1] as f32,
        ]
    }
}
fn draw_radar(batch: &mut Batch, arena: &Arena, drive: &crate::driving::Drive, window: &Window) {
    use tm1_core::drive::RadarSignal;
    let scale = window.height() / 240.;
    let origin = Vec2::new(-window.width() / 2., window.height() / 2.);
    let white = arena.uv(0, Vec2::ZERO);
    let mut rect = |x: i32, y: i32, w: i32, h: i32, uv: [[f32; 2]; 4], color: [f32; 4]| {
        let points = [[x, y], [x + w, y], [x, y + h], [x + w, y + h]];
        for &i in indices(4) {
            let [px, py] = points[i];
            batch.push(
                (origin + scale * Vec2::new(px as f32, -(py as f32))).extend(0.),
                uv[i],
                color,
            );
        }
    };
    rect(16, 32, 73, 45, [white; 4], [0.204, 0.443, 0.067, 0.60]);
    rect(51, 55, 5, 1, [white; 4], [0., 0.188, 0., 1.]);
    rect(53, 54, 1, 3, [white; 4], [0., 0.188, 0., 1.]);
    let mut edges = [false; 4];
    for ai in drive.opponents.iter().filter(|ai| ai.health().0 > 0) {
        match drive.player.radar_signal(ai.position(), ai.vehicle()) {
            RadarSignal::Contact { x, y, rgb } => rect(
                53 + x,
                55 + y,
                2,
                2,
                [white; 4],
                [
                    rgb[0] as f32 / 255.,
                    rgb[1] as f32 / 255.,
                    rgb[2] as f32 / 255.,
                    1.,
                ],
            ),
            RadarSignal::Up => edges[0] = true,
            RadarSignal::Down => edges[1] = true,
            RadarSignal::Right => edges[2] = true,
            RadarSignal::Left => edges[3] = true,
        }
    }
    if drive.player.frame() % 2 == 0 {
        let (active, _) = drive.contacts.health_stands();
        for &(slot, pos) in &arena.scene.health_stands {
            let [x, y] = drive.player.radar_offset(pos);
            if active & (1 << slot) != 0 && (-36..36).contains(&x) && (-22..22).contains(&y) {
                for (dx, dy) in [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)] {
                    rect(
                        53 + x + dx,
                        55 + y + dy,
                        1,
                        1,
                        [white; 4],
                        [1., 0.784, 1., 1.],
                    );
                }
            }
        }
    }
    for (i, (x, y)) in [(47, 26), (47, 77), (89, 49), (10, 49)]
        .into_iter()
        .enumerate()
    {
        if edges[i] {
            let f = arena.arrows[i];
            let [u, v] = f.uv[0];
            let w = i32::from(f.width) + 1;
            let h = i32::from(f.uv[2][1].wrapping_sub(v)) + 1;
            let u = i32::from(u) + i32::from(i != 3);
            let v = i32::from(v);
            let slot = arena.tiles[&(Some(f.texture), true)];
            let uv = [
                [u, v],
                [u + w - 1, v],
                [u, v + h - 1],
                [u + w - 1, v + h - 1],
            ]
            .map(|[u, v]| arena.uv(slot, Vec2::new((u as u8) as f32, (v as u8) as f32)));
            rect(x, y, w, h, uv, [1.; 4]);
        }
    }
}
fn indices(n: usize) -> &'static [usize] {
    if n == 4 {
        &[0, 1, 2, 2, 1, 3]
    } else {
        &[0, 1, 2]
    }
}
#[derive(Default)]
struct Batch {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
}
impl Batch {
    fn push(&mut self, pos: Vec3, uv: [f32; 2], color: [f32; 4]) {
        self.positions.push(pos.to_array());
        self.uvs.push(uv);
        self.colors.push(color);
    }
    fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
    fn write(self, meshes: &mut Assets<Mesh>, handle: &Handle<Mesh>) {
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        }
    }
}
