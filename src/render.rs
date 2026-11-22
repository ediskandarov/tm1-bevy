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

#[derive(Component)]
pub struct MirrorCamera;
#[derive(Component)]
pub struct MirrorDisplay;
const MIRROR_WIDTH: u32 = 101;
const MIRROR_HEIGHT: u32 = 30;
const MIRROR_FOCAL: f32 = 91.;

#[derive(Default)]
struct ViewMeshes {
    additive: Handle<Mesh>,
    solids: Handle<Mesh>,
    transparent: Handle<Mesh>,
}
const TILE: usize = 256;
const MAX_ATLAS_SIDE: usize = 8192;

#[derive(Resource)]
pub struct Arena {
    pub(crate) fonts: tm1_assets::screen::Fonts,
    pub(crate) pause_art: crate::pause::Art,
    cockpit: tm1_assets::screen::Cockpit,
    scene: Scene,
    sky: Option<Scene>,
    missile_models: [Option<Scene>; 2],
    merc_models: [Vec<Scene>; 2],
    pedestrians: Vec<Vec<Scene>>,
    contrail_frames: [SpriteFrame; 8],
    explosion_frames: [SpriteFrame; 16],
    spark_frames: [SpriteFrame; 4],
    flame_frames: [SpriteFrame; 10],
    burn_frames: [SpriteFrame; 8],
    smoke_frames: [[SpriteFrame; 16]; 2],
    vehicle_start: usize,
    body_ranges: Vec<[std::ops::Range<usize>; 3]>,
    overview: bool,
    radar: bool,
    list_cars: bool,
    icons: tm1_assets::scene::DashboardIcons,
    arrows: [SpriteFrame; 4],
    pixels: Vec<u8>,
    atlas_size: [usize; 2],
    tiles: BTreeMap<(Option<TextureKey>, bool), usize>,
    mesh: Handle<Mesh>,
    main_view: ViewMeshes,
    rear_view: ViewMeshes,
    center: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    zoom: f32,
}
impl Arena {
    pub fn new(
        fonts: tm1_assets::screen::Fonts,
        pause_art: crate::pause::Art,
        cockpit: tm1_assets::screen::Cockpit,
        mut scene: Scene,
        sky: Option<Scene>,
        vehicle: [Scene; 3],
        opponents: Vec<[Scene; 3]>,
        missile_models: [Option<Scene>; 2],
        merc_models: [Vec<Scene>; 2],
        pedestrians: Vec<Vec<Scene>>,
        explosion_frames: [SpriteFrame; 16],
        spark_frames: [SpriteFrame; 4],
        flame_frames: [SpriteFrame; 10],
        burn_frames: [SpriteFrame; 8],
        smoke_frames: [[SpriteFrame; 16]; 2],
        contrail_frames: [SpriteFrame; 8],
        icons: tm1_assets::scene::DashboardIcons,
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
            .chain(
                merc_models
                    .iter()
                    .chain(&pedestrians)
                    .flatten()
                    .flat_map(|s| &s.polygons),
            )
        {
            let next = tiles.len();
            tiles.entry((p.texture, p.semi_transparent)).or_insert(next);
        }
        for (frame, semi) in explosion_frames
            .iter()
            .chain(&spark_frames)
            .chain(&flame_frames)
            .chain(&burn_frames)
            .chain(smoke_frames.iter().flatten())
            .chain(&contrail_frames)
            .chain(&arrows)
            .map(|f| (f, true))
            .chain(icons.frames())
        {
            let next = tiles.len();
            tiles.entry((Some(frame.texture), semi)).or_insert(next);
        }
        let columns = tiles.len().isqrt().max(1);
        let width = columns * TILE;
        let height = tiles.len().div_ceil(columns) * TILE;
        ensure!(
            width.max(height) <= MAX_ATLAS_SIDE,
            "texture atlas exceeds supported dimensions"
        );
        let mut pixels = vec![0; width * height * 4];
        for ((key, semi), slot) in &tiles {
            for y in 0..TILE {
                for x in 0..TILE {
                    let color = match key {
                        Some(k) => vram.sample_primitive(*k, x as u8, y as u8, *semi)?,
                        None => [255, 255, 255, if *semi { 128 } else { 255 }],
                    };
                    let i = ((slot / columns * TILE + y) * width + slot % columns * TILE + x) * 4;
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
            fonts,
            pause_art,
            cockpit,
            scene,
            sky,
            missile_models,
            merc_models,
            pedestrians,
            vehicle_start,
            body_ranges,
            explosion_frames,
            spark_frames,
            flame_frames,
            burn_frames,
            smoke_frames,
            contrail_frames,
            overview: false,
            radar: true,
            list_cars: false,
            icons,
            arrows,
            pixels,
            atlas_size: [width, height],
            tiles,
            mesh: Handle::default(),
            main_view: default(),
            rear_view: default(),
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
    let mut cockpit_image = Image::new(
        Extent3d {
            width: 320,
            height: 240,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        arena.cockpit.frame(0),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    cockpit_image.sampler = ImageSampler::nearest();
    let mut pause_image = cockpit_image.clone();
    pause_image.data = Some(vec![0; 320 * 240 * 4]);
    commands.spawn((
        crate::campaign::StageEntity,
        crate::pause::Overlay::default(),
        Sprite::from_image(images.add(pause_image)),
        Visibility::Hidden,
        Transform::from_xyz(0., 0., 8.),
    ));
    for notice in [BattleNotice::Boss, BattleNotice::Lost] {
        let mut image = cockpit_image.clone();
        let mut pixels = [0, 0, 0, 128].repeat(320 * 240);
        match notice {
            BattleNotice::Boss => boss_text(&arena.fonts, &mut pixels),
            BattleNotice::Lost => arena.fonts.centered(
                &mut pixels,
                tm1_assets::screen::Face::Text,
                119,
                "YOU LOSE!",
            ),
        }
        image.data = Some(pixels);
        commands.spawn((
            crate::campaign::StageEntity,
            notice,
            Sprite::from_image(images.add(image)),
            Visibility::Hidden,
            Transform::from_xyz(0., 0., 4.),
        ));
    }
    let mut lettering = cockpit_image.clone();
    lettering.data = Some(vec![0; 320 * 240 * 4]);
    commands.spawn((
        crate::campaign::StageEntity,
        HudLetters::default(),
        Sprite::from_image(images.add(lettering)),
        Transform::from_xyz(0., 0., 0.5),
    ));

    commands.spawn((
        crate::campaign::StageEntity,
        CockpitOverlay {
            degrees: 0,
            frame: None,
        },
        Sprite::from_image(images.add(cockpit_image)),
        Visibility::Hidden,
        Transform::from_xyz(0., 0., -1.),
    ));
    let texture = images.add(image);
    let additive_material = additive_materials.add(crate::additive::Additive {
        atlas: texture.clone(),
    });
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
            crate::campaign::StageEntity,
            Mesh2d(meshes.add(batch.mesh())),
            MeshMaterial2d(masked.clone()),
            RenderLayers::layer(2),
            NoFrustumCulling,
        ));
        commands.spawn((
            crate::campaign::StageEntity,
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
                crate::campaign::StageEntity,
                VehicleMesh(car),
                BodyStage(stage),
                if stage == 0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Mesh2d(meshes.add(batch.mesh())),
                MeshMaterial2d(masked.clone()),
                RenderLayers::from_layers(&[1, 3]),
                NoFrustumCulling,
            ));
        }
    }
    let arena_views = &mut *arena;
    for (layer, view) in [
        (1, &mut arena_views.main_view),
        (3, &mut arena_views.rear_view),
    ] {
        view.additive = meshes.add(Batch::default().mesh());
        commands.spawn((
            crate::campaign::StageEntity,
            Mesh2d(view.additive.clone()),
            MeshMaterial2d(additive_material.clone()),
            RenderLayers::layer(layer),
            NoFrustumCulling,
        ));
        for (handle, material) in [
            (&mut view.solids, masked.clone()),
            (&mut view.transparent, blend.clone()),
        ] {
            *handle = meshes.add(Batch::default().mesh());
            commands.spawn((
                crate::campaign::StageEntity,
                Mesh2d(handle.clone()),
                MeshMaterial2d(material),
                RenderLayers::layer(layer),
                NoFrustumCulling,
            ));
        }
    }
    let mut mirror_image = Image::new_target_texture(
        MIRROR_WIDTH,
        MIRROR_HEIGHT,
        TextureFormat::Bgra8UnormSrgb,
        None,
    );
    mirror_image.sampler = ImageSampler::nearest();
    let mirror_target = images.add(mirror_image);
    for sky in [false, true] {
        if sky && arena.sky.is_none() {
            continue;
        }
        let mirror = commands
            .spawn((
                crate::campaign::StageEntity,
                WorldCamera,
                MirrorCamera,
                Camera2d,
                Camera {
                    order: if sky { -3 } else { -2 },
                    is_active: false,
                    clear_color: if !sky && arena.sky.is_some() {
                        ClearColorConfig::None
                    } else {
                        ClearColorConfig::Default
                    },
                    ..default()
                },
                bevy::camera::RenderTarget::Image(mirror_target.clone().into()),
                Projection::Perspective(PerspectiveProjection {
                    fov: 2. * (MIRROR_HEIGHT as f32 / (2. * MIRROR_FOCAL)).atan(),
                    near: if sky { 1. } else { arena.radius * 0.005 },
                    far: if sky { 100000. } else { arena.radius * 10. },
                    ..default()
                }),
                RenderLayers::layer(if sky { 2 } else { 3 }),
            ))
            .id();
        if sky {
            commands.entity(mirror).insert(SkyCamera);
        }
    }
    commands.spawn((
        crate::campaign::StageEntity,
        MirrorDisplay,
        Sprite {
            flip_x: true,
            ..Sprite::from_image(mirror_target)
        },
        Visibility::Hidden,
        Transform::from_xyz(0., 0., 0.75),
    ));
    commands.spawn((
        crate::campaign::StageEntity,
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
        crate::campaign::StageEntity,
        Mesh2d(arena.mesh.clone()),
        MeshMaterial2d(blend),
        NoFrustumCulling,
    ));
    commands.spawn((
        crate::campaign::StageEntity,
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
    commands.spawn((
        crate::campaign::StageEntity,
        HudLabel,
        Text2d::new(""),
        TextFont::from_font_size(15.),
        bevy::sprite::Anchor::BOTTOM_LEFT,
        Transform::from_xyz(0., 0., 1.),
    ));
    commands.spawn((
        crate::campaign::StageEntity,
        LostLifeTint,
        Visibility::Hidden,
        Sprite::from_color(Color::BLACK, Vec2::ONE),
        Transform::from_xyz(0., 0., 3.),
    ));
}
#[derive(Component)]
pub struct LostLifeTint;
#[derive(Component)]
pub enum BattleNotice {
    Boss,
    Lost,
}
pub fn boss_text(fonts: &tm1_assets::screen::Fonts, pixels: &mut [u8]) {
    for (y, text) in [
        (70, "CONGRATULATIONS!"),
        (100, "You have made it to the"),
        (130, "FINAL round.  Prepare to"),
        (160, "battle MINION, last year's"),
        (190, "Twisted Metal winner!"),
    ] {
        fonts.centered(pixels, tm1_assets::screen::Face::Text, y, text);
    }
}
pub fn battle_notice(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    mut backdrop: Single<(&mut Visibility, &mut Sprite), With<LostLifeTint>>,
    mut notices: Query<(&BattleNotice, &mut Visibility, &mut Sprite), Without<LostLifeTint>>,
) {
    let size = Some(Vec2::new(window.width(), window.height()));
    *backdrop.0 = Visibility::Hidden;
    if let Some(elapsed) = drive.lost_life_elapsed() {
        *backdrop.0 = Visibility::Visible;
        let field = (elapsed.as_secs_f64() * 60.).floor() as u32 + 1;
        let red = 255 - (30 - field.min(30)) * 255 / 30;
        backdrop.1.color = Color::srgba_u8(red as u8, 0, 0, 128);
        backdrop.1.custom_size = size;
    }
    for (notice, mut visibility, mut sprite) in &mut notices {
        let show = match notice {
            BattleNotice::Lost => drive.lives == 0,
            BattleNotice::Boss => drive.showing_boss_intro(),
        };
        *visibility = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        sprite.custom_size = size;
    }
}
#[derive(Component)]
pub struct HudLabel;
#[derive(Component)]
pub struct CockpitOverlay {
    degrees: i32,
    frame: Option<u32>,
}

pub fn cockpit(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    arena: Res<Arena>,
    mut images: ResMut<Assets<Image>>,
    mut overlay: Single<(&mut CockpitOverlay, &mut Sprite, &mut Visibility)>,
) {
    let (state, sprite, visibility) = &mut *overlay;
    let showing = drive.camera.mode == tm1_core::drive::CarView::Cockpit
        && !drive.camera.show_body
        && drive.player.dashboard_enabled()
        && !arena.overview
        && drive.helicopter_pose().is_none();
    **visibility = if showing {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if !showing {
        return;
    }
    sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
    if state.frame == Some(drive.player.frame()) {
        return;
    }
    state.frame = Some(drive.player.frame());
    let angle = drive.player.cockpit_steering_angle(state.degrees);
    if angle != state.degrees {
        state.degrees = angle;
        if let Some(mut image) = images.get_mut(&sprite.image) {
            image.data = Some(arena.cockpit.frame(angle));
        }
    }
}
#[derive(Component, Default)]
pub struct HudLetters {
    last: Option<(i32, Option<tm1_core::drive::Vehicle>, u16, bool)>,
}
pub fn hud_letters(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    arena: Res<Arena>,
    mut images: ResMut<Assets<Image>>,
    mut overlay: Single<(&mut HudLetters, &mut Sprite)>,
) {
    use tm1_assets::screen::Face;
    use tm1_core::drive::Vehicle;
    let (state, sprite) = &mut *overlay;
    sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
    let speed = drive.player.speed_mph(3).abs() % 1000;
    let sign = drive
        .player
        .hud_opponent(&drive.opponents)
        .filter(|ai| ai.health().0 > 0)
        .map(|ai| ai.vehicle());
    let mask = if arena.list_cars {
        drive
            .opponents
            .iter()
            .enumerate()
            .filter(|(_, ai)| ai.health().0 > 0)
            .fold(0u16, |mask, (i, _)| mask | (1 << i))
    } else {
        0
    };
    let at_capacity = drive.contacts.carry_notice_frames() > 0;
    if state.last == Some((speed, sign, mask, at_capacity)) {
        return;
    }
    state.last = Some((speed, sign, mask, at_capacity));
    if let Some(mut image) = images.get_mut(&sprite.image)
        && let Some(pixels) = &mut image.data
    {
        pixels.fill(0);
        arena
            .fonts
            .at(pixels, Face::Speed, 102, 214, &format!("{speed:3}"));
        if at_capacity {
            arena
                .fonts
                .centered_at(pixels, Face::Control, 159, 155, "Max Carrying Capacity");
        }
        let name = |car: Vehicle| {
            let card = Vehicle::CARD_ORDER
                .iter()
                .position(|&v| v == car)
                .unwrap_or(12);
            arena.fonts.car_sign(card)
        };
        if let Some(car) = sign {
            arena.fonts.centered(pixels, Face::Sign, 33, name(car));
        }
        for (i, ai) in drive.opponents.iter().enumerate() {
            if mask & (1 << i) != 0 {
                arena.fonts.at(
                    pixels,
                    Face::Sign,
                    220,
                    70 + i as i32 * 12,
                    name(ai.vehicle()),
                );
            }
        }
    }
}

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
    let state =
        if drive.lives == 0 || drive.lost_life_elapsed().is_some() || drive.showing_boss_intro() {
            ""
        } else if drive.player.health().0 <= 0 {
            "WRECKED   R · Restart"
        } else if drive.cleared() {
            "ARENA CLEARED   R · Restart"
        } else {
            ""
        };
    let message = state.to_owned();
    if label.0 != message {
        label.0 = message;
    }
}

pub fn controls(
    drive: Res<crate::driving::Drive>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut arena: ResMut<Arena>,
    mut exit: MessageWriter<AppExit>,
) {
    if drive.pause.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        arena.list_cars = !arena.list_cars;
    }
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
    mut cameras: Query<
        (
            &mut Transform,
            &mut Camera,
            Has<SkyCamera>,
            Has<MirrorCamera>,
        ),
        With<WorldCamera>,
    >,
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
    let hide_player =
        !arena.overview && drive.helicopter_pose().is_none() && !drive.camera.show_body;
    let rotate = |local: Vec3| {
        if drive.camera.mode == tm1_core::drive::CarView::Cockpit {
            let p = [local.x, -local.z, local.y];
            world(std::array::from_fn(|i| {
                (0..3).map(|j| matrix[j][i] as f32 * p[j] / 4096.).sum()
            }))
        } else {
            vehicle_rotation * local
        }
    };
    let (eye, target, camera_up) = if arena.overview {
        (
            arena.center + direction * arena.radius * arena.zoom,
            arena.center,
            Vec3::Y,
        )
    } else if let Some([x, y, z, pitch, yaw]) = drive.helicopter_pose().or(drive.camera.fixed_pose)
    {
        let eye = world([x, y, z].map(|v| v as f32));
        let radians = std::f32::consts::TAU / 4096.;
        let (sp, cp) = (pitch as f32 * radians).sin_cos();
        let (sy, cy) = (yaw as f32 * radians).sin_cos();
        (
            eye,
            eye + Vec3::new(sy * cp, -sp, -cy * cp) * 100.,
            Vec3::new(sy * sp, cp, -cy * sp),
        )
    } else {
        let eye = origin + rotate(world(drive.camera.offset.map(|v| v as f32)));
        let pitch = drive.camera.pitch as f32 * std::f32::consts::TAU / 4096.;
        let (sin, cos) = pitch.sin_cos();
        (
            eye,
            eye + rotate(Vec3::new(0., -sin, -cos)) * 100.,
            rotate(Vec3::new(0., cos, -sin)),
        )
    };
    // VIEW.c viewProc(1) copies the parent pose and local translation, then
    // reverses local yaw and subtracts 56 PS1 angle units from local pitch.
    let angle = (drive.camera.pitch - 56) as f32 * std::f32::consts::TAU / 4096.;
    let (sin, cos) = angle.sin_cos();
    let (rear_forward, rear_up) = if drive.camera.fixed_pose.is_some() {
        let forward = (target - eye).normalize();
        (
            -forward * cos - camera_up * sin,
            camera_up * cos - forward * sin,
        )
    } else {
        (
            rotate(Vec3::new(0., -sin, cos)).normalize(),
            rotate(Vec3::new(0., cos, sin)).normalize(),
        )
    };
    let mirror = drive.mirror && !arena.overview;
    for (mut camera, mut settings, sky, rear) in &mut cameras {
        if rear {
            settings.is_active = mirror;
            if !mirror {
                continue;
            }
            *camera = Transform::from_translation(eye).looking_to(rear_forward, rear_up);
            camera.scale.x = 3690. / 4096.;
        } else {
            *camera = Transform::from_translation(eye).looking_at(target, camera_up);
        }
        if sky {
            camera.translation -= Vec3::new(origin.x, 0., origin.z);
        }
    }
    for (vehicle, stage, mut visibility, mut entity) in &mut vehicles {
        let Some(&(origin, matrix, selected)) = poses.get(vehicle.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if stage.0 != selected || (vehicle.0 == 0 && hide_player) {
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
    let mut screen_batch = Batch::default();
    draw_instruments(&mut screen_batch, &arena, &drive, &window);
    if arena.radar {
        draw_radar(&mut screen_batch, &arena, &drive, &window);
    }
    screen_batch.write(&mut meshes, &arena.mesh);
    draw_view(
        &arena,
        &drive,
        &mut meshes,
        &arena.main_view,
        &poses,
        hide_player,
        eye,
        (target - eye).normalize(),
        camera_up,
        window.height() * 0.85,
    );
    if mirror {
        draw_view(
            &arena,
            &drive,
            &mut meshes,
            &arena.rear_view,
            &poses,
            hide_player,
            eye,
            rear_forward,
            rear_up,
            MIRROR_FOCAL,
        );
    }
    if let Some(start) = started {
        crate::smoke::sample("CPU geometry preparation", &mut timings, start);
    }
}

pub fn mirror_display(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    arena: Res<Arena>,
    mut display: Single<(&mut Sprite, &mut Transform, &mut Visibility), With<MirrorDisplay>>,
) {
    let (sprite, pos, visibility) = &mut *display;
    **visibility = if drive.mirror && !arena.overview {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let scale = Vec2::new(window.width() / 320., window.height() / 240.);
    sprite.custom_size = Some(scale * Vec2::new(MIRROR_WIDTH as f32, MIRROR_HEIGHT as f32));
    pos.translation = (scale
        * Vec2::new(
            211. + MIRROR_WIDTH as f32 / 2. - 160.,
            120. - 20. - MIRROR_HEIGHT as f32 / 2.,
        ))
    .extend(0.75);
}

fn draw_view(
    arena: &Arena,
    drive: &crate::driving::Drive,
    meshes: &mut Assets<Mesh>,
    view: &ViewMeshes,
    poses: &[(Vec3, [[i16; 3]; 3], usize)],
    hide_player: bool,
    eye: Vec3,
    forward: Vec3,
    camera_up: Vec3,
    focal: f32,
) {
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
    let right = forward.cross(camera_up).normalize();
    let up = right.cross(forward);
    let near = arena.radius * 0.005;
    let hidden = drive.contacts.hidden_pickups();
    let switches = drive.contacts.arena_switch_branches();
    let (_, stand_branches) = drive.contacts.health_stands();
    let mut solids = Batch::default();
    let mut transparent = Batch::default();
    let mut additive = Batch::default();
    let visible = (0..arena.vehicle_start).chain(
        arena
            .body_ranges
            .iter()
            .zip(poses)
            .skip(usize::from(hide_player))
            .flat_map(|(ranges, pose)| ranges[pose.2].clone()),
    );
    let mut hidden_world = vec![false; arena.vehicle_start];
    for controller in &arena.scene.visibility {
        hidden_world[controller.hidden_polygons([eye.x, -eye.z, eye.y])].fill(true);
    }
    for (address, branch, polygons) in &arena.scene.damage_branches {
        if drive.contacts.damage_branch(*address) != *branch {
            hidden_world[polygons.clone()].fill(true);
        }
    }
    let billboard_point = |center: Vec3, extent: Option<[f32; 2]>, corner: usize| {
        if let Some([w, h]) = extent {
            center
                + right * if corner & 1 == 0 { -w } else { w }
                + up * if corner < 2 { h } else { -h }
        } else {
            center
        }
    };
    for i in visible {
        if i < arena.vehicle_start && hidden_world[i] {
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
        if p.arena_switches
            .iter()
            .any(|&(slot, branch)| switches[usize::from(slot)] != branch)
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
            let point = billboard_point(transform(i, v.position), p.billboard, i_vertex);
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
    let missiles = poses[..count].iter().filter_map(|pose| {
        Some((
            arena.missile_models[usize::from(pose[0] == 312)].as_ref()?,
            *pose,
        ))
    });
    let mercs = arena
        .merc_models
        .iter()
        .enumerate()
        .filter_map(|(slot, frames)| {
            let [branch, x, y, z, _] = drive.contacts.merc_pose(slot)?;
            Some((
                frames.get(branch as usize)?,
                [0, x, y, z, 4096, 0, 0, 0, 4096, 0, 0, 0, 4096],
            ))
        });
    let pedestrians = arena
        .pedestrians
        .iter()
        .enumerate()
        .filter_map(|(i, frames)| {
            let pose = drive.contacts.pedestrian_pose(i)?;
            if pose[13] == 0 {
                return None;
            }
            Some((
                frames.get(pose[0] as usize)?,
                pose[..13].try_into().unwrap(),
            ))
        });
    for (model, pose) in missiles.chain(mercs).chain(pedestrians) {
        let origin = world([pose[1] as f32, pose[2] as f32, pose[3] as f32]);
        for p in &model.polygons {
            if p.animation
                .iter()
                .any(|a| a.selected(drive.player.frame()) != a.child)
            {
                continue;
            }
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
                    billboard_point(origin + world(rotated), p.billboard, i),
                    arena.uv(slot, v.uv.into()),
                    v.color,
                );
            }
        }
    }
    let mut flames = [[0; 4]; 80];
    let count = drive.contacts.flame_particles(&mut flames);
    let mut effects = [[0; 5]; 122];
    let effect_count = drive.contacts.effects(&mut effects);
    let particles = flames[..count].iter().map(|&[x, y, z, index]| {
        (
            [x, y, z],
            &arena.explosion_frames[index as usize],
            6,
            [1.; 4],
        )
    });
    let wreck_effects = effects[..effect_count]
        .iter()
        .filter_map(|&[x, y, z, kind, ttl]| {
            let (frame, size) = match kind {
                8 | 34 => {
                    let age = 88 - ttl;
                    let frame = if age < 24 { age } else { (age - 24) % 32 } / 2;
                    (
                        &arena.smoke_frames[0][frame as usize],
                        if kind == 8 { 16 } else { 32 },
                    )
                }
                1 => (&arena.spark_frames[(4 - ttl) as usize], 5),
                31 => (&arena.smoke_frames[1][((24 - ttl) / 2) as usize], 8),
                32 => (&arena.smoke_frames[0][(16 - ttl) as usize], 4),
                53 => (&arena.explosion_frames[(2 - ttl) as usize], 8),
                44..=50 => (&arena.burn_frames[(kind - 44 + 2 - ttl) as usize], 8),
                38..=43 => (&arena.burn_frames[(kind - 38 + 3 - ttl) as usize], 8),
                35 => (&arena.flame_frames[((104 - ttl) % 8) as usize], 12),
                27 => (&arena.flame_frames[(8 - ttl) as usize], 12),
                28 | 37 | 55 => (&arena.explosion_frames[((32 - ttl) / 2) as usize], 16),
                29 => (&arena.explosion_frames[(16 - ttl) as usize], 72),
                30 => (&arena.explosion_frames[(16 - ttl) as usize], 96),
                _ => return None,
            };
            Some((
                [x, y, z],
                frame,
                size,
                if kind == 55 {
                    [0.5, 0.5, 255. / 128., 1.]
                } else {
                    [1.; 4]
                },
            ))
        });
    for (position, frame, size, tint) in particles.chain(wreck_effects) {
        let center = world(position.map(|v| v as f32));
        let radius = f32::from(frame.width) * size as f32 / 8.;
        let slot = arena.tiles[&(Some(frame.texture), true)];
        let batch = if (frame.texture.page >> 5) & 3 == 1 {
            &mut additive
        } else {
            &mut transparent
        };
        for &i in indices(4) {
            let offset = right * if i & 1 == 0 { -radius } else { radius }
                + up * if i < 2 { radius } else { -radius };
            let [u, v] = frame.uv[i];
            batch.push(
                center + offset,
                arena.uv(slot, Vec2::new(u.into(), v.into())),
                tint,
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
    for &[x, y, z, kind, ttl] in &effects[..effect_count] {
        let delta = world([x as f32, y as f32, z as f32]) - eye;
        let depth = delta.dot(forward);
        if depth < near {
            continue;
        }
        let center = Vec2::new(delta.dot(right), delta.dot(up)) * (focal / depth);
        let (size, tint) = match kind {
            2 => (12. + (7 - ttl) as f32 * 6., [1., 0.24, 0.04, 0.82]),
            7 => (3., [0.7, 0.5, 0.3, 1.]),
            5 => (3., [0.35, 0.65, 1., 1.]),
            6 => (16., [0.2, 0.16, 0.14, 1.]),
            1 | 3 | 8 | 26..=55 | 401..=406 | 410..=414 | 430 => continue,
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
    additive.write(meshes, &view.additive);
    solids.write(meshes, &view.solids);
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
    sorted.write(meshes, &view.transparent);
}

impl Arena {
    fn uv(&self, slot: usize, uv: Vec2) -> [f32; 2] {
        [
            ((slot % (self.atlas_size[0] / TILE) * TILE) as f32 + uv.x + 0.5)
                / self.atlas_size[0] as f32,
            ((slot / (self.atlas_size[0] / TILE) * TILE) as f32 + uv.y + 0.5)
                / self.atlas_size[1] as f32,
        ]
    }
}
// UA_DASH.c places instruments on a 320 by 240 framebuffer.
struct Instruments<'a> {
    batch: &'a mut Batch,
    arena: &'a Arena,
    size: Vec2,
}
impl Instruments<'_> {
    fn quad(&mut self, [x, y, w, h]: [i32; 4], uv: [[f32; 2]; 4], colors: [[f32; 4]; 2]) {
        for &i in indices(4) {
            let px = x + if i % 2 == 1 { w } else { 0 };
            let py = y + if i >= 2 { h } else { 0 };
            let pos = Vec2::new(px as f32 / 320. - 0.5, 0.5 - py as f32 / 240.) * self.size;
            self.batch.push(pos.extend(0.), uv[i], colors[i % 2]);
        }
    }
    fn bar(&mut self, rect: [i32; 4], colors: [[u8; 3]; 2]) {
        let colors =
            colors.map(|[r, g, b]| Color::srgba_u8(r, g, b, 128).to_linear().to_f32_array());
        self.quad(rect, [self.arena.uv(0, Vec2::ZERO); 4], colors);
    }
    fn sprite(&mut self, x: i32, y: i32, f: SpriteFrame, tint: [u8; 3], semi: bool) {
        let slot = self.arena.tiles[&(Some(f.texture), semi)];
        let uv =
            f.uv.map(|[u, v]| self.arena.uv(slot, Vec2::new(u.into(), v.into())));
        let w = 1 + i32::from(f.width);
        let h = 1 + i32::from(f.uv[2][1].wrapping_sub(f.uv[0][1]));
        let tint = tint.map(|v| v as f32 / 128.);
        self.quad([x, y, w, h], uv, [[tint[0], tint[1], tint[2], 1.]; 2]);
    }
    fn number(&mut self, mut x: i32, y: i32, n: i32, digits: &[SpriteFrame; 10], tint: [u8; 3]) {
        if n >= 10 {
            let tens = digits[(n / 10 % 10) as usize];
            self.sprite(x, y, tens, tint, false);
            x += i32::from(tens.width) + 2;
        }
        self.sprite(x, y, digits[(n % 10) as usize], tint, false);
    }
}
fn health_colors(tier: u8) -> [[u8; 3]; 2] {
    match tier {
        0 => [[255, 0, 0]; 2],
        1 => [[255, 42, 0], [255, 115, 0]],
        2 => [[186, 155, 32], [221, 231, 50]],
        _ => [[0, 112, 10], [49, 148, 41]],
    }
}
fn draw_instruments(
    batch: &mut Batch,
    arena: &Arena,
    drive: &crate::driving::Drive,
    window: &Window,
) {
    let mut hud = Instruments {
        batch,
        arena,
        size: Vec2::new(window.width(), window.height()),
    };
    let icons = &arena.icons;
    let (hp, max) = drive.player.health();
    let pct = hp * 100 / max.max(1);
    let (tier, holes) = tm1_core::drive::PlayerVehicle::windshield_damage(pct);
    let cockpit = drive.camera.mode == tm1_core::drive::CarView::Cockpit
        && !drive.camera.show_body
        && drive.player.dashboard_enabled()
        && !arena.overview
        && drive.helicopter_pose().is_none();
    if drive.mirror && !arena.overview && !cockpit {
        hud.quad(
            [209, 18, 105, 34],
            [arena.uv(0, Vec2::ZERO); 4],
            [[0., 0., 0., 1.]; 2],
        );
    }
    let white = [128; 3];
    let green = [36, 120, 36];
    if cockpit && drive.player.vehicle() != tm1_core::drive::Vehicle::MrGrimm {
        for &[x, y] in &drive.windshield_holes[..holes] {
            hud.sprite(x, y, icons.hole, white, true);
        }
    } else {
        hud.bar([216, 190, 90, 10], [[10; 3]; 2]);
        hud.bar([102, 21, 88, 15], [[10; 3]; 2]);
    }
    hud.bar(
        [217, 191, (pct * 88 / 100).clamp(1, 88), 8],
        health_colors(tier),
    );
    if let Some(target) = drive.player.hud_opponent(&drive.opponents) {
        let (hp, max) = target.health();
        hud.bar(
            [103, 22, (hp * 86 / max.max(1)).clamp(1, 86), 13],
            health_colors(target.health_tier()),
        );
    }
    for (i, (x, y)) in [(102, 40), (151, 37), (216, 202), (216, 215)]
        .into_iter()
        .enumerate()
    {
        hud.sprite(
            x,
            y,
            icons.labels[i],
            if i < 2 { white } else { green },
            false,
        );
    }
    let (slot, ammo, special) = drive.player.dashboard_weapon();
    for i in 0..special.clamp(0, 5) {
        hud.sprite(262 + i32::from(i) * 9, 202, icons.labels[4], white, false);
    }
    for i in 0..drive.lives.min(3) {
        hud.sprite(309, 189 + i32::from(i) * 5, icons.life, white, true);
    }
    if let Some(icon) = icons.weapons.get(slot).filter(|_| ammo > 0) {
        hud.sprite(255, 210, *icon, white, true);
        hud.number(290, 212, i32::from(ammo), &icons.digits, green);
    }
    if drive.player.gun_overheated() {
        hud.bar([216, 214, 38, 10], [[255, 0, 0]; 2]);
    }
    hud.number(
        243,
        202,
        drive.player.turbo_charge(),
        &icons.turbo_digits,
        green,
    );
    let living = drive
        .opponents
        .iter()
        .filter(|ai| ai.health().0 > 0)
        .count() as i32;
    hud.number(134, 39, living, &icons.digits, green);
    hud.number(
        179,
        39,
        drive.opponents.len() as i32 - living,
        &icons.digits,
        [120, 36, 36],
    );
}
fn draw_radar(batch: &mut Batch, arena: &Arena, drive: &crate::driving::Drive, window: &Window) {
    use tm1_core::drive::RadarSignal;
    let scale = Vec2::new(window.width() / 320., window.height() / 240.);
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
