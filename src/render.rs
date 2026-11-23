//! Arena presentation in PlayStation ordering-table traversal order.
use anyhow::{Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    ecs::system::SystemParam,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    sprite_render::AlphaMode2d,
};
use std::collections::BTreeMap;
use tm1_assets::{
    scene::{Scene, SpriteFrame, TextureKey, Wheel},
    texture::Vram,
};

use bevy::camera::visibility::RenderLayers;

struct BodyGeometry {
    polygons: std::ops::Range<usize>,
    lods: Vec<tm1_assets::scene::LevelOfDetail>,
}

struct CarPose {
    origin: Vec3,
    matrix: [[i16; 3]; 3],
    body_stage: usize,
    shadow_visible: bool,
}
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PacketMode {
    Average,
    Add,
    Subtract,
    QuarterAdd,
    Opaque,
    SubtractOpaque,
    UntexturedOpaque,
    UntexturedAverage,
}

impl PacketMode {
    fn texture(texture: TextureKey) -> Self {
        [Self::Average, Self::Add, Self::Subtract, Self::QuarterAdd]
            [usize::from((texture.page >> 5) & 3)]
    }

    fn polygon(p: &tm1_assets::scene::Polygon) -> Self {
        match (p.texture, p.semi_transparent) {
            (Some(texture), true) => Self::texture(texture),
            (Some(_), false) => Self::Opaque,
            (None, true) => Self::UntexturedAverage,
            (None, false) => Self::UntexturedOpaque,
        }
    }
}

struct PacketMesh {
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<crate::texture_blend::TextureBlend>,
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
    bullet_frames: [Option<[SpriteFrame; 4]>; 4],
    fragment_frames: [[SpriteFrame; 4]; 2],
    drop_sprites: [SpriteFrame; 4],
    contrail_frames: [SpriteFrame; 8],
    explosion_frames: [SpriteFrame; 16],
    spark_frames: [SpriteFrame; 4],
    flame_frames: [SpriteFrame; 10],
    burn_frames: [SpriteFrame; 8],
    smoke_frames: [[SpriteFrame; 16]; 2],
    vehicle_start: usize,
    body_ranges: Vec<[BodyGeometry; 3]>,
    overview: bool,
    icons: tm1_assets::scene::DashboardIcons,
    arrows: [SpriteFrame; 4],
    pixels: Vec<u8>,
    atlas_size: [usize; 2],
    tiles: BTreeMap<(Option<TextureKey>, bool), usize>,
    mesh: Handle<Mesh>,
    windshield_mesh: Handle<Mesh>,
    packet_meshes: [Vec<PacketMesh>; 2],
    active_runs: [usize; 2],
    atlas: Handle<Image>,
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
        bullet_frames: [Option<[SpriteFrame; 4]>; 4],
        fragment_frames: [[SpriteFrame; 4]; 2],
        drop_sprites: [SpriteFrame; 4],
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
                    BodyGeometry {
                        polygons: start..scene.polygons.len(),
                        lods: variant.lods.unwrap_or_default(),
                    }
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
            .chain(bullet_frames.iter().flatten().flatten())
            .chain(&arrows)
            .map(|f| (f, true))
            .chain(
                drop_sprites
                    .iter()
                    .chain(fragment_frames.iter().flatten())
                    .map(|f| (f, false)),
            )
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
            bullet_frames,
            fragment_frames,
            drop_sprites,
            contrail_frames,
            overview: false,
            icons,
            arrows,
            pixels,
            atlas_size: [width, height],
            tiles,
            mesh: Handle::default(),
            windshield_mesh: Handle::default(),
            packet_meshes: default(),
            active_runs: [0; 2],
            atlas: default(),
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
    duel: Option<Res<crate::duel::Duel>>,
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
    let mut cockpit_image = Image::new(
        Extent3d {
            width: 320,
            height: 240,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        arena.cockpit.frame([0, 4096]),
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
    arena.atlas = texture.clone();
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
    if duel.is_none() {
        arena.windshield_mesh = meshes.add(Batch::default().mesh());
        commands.spawn((
            crate::campaign::StageEntity,
            Mesh2d(arena.windshield_mesh.clone()),
            MeshMaterial2d(blend.clone()),
            Transform::from_xyz(0., 0., -1.5),
            NoFrustumCulling,
        ));
    }
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
/// Expand the RGB555 framebuffer value of an untextured GPU command for display.
pub(crate) fn flat_color(rgb: [u8; 3], alpha: u8) -> Color {
    let [r, g, b] = rgb.map(|value| (value & 0xf8) | (value >> 5));
    Color::srgba_u8(r, g, b, alpha)
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
        backdrop.1.color = flat_color([red as u8, 0, 0], 128);
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
    mut steering: Local<[i32; 2]>,
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
    // Each JP drawing routine retains its angle between dashboard loads.
    let slot = usize::from(drive.player.vehicle() == tm1_core::drive::Vehicle::MrGrimm);
    let angle = drive.player.cockpit_steering_angle(steering[slot]);
    steering[slot] = angle;
    if angle != state.degrees {
        state.degrees = angle;
        if let Some(mut image) = images.get_mut(&sprite.image) {
            image.data = Some(
                arena
                    .cockpit
                    .frame(drive.player.wheel_sin_cos(i64::from(angle * 4096 / 360))),
            );
        }
    }
}
#[derive(Component, Default)]
pub struct HudLetters {
    last: Option<(i32, Option<tm1_core::drive::Vehicle>, u16, bool)>,
}
fn speed_text(speed: i32) -> String {
    if speed >= 1000 {
        format!("{:03}", speed % 1000)
    } else {
        format!("{speed:3}")
    }
}
pub fn hud_letters(
    window: Single<&Window>,
    drive: Res<crate::driving::Drive>,
    arena: Res<Arena>,
    mut images: ResMut<Assets<Image>>,
    mut overlay: Single<(&mut HudLetters, &mut Sprite)>,
) {
    use tm1_assets::screen::Face;
    let (state, sprite) = &mut *overlay;
    sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
    let speed = drive.player.speed_mph(3).abs();
    let sign = drive
        .player
        .hud_opponent(&drive.opponents)
        .filter(|ai| ai.health().0 > 0)
        .map(|ai| ai.vehicle());
    let mask = if drive.list_cars {
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
            .sprite_at(pixels, Face::Speed, 102, 214, &speed_text(speed));
        if at_capacity {
            arena.fonts.sprite_centered_at(
                pixels,
                Face::Control,
                159,
                155,
                "Max Carrying Capacity",
            );
        }
        if let Some(car) = sign {
            arena.fonts.sprite_centered_at(
                pixels,
                Face::Sign,
                146,
                33,
                rival_name(&arena.fonts, car),
            );
        }
        draw_rival_list(
            pixels,
            &arena.fonts,
            drive.opponents.iter().map(|ai| ai.vehicle()),
            mask,
        );
    }
}

fn rival_name(fonts: &tm1_assets::screen::Fonts, car: tm1_core::drive::Vehicle) -> &str {
    fonts.car_sign(car.card_index())
}

fn draw_rival_list(
    pixels: &mut [u8],
    fonts: &tm1_assets::screen::Fonts,
    cars: impl Iterator<Item = tm1_core::drive::Vehicle>,
    living: u16,
) {
    for (slot, car) in cars.enumerate() {
        if living & (1 << slot) != 0 {
            fonts.sprite_at(
                pixels,
                tm1_assets::screen::Face::Sign,
                220,
                70 + slot as i32 * 12,
                rival_name(fonts, car),
            );
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
    mut arena: ResMut<Arena>,
    drive: Res<crate::driving::Drive>,
    capture: Res<crate::smoke::Capture>,
    mut timings: Local<Vec<f64>>,
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
    mut renderer: PacketRenderer,
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
    let poses: Vec<_> = std::iter::once(CarPose {
        origin,
        matrix,
        body_stage: drive.player.body_stage(),
        shadow_visible: drive.player.shadow_visible(),
    })
    .chain(drive.opponents.iter().map(|ai| CarPose {
        origin: world(ai.position().map(|v| v as f32)),
        matrix: ai.rotation(),
        body_stage: ai.body_stage(),
        shadow_visible: ai.shadow_visible(),
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
    let lights = drive.contacts.car_lights();
    let mut screen_batch = Batch::default();
    let mut windshield = Batch::default();
    draw_instruments(&mut screen_batch, &mut windshield, &arena, &drive, &window);
    windshield.write(&mut renderer.meshes, &arena.windshield_mesh);
    screen_batch.write(&mut renderer.meshes, &arena.mesh);
    let packets = draw_view(
        &arena,
        &drive.player,
        &drive.contacts,
        false,
        &poses,
        &lights,
        hide_player.then_some(0),
        false,
        eye,
        (target - eye).normalize(),
        camera_up,
        window.height() * 0.85,
    );
    renderer.submit(&mut arena, false, packets);
    if mirror {
        let packets = draw_view(
            &arena,
            &drive.player,
            &drive.contacts,
            true,
            &poses,
            &lights,
            hide_player.then_some(0),
            false,
            eye,
            rear_forward,
            rear_up,
            MIRROR_FOCAL,
        );
        renderer.submit(&mut arena, true, packets);
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

// LIGHT.c uses three directional lights of the same color in each special
// environment. Approximate their intensity over the baked vertex colors.
fn body_light_color(env: u8) -> [f32; 3] {
    let channels = match env {
        1 => [255, 0, 0],
        2 => [60, 140, 170],
        _ => return [1.; 3],
    };
    channels.map(|c| (32 + c) as f32 / 160.)
}

fn packet_rgb([r, g, b]: [u8; 3]) -> [f32; 4] {
    [
        f32::from(r) / 255.,
        f32::from(g) / 255.,
        f32::from(b) / 255.,
        1.,
    ]
}

fn draw_view(
    arena: &Arena,
    player: &tm1_core::drive::PlayerVehicle,
    contacts: &tm1_core::drive::CollisionScene,
    secondary: bool,
    poses: &[CarPose],
    lights: &[u8; 9],
    hidden_car: Option<usize>,
    split_screen: bool,
    eye: Vec3,
    forward: Vec3,
    camera_up: Vec3,
    focal: f32,
) -> Batch {
    let wheel_steps = contacts.wheel_steps();
    let [sin, cos] = player
        .wheel_sin_cos(wheel_steps[4])
        .map(|n| f32::from(n) / 4096.);
    let transform = |index: usize, p: [f32; 3]| {
        if index >= arena.vehicle_start {
            let car = arena
                .body_ranges
                .partition_point(|ranges| ranges[2].polygons.end <= index);
            let pose = &poses[car];
            let p = if let Some(Wheel::Hammerhead { pivot, axis }) =
                arena.scene.polygons[index].wheel
            {
                let axis = Vec3::from_array(axis);
                let pivot = Vec3::from_array(pivot);
                let offset = Vec3::from_array(p) - pivot;
                (pivot
                    + offset * cos
                    + axis.cross(offset) * sin
                    + axis * axis.dot(offset) * (1. - cos))
                    .to_array()
            } else {
                p
            };
            pose.origin
                + world(std::array::from_fn(|i| {
                    (0..3)
                        .map(|j| pose.matrix[j][i] as f32 * p[j] / 4096.)
                        .sum()
                }))
        } else {
            world(p)
        }
    };
    let right = forward.cross(camera_up).normalize();
    let up = right.cross(forward);
    let near = arena.radius * 0.005;
    let hidden = contacts.hidden_pickups();
    let switches = contacts.arena_switch_branches();
    let (_, stand_branches) = contacts.health_stands();
    let mut batch = Batch::default();
    if !secondary {
        let [r, g, b, a] = contacts.hit_sheet();
        if a != 0 {
            let white = arena.uv(0, Vec2::ZERO);
            let height = if split_screen { 120. } else { 240. };
            for &corner in indices(4) {
                let x = if corner & 1 == 0 { -1. } else { 1. };
                let y = if corner < 2 { 1. } else { 1. - 480. / height };
                batch.push(Vec3::new(x, y, 0.5), white, packet_rgb([r, g, b]));
            }
            let mode = if a == 255 {
                PacketMode::UntexturedOpaque
            } else {
                PacketMode::UntexturedAverage
            };
            batch.order_packet(6, 0, DrawPass::HitSheet, mode);
        }
        if !split_screen && let Some(radar) = contacts.radar.frame() {
            draw_radar(&mut batch, arena, radar);
        }
    }
    let visible = (0..arena.vehicle_start).chain(
        arena
            .body_ranges
            .iter()
            .zip(poses)
            .enumerate()
            .filter(|(slot, _)| Some(*slot) != hidden_car)
            .flat_map(|(_, (ranges, pose))| {
                ranges[pose.body_stage]
                    .polygons
                    .clone()
                    .filter(|&i| pose.shadow_visible || !arena.scene.polygons[i].car_shadow)
            }),
    );
    let mut hidden_geometry = vec![false; arena.scene.polygons.len()];
    for (stages, pose) in arena.body_ranges.iter().zip(poses) {
        let body = &stages[pose.body_stage];
        let delta = pose.origin - eye;
        let matrix = tm1_core::math::Matrix {
            rotation: pose.matrix,
            ..Default::default()
        };
        let cs_position = tm1_core::math::multiply(
            &matrix,
            [delta.x as i32, -delta.z as i32, delta.y as i32],
            false,
        );
        let local_eye = cs_position.map(|v| -(v as f32));
        for lod in &body.lods {
            lod.hide_other_branches(local_eye, 3, &mut hidden_geometry[body.polygons.clone()]);
        }
    }
    for lod in arena.scene.lods.iter().flatten() {
        lod.hide_other_branches([eye.x, -eye.z, eye.y], 3, &mut hidden_geometry);
    }
    for controller in &arena.scene.visibility {
        hidden_geometry[controller.hidden_polygons([eye.x, -eye.z, eye.y])].fill(true);
    }
    for (address, branch, polygons) in &arena.scene.damage_branches {
        if contacts.damage_branch(*address) != *branch {
            hidden_geometry[polygons.clone()].fill(true);
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
        if hidden_geometry[i] {
            continue;
        }
        let p = &arena.scene.polygons[i];
        if p.animation
            .iter()
            .any(|frame| frame.selected(player.frame()) != frame.child)
        {
            continue;
        }
        if let Some(Wheel::Tire {
            slot,
            initial,
            count,
            branch,
        }) = p.wheel
            && (wheel_steps[slot] + i64::from(initial)).rem_euclid(i64::from(count))
                != i64::from(branch)
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
        if !p.double_sided
            && p.billboard.is_none()
            && back_facing([0, 1, 2].map(|j| transform(i, p.vertices[j].position)), eye)
        {
            continue;
        }
        let indices = indices(p.vertices.len());
        let tint = if i >= arena.vehicle_start {
            let car = arena
                .body_ranges
                .partition_point(|ranges| ranges[2].polygons.end <= i);
            body_light_color(lights[car])
        } else {
            [1.; 3]
        };
        for &i_vertex in indices {
            let v = &p.vertices[i_vertex];
            let point = billboard_point(transform(i, v.position), p.billboard, i_vertex);
            let [r, g, b, a] = v.color;
            batch.push(
                point,
                arena.uv(slot, v.uv.into()),
                [r * tint[0], g * tint[1], b * tint[2], a],
            );
        }
        batch.order_geometry(
            indices.len(),
            eye,
            forward,
            p.fixed_ot,
            PacketMode::polygon(p),
        );
    }
    let mut faces = [[0; 14]; 180];
    let count = contacts.contrail_faces(&mut faces);
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
            batch.push(
                points[i],
                arena.uv(slot, Vec2::new(u.into(), v.into())),
                [tint[0], tint[1], tint[2], 1.],
            );
        }
        batch.order_geometry(6, eye, forward, 0, PacketMode::Add);
    }
    let mut poses = [[0; 13]; 10];
    let count = contacts.missile_poses(&mut poses);
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
            let [branch, x, y, z, _] = contacts.merc_pose(slot)?;
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
            let pose = contacts.pedestrian_pose(i)?;
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
                .any(|a| a.selected(player.frame()) != a.child)
            {
                continue;
            }
            let mut points = [Vec3::ZERO; 4];
            for (corner, vertex) in p.vertices.iter().enumerate() {
                let rotated = std::array::from_fn(|k| {
                    (0..3)
                        .map(|j| pose[4 + 3 * j + k] as f32 * vertex.position[j] / 4096.)
                        .sum()
                });
                points[corner] = billboard_point(origin + world(rotated), p.billboard, corner);
            }
            if !p.double_sided
                && p.billboard.is_none()
                && back_facing([points[0], points[1], points[2]], eye)
            {
                continue;
            }
            let slot = arena.tiles[&(p.texture, p.semi_transparent)];
            for &i in indices(p.vertices.len()) {
                let v = &p.vertices[i];
                batch.push(points[i], arena.uv(slot, v.uv.into()), v.color);
            }
            batch.order_geometry(
                indices(p.vertices.len()).len(),
                eye,
                forward,
                p.fixed_ot,
                PacketMode::polygon(p),
            );
        }
    }
    let camera_point = |p: [i32; 3]| {
        let d = world(p.map(|n| n as f32)) - eye;
        Vec3::new(d.dot(right), d.dot(up), d.dot(forward))
    };
    let (lens, center) = if split_screen {
        (228, [160, 60])
    } else if secondary {
        (
            MIRROR_FOCAL as i32,
            [MIRROR_WIDTH as i32 / 2, MIRROR_HEIGHT as i32 / 2],
        )
    } else {
        (204, [160, 120])
    };
    let pixel_scale = focal / lens as f32;
    let bucket_depth = |bucket: u32| ((bucket * 8) as f32).max(near + 0.01);
    let drop_camera = |p: Vec3| {
        let d = p - eye;
        [
            d.dot(right) as i32,
            -d.dot(up) as i32,
            d.dot(forward) as i32,
        ]
    };
    let mut drops = [[0; 4]; 12];
    let drop_count = contacts.drawn_drops(secondary, &mut drops);
    for &[x, y, z, weapon] in &drops[..drop_count] {
        let center = world([x as f32, y as f32, z as f32]);
        let camera = drop_camera(center);
        let Some(bucket) = tm1_core::drive::drop_bucket(camera, weapon) else {
            continue;
        };
        let index = match weapon {
            6 => 0,
            8 => 1,
            9 => 2,
            10 => 3,
            _ => continue,
        };
        let sprite = &arena.drop_sprites[index];
        let tile = arena.tiles[&(Some(sprite.texture), false)];
        let billboard = tm1_core::drive::drop_billboard(camera, [lens; 2]);
        let depth = bucket_depth(bucket);
        for &i in indices(4) {
            let xy = if weapon == 9 {
                let dx = if i & 1 == 0 { -48. } else { 48. };
                let dy = if i < 2 { -48. } else { 48. };
                let corner = center + world([dx, dy, 0.]);
                tm1_core::drive::drop_screen_point(drop_camera(corner), [lens; 2])
            } else {
                billboard[i]
            };
            let point = eye
                + forward * depth
                + (right * xy[0] as f32 - up * xy[1] as f32) * (depth / lens as f32);
            batch.push(
                point,
                arena.uv(tile, Vec2::from(sprite.uv[i].map(f32::from))),
                [1.; 4],
            );
        }
        batch.order_packet(6, bucket, DrawPass::Bullets, PacketMode::Opaque);
    }
    let white = arena.uv(0, Vec2::ZERO);
    let quad = |batch: &mut Batch, points: [Vec2; 4], depth: f32, colors: [[f32; 4]; 2]| {
        for &i in indices(4) {
            let xy = points[i] * (depth / focal);
            batch.push(
                eye + forward * depth + right * xy.x + up * xy.y,
                white,
                colors[i / 2],
            );
        }
    };
    let mut bullets = [[0; 8]; 40];
    let bullet_count = contacts.drawn_bullets(secondary, &mut bullets);
    let mut arcs = [[[0; 3]; 16]; 3];
    let arc_count = contacts.taser_arcs(secondary, &mut arcs);
    let mut particles = [[0; 5]; 80];
    let count = contacts.drawn_particles(secondary, &mut particles);
    for &[x, y, z, kind, ttl] in &particles[..count] {
        use tm1_core::drive::ParticleAnimation::*;
        let Some(particle) = tm1_core::drive::particle_frame(kind, ttl) else {
            continue;
        };
        let frames = match particle.animation {
            Explosion => &arena.explosion_frames[..],
            Smoke => &arena.smoke_frames[0][..],
            Steam => &arena.smoke_frames[1][..],
            Spark => &arena.spark_frames[..],
            Burn => &arena.burn_frames[..],
            Fire => &arena.flame_frames[..],
        };
        let coords = |p| {
            let q = camera_point(p);
            [q.x as i32, -q.y as i32, q.z as i32]
        };
        let base = coords([x, y, z]);
        let top = (kind == 35).then(|| coords([x, y, z + 80]));
        let Some(corners) = player.particle_quad(
            base,
            top,
            [lens; 2],
            center,
            frames[particle.first].width,
            particle.scale,
        ) else {
            continue;
        };
        let sprite = &frames[particle.frame];
        let tile = arena.tiles[&(Some(sprite.texture), true)];
        let depth = bucket_depth(particle.ordering_bucket(base[2]));
        let tint = if kind == 55 {
            [0.5, 0.5, 255. / 128., 1.]
        } else {
            [1.; 4]
        };
        for &i in indices(4) {
            let [u, v] = corners[i];
            let offset = right * u as f32 - up * v as f32;
            batch.push(
                eye + forward * depth + offset * (depth * pixel_scale / focal),
                arena.uv(tile, Vec2::from(sprite.uv[i].map(f32::from))),
                tint,
            );
        }
        batch.order_packet(
            6,
            particle.ordering_bucket(base[2]),
            DrawPass::Explosions,
            PacketMode::texture(sprite.texture),
        );
    }
    let mut fragments = [[0; 5]; 20];
    let fragment_count = contacts.fragment_poses(secondary, &mut fragments);
    for &[x, y, z, variant, phase] in &fragments[..fragment_count] {
        let camera = camera_point([x, y, z]);
        let coords = [camera.x as i32, -camera.y as i32, camera.z as i32];
        let Some((bucket, corners)) = tm1_core::drive::project_fragment(coords, [lens; 2], center)
        else {
            continue;
        };
        let sprite = &arena.fragment_frames[variant as usize][phase as usize];
        let tile = arena.tiles[&(Some(sprite.texture), false)];
        let draw_depth = bucket_depth(bucket);
        for &i in indices(4) {
            let [u, v] = corners[i];
            let offset = right * u as f32 - up * v as f32;
            batch.push(
                eye + forward * draw_depth + offset * (draw_depth * pixel_scale / focal),
                arena.uv(tile, Vec2::from(sprite.uv[i].map(f32::from))),
                [1.; 4],
            );
        }
        batch.order_packet(6, bucket, DrawPass::Explosions, PacketMode::Opaque);
    }
    for arc in &arcs[..arc_count] {
        let points = arc.map(|p| {
            let camera = camera_point(p);
            let coords = [camera.x as i32, -camera.y as i32, camera.z as i32];
            let (bucket, [x, y]) = tm1_core::drive::project_taser_point(coords, [lens; 2], center)?;
            Some((Vec2::new(x as f32, -y as f32) * pixel_scale, bucket))
        });
        for (j, segment) in points.windows(2).enumerate() {
            let (Some((a, _)), Some((b, bucket))) = (segment[0], segment[1]) else {
                continue;
            };
            let normal = (b - a).normalize_or_zero().perp() * (0.5 * pixel_scale);
            let colors = [j, j + 1].map(|i| {
                let level = (250 - 12 * i) as u8;
                packet_rgb([level, level, 255])
            });
            quad(
                &mut batch,
                [a - normal, a + normal, b - normal, b + normal],
                bucket_depth(bucket),
                colors,
            );
            batch.order_packet(6, bucket, DrawPass::Bullets, PacketMode::UntexturedOpaque);
        }
    }
    for &[x, y, z, px, py, pz, kind, life] in &bullets[..bullet_count] {
        let head = camera_point([x, y, z]);
        let tail = camera_point([px, py, pz]);
        let camera_coords = |p: Vec3| [p.x as i32, -p.y as i32, p.z as i32];
        let head = camera_coords(head);
        let Some((bucket, primitive)) =
            tm1_core::bullet::project(kind, life, head, camera_coords(tail), [lens; 2], center)
        else {
            continue;
        };
        let screen = |[x, y]: [i32; 2]| Vec2::new(x as f32, -y as f32) * pixel_scale;
        let depth = bucket_depth(bucket);
        let vertex = |point: Vec2| {
            eye + forward * depth + (right * point.x + up * point.y) * (depth / focal)
        };
        use tm1_core::bullet::Primitive;
        match primitive {
            Primitive::Line { points, colors } => {
                let [a, b] = points.map(screen);
                let normal = (b - a).normalize_or_zero().perp() * (0.5 * pixel_scale);
                quad(
                    &mut batch,
                    [a - normal, a + normal, b - normal, b + normal],
                    depth,
                    colors.map(packet_rgb),
                );
                batch.order_packet(6, bucket, DrawPass::Bullets, PacketMode::UntexturedOpaque);
            }
            Primitive::Triangle { points, colors } => {
                for (point, color) in points.into_iter().zip(colors) {
                    batch.push(vertex(screen(point)), white, packet_rgb(color));
                }
                batch.order_packet(3, bucket, DrawPass::Bullets, PacketMode::UntexturedOpaque);
            }
            Primitive::Sprite {
                center,
                radius,
                group,
                frame,
                tint,
            } => {
                let Some(frames) = &arena.bullet_frames[group] else {
                    continue;
                };
                let sprite = &frames[frame];
                let tile = arena.tiles[&(Some(sprite.texture), true)];
                let [r, g, b] = tint.map(|v| f32::from(v) / 128.);
                for &i in indices(4) {
                    let point = [
                        center[0] + if i & 1 == 0 { -radius } else { radius },
                        center[1] + if i < 2 { -radius } else { radius },
                    ];
                    batch.push(
                        vertex(screen(point)),
                        arena.uv(tile, Vec2::from(sprite.uv[i].map(f32::from))),
                        [r, g, b, 1.],
                    );
                }
                batch.order_packet(
                    6,
                    bucket,
                    DrawPass::Bullets,
                    PacketMode::texture(sprite.texture),
                );
            }
        }
    }
    batch
}

impl Arena {
    fn sprite_rects(
        &self,
        frame: SpriteFrame,
        shift: u8,
        semi: bool,
        mut draw: impl FnMut([i32; 4], [[f32; 2]; 4]),
    ) {
        let slot = self.tiles[&(Some(frame.texture), semi)];
        let [u, v] = frame.uv[0];
        let width = i32::from(frame.width.wrapping_add(1));
        let height = i32::from(frame.uv[2][1].wrapping_sub(v).wrapping_add(1));
        for (dy, v, h) in sprite_axis(v, height) {
            for (dx, u, w) in sprite_axis(u.wrapping_add(shift), width) {
                let uv = [[u, v], [u + w, v], [u, v + h], [u + w, v + h]]
                    .map(|[u, v]| self.uv(slot, Vec2::new(u as f32 - 0.5, v as f32 - 0.5)));
                draw([dx, dy, w, h], uv);
            }
        }
    }
    fn uv(&self, slot: usize, uv: Vec2) -> [f32; 2] {
        [
            ((slot % (self.atlas_size[0] / TILE) * TILE) as f32 + uv.x + 0.5)
                / self.atlas_size[0] as f32,
            ((slot / (self.atlas_size[0] / TILE) * TILE) as f32 + uv.y + 0.5)
                / self.atlas_size[1] as f32,
        ]
    }
}
// A PS1 SPRT advances one texel per pixel and wraps inside its own texture page.
fn sprite_axis(start: u8, length: i32) -> impl Iterator<Item = (i32, i32, i32)> {
    let first = length.min(256 - i32::from(start));
    [(0, i32::from(start), first), (first, 0, length - first)]
        .into_iter()
        .filter(|&(_, _, len)| len > 0)
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
        let colors = colors.map(|rgb| flat_color(rgb, 128).to_linear().to_f32_array());
        self.quad(rect, [self.arena.uv(0, Vec2::ZERO); 4], colors);
    }
    fn sprite(&mut self, x: i32, y: i32, f: SpriteFrame, tint: [u8; 3], semi: bool) {
        let tint = tint.map(|v| v as f32 / 128.);
        self.arena.sprite_rects(f, 0, semi, |[dx, dy, w, h], uv| {
            self.quad(
                [x + dx, y + dy, w, h],
                uv,
                [[tint[0], tint[1], tint[2], 1.]; 2],
            );
        });
    }
    fn number(&mut self, x: i32, y: i32, n: i32, digits: &[SpriteFrame; 10], tint: [u8; 3]) {
        for (x, digit) in counter_digits(x, n, digits) {
            self.sprite(x, y, digit, tint, false);
        }
    }
}
fn counter_digits(
    mut x: i32,
    n: i32,
    digits: &[SpriteFrame; 10],
) -> impl Iterator<Item = (i32, SpriteFrame)> {
    let tens = (n >= 10).then(|| {
        let digit = digits[(n / 10 % 10) as usize];
        let left = x;
        x += i32::from(digit.width.wrapping_add(1)) + 1;
        (left, digit)
    });
    tens.into_iter()
        .chain(std::iter::once((x, digits[(n % 10) as usize])))
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
    windshield: &mut Batch,
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
        // Reverse ordering-table traversal draws the glass before the cockpit panels.
        let mut glass = Instruments {
            batch: windshield,
            arena,
            size: hud.size,
        };
        for &[x, y] in &drive.windshield_holes[..holes] {
            glass.sprite(x, y, icons.hole, white, true);
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
    if drive.player.gun_overheated() {
        hud.bar([216, 214, 38, 10], [[255, 0, 0]; 2]);
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
fn draw_radar(batch: &mut Batch, arena: &Arena, radar: &tm1_core::drive::RadarFrame) {
    use tm1_core::drive::RadarSignal;
    let white = arena.uv(0, Vec2::ZERO);
    let mut rect = |x: i32, y: i32, w: i32, h: i32, uv: [[f32; 2]; 4], color: [f32; 4], mode| {
        let points = [[x, y], [x + w, y], [x, y + h], [x + w, y + h]];
        for &i in indices(4) {
            let [px, py] = points[i];
            batch.push(
                Vec3::new(px as f32 / 160. - 1., 1. - py as f32 / 120., 0.5),
                uv[i],
                color,
            );
        }
        batch.order_packet(6, 0, DrawPass::Radar, mode);
    };
    for (i, x, y) in [(0, 47, 26), (1, 47, 77), (3, 10, 49), (2, 89, 49)] {
        if radar.edges[i] {
            let f = arena.arrows[i];
            arena.sprite_rects(f, u8::from(i != 3), true, |[dx, dy, w, h], uv| {
                rect(
                    x + dx,
                    y + dy,
                    w,
                    h,
                    uv,
                    [1.; 4],
                    PacketMode::texture(f.texture),
                );
            });
        }
    }
    // Radar signals are prepended to the PS1 ordering table.
    for &mark in &radar.marks {
        match mark {
            RadarSignal::Contact { x, y, rgb } => rect(
                53 + x,
                55 + y,
                2,
                2,
                [white; 4],
                packet_rgb(rgb),
                PacketMode::UntexturedOpaque,
            ),
            RadarSignal::Stand { x, y } => {
                for (dx, dy) in [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)] {
                    rect(
                        53 + x + dx,
                        55 + y + dy,
                        1,
                        1,
                        [white; 4],
                        packet_rgb([255, 200, 255]),
                        PacketMode::UntexturedOpaque,
                    );
                }
            }
        }
    }
    rect(
        51,
        55,
        5,
        1,
        [white; 4],
        packet_rgb([0, 48, 0]),
        PacketMode::UntexturedOpaque,
    );
    rect(
        53,
        54,
        1,
        3,
        [white; 4],
        packet_rgb([0, 48, 0]),
        PacketMode::UntexturedOpaque,
    );
    rect(
        16,
        32,
        73,
        45,
        [white; 4],
        packet_rgb([52, 113, 17]),
        PacketMode::UntexturedAverage,
    );
}
// The PS1 screen Y axis points down. geomProc keeps clockwise faces in the normal view.
fn back_facing([a, b, c]: [Vec3; 3], eye: Vec3) -> bool {
    (b - a).cross(c - a).dot(eye - a) > 0.
}

fn indices(n: usize) -> &'static [usize] {
    if n == 4 {
        &[0, 1, 2, 2, 1, 3]
    } else {
        &[0, 1, 2]
    }
}
// RT submits these passes in this order. AddPrim prepends within each bucket.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DrawPass {
    HitSheet,
    Bullets,
    Explosions,
    Radar,
    Hierarchy,
}

struct DrawPacket {
    bucket: u32,
    pass: DrawPass,
    mode: PacketMode,
    vertices: std::ops::Range<u32>,
}

#[derive(Default)]
struct Batch {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    packets: Vec<DrawPacket>,
}
impl Batch {
    fn push(&mut self, pos: Vec3, uv: [f32; 2], color: [f32; 4]) {
        self.positions.push(pos.to_array());
        self.uvs.push(uv);
        self.colors.push(color);
    }
    fn order_packet(&mut self, vertex_count: usize, bucket: u32, pass: DrawPass, mode: PacketMode) {
        let end = self.positions.len() as u32;
        self.packets.push(DrawPacket {
            bucket,
            pass,
            mode,
            vertices: end - vertex_count as u32..end,
        });
    }

    fn geometry_bucket(
        &mut self,
        vertex_count: usize,
        eye: Vec3,
        forward: Vec3,
        fixed_ot: u32,
    ) -> Option<u32> {
        let start = self.positions.len() - vertex_count;
        let vertices = &self.positions[start..];
        let depth = |i: usize| (Vec3::from_array(vertices[i]) - eye).dot(forward) as u32;
        // geomProc's triangle path compares SZ1 and SZ3, then SZ3 again.
        // The quad path includes all four GTE depths (expanded indices 0,1,2,2,1,3).
        let mut farthest = depth(0).max(depth(2));
        if vertex_count == 6 {
            farthest = farthest.max(depth(1)).max(depth(5));
        }
        let bucket = farthest >> 3;
        if !(2..=4095).contains(&bucket) {
            self.positions.truncate(start);
            self.uvs.truncate(start);
            self.colors.truncate(start);
            return None;
        }
        Some(if fixed_ot == 0 { bucket } else { fixed_ot })
    }

    fn order_geometry(
        &mut self,
        count: usize,
        eye: Vec3,
        forward: Vec3,
        fixed: u32,
        mode: PacketMode,
    ) {
        if let Some(bucket) = self.geometry_bucket(count, eye, forward, fixed) {
            self.order_packet(count, bucket, DrawPass::Hierarchy, mode);
        }
    }

    fn runs(mut self) -> Vec<(bool, Batch)> {
        self.packets
            .sort_unstable_by_key(|p| std::cmp::Reverse((p.bucket, p.pass, p.vertices.start)));
        let mut runs: Vec<(bool, Batch)> = Vec::new();
        for packet in self.packets {
            // Reverse subtraction cannot replace an opaque texel. Emit the two
            // disjoint texture masks consecutively, before the following packet.
            let ordinary =
                (packet.mode == PacketMode::Subtract).then_some(PacketMode::SubtractOpaque);
            for mode in std::iter::once(packet.mode).chain(ordinary) {
                let subtract = mode == PacketMode::Subtract;
                if runs.last().is_none_or(|r| r.0 != subtract) {
                    runs.push((subtract, Batch::default()));
                }
                let run = &mut runs.last_mut().unwrap().1;
                let range = packet.vertices.start as usize..packet.vertices.end as usize;
                run.positions
                    .extend_from_slice(&self.positions[range.clone()]);
                run.uvs.extend_from_slice(&self.uvs[range.clone()]);
                let screen = matches!(packet.pass, DrawPass::HitSheet | DrawPass::Radar);
                let flags = mode as u8 | if screen { 0x10 } else { 0 };
                run.colors.extend(
                    self.colors[range]
                        .iter()
                        .map(|&[r, g, b, _]| [r, g, b, f32::from(flags)]),
                );
            }
        }
        runs
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
        if self.positions.is_empty()
            && meshes
                .get(handle)
                .is_some_and(|mesh| mesh.count_vertices() == 0)
        {
            return;
        }
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        }
    }
}

#[derive(SystemParam)]
pub struct PacketRenderer<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<crate::texture_blend::TextureBlend>>,
}

impl PacketRenderer<'_, '_> {
    fn submit(&mut self, arena: &mut Arena, secondary: bool, packets: Batch) {
        let view = usize::from(secondary);
        let pool = &mut arena.packet_meshes[view];
        let previous = arena.active_runs[view];
        let runs = packets.runs();
        let active = runs.len();
        for (index, (subtract, run)) in runs.into_iter().enumerate() {
            if let Some(item) = pool.get(index) {
                if index >= previous {
                    self.commands
                        .entity(item.entity)
                        .insert(Visibility::Inherited);
                }
                if self
                    .materials
                    .get(&item.material)
                    .is_some_and(|m| m.subtract != subtract)
                {
                    self.materials.get_mut(&item.material).unwrap().subtract = subtract;
                }
                run.write(&mut self.meshes, &item.mesh);
            } else {
                let mesh = self.meshes.add(run.mesh());
                let material = self.materials.add(crate::texture_blend::TextureBlend {
                    atlas: arena.atlas.clone(),
                    subtract,
                    order: index,
                });
                let entity = self
                    .commands
                    .spawn((
                        crate::campaign::StageEntity,
                        Mesh2d(mesh.clone()),
                        MeshMaterial2d(material.clone()),
                        RenderLayers::layer(if secondary { 3 } else { 1 }),
                        NoFrustumCulling,
                    ))
                    .id();
                pool.push(PacketMesh {
                    entity,
                    mesh,
                    material,
                });
            }
        }
        for item in pool.iter().take(previous).skip(active) {
            self.commands.entity(item.entity).insert(Visibility::Hidden);
        }
        arena.active_runs[view] = active;
    }
}

pub fn duel_view(
    mut arena: ResMut<Arena>,
    duel: Res<crate::duel::Duel>,
    window: Single<&Window>,
    mut renderer: PacketRenderer,
    mut images: ResMut<Assets<Image>>,
    mut display: Single<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (With<MirrorDisplay>, Without<WorldCamera>),
    >,
    mut cameras: Query<
        (
            &mut Transform,
            &mut Camera,
            &mut Projection,
            Has<SkyCamera>,
            Has<MirrorCamera>,
        ),
        With<WorldCamera>,
    >,
) {
    let poses = duel.cars.each_ref().map(|car| CarPose {
        origin: world(car.position().map(|v| v as f32)),
        matrix: car.rotation(),
        body_stage: car.body_stage(),
        shadow_visible: car.shadow_visible(),
    });
    let width = window.physical_width();
    let height = window.physical_height();
    if width == 0 || height < 2 {
        return;
    }
    // Composite the second viewport after the primary window camera. A Metal
    // attachment clear can cover pixels outside the camera viewport.
    let bottom = height - height / 2;
    let scale = window.scale_factor();
    let (sprite, transform, visibility) = &mut *display;
    if let Some(mut image) = images.get_mut(&sprite.image) {
        if image.width() != width || image.height() != bottom {
            image.resize(Extent3d {
                width,
                height: bottom,
                depth_or_array_layers: 1,
            });
        }
    }
    sprite.flip_x = false;
    sprite.custom_size = Some(Vec2::new(width as f32, bottom as f32) / scale);
    transform.translation = Vec3::new(0., -((height / 2) as f32) / (2. * scale), -1.);
    **visibility = Visibility::Visible;
    let mut panel = Batch::default();
    let mut hud = Instruments {
        batch: &mut panel,
        arena: &arena,
        size: Vec2::new(window.width(), window.height()),
    };
    hud.quad(
        [0, 116, 320, 6],
        [arena.uv(0, Vec2::ZERO); 4],
        [[0., 0., 0., 1.]; 2],
    );
    for (slot, car) in duel.cars.iter().enumerate() {
        let bias = if slot == 0 { -120 } else { 0 };
        let (hp, max) = car.health();
        let percent = hp * 100 / max.max(1);
        let (tier, _) = tm1_core::drive::PlayerVehicle::windshield_damage(percent);
        hud.bar([216, 190 + bias, 90, 10], [[10; 3]; 2]);
        hud.bar(
            [217, 191 + bias, (percent * 88 / 100).clamp(1, 88), 8],
            health_colors(tier),
        );
        let icons = &arena.icons;
        let (weapon, ammo, special) = car.dashboard_weapon();
        let green = [36, 120, 36];
        hud.sprite(309, 189 + bias, icons.life, [128; 3], true);
        if car.gun_overheated() {
            hud.bar([216, 214 + bias, 38, 10], [[255, 0, 0]; 2]);
        }
        for (index, y) in [(2, 202), (3, 215)] {
            hud.sprite(216, y + bias, icons.labels[index], green, false);
        }
        hud.number(
            243,
            202 + bias,
            car.turbo_charge(),
            &icons.turbo_digits,
            green,
        );
        for i in 0..special.clamp(0, 5) {
            hud.sprite(
                262 + 9 * i32::from(i),
                202 + bias,
                icons.labels[4],
                [128; 3],
                false,
            );
        }
        if let Some(icon) = icons.weapons.get(weapon).filter(|_| ammo > 0) {
            hud.sprite(255, 210 + bias, *icon, [128; 3], true);
            hud.number(290, 212 + bias, i32::from(ammo), &icons.digits, green);
        }
    }
    panel.write(&mut renderer.meshes, &arena.mesh);
    let lights = duel.scene.car_lights();
    for slot in 0..2 {
        let car = &duel.cars[slot];
        let view = &duel.cameras[slot];
        let radians = std::f32::consts::TAU / 4096.;
        let yaw = Quat::from_rotation_y(-(car.heading() as f32) * radians);
        let (eye, forward, up) = if let Some([x, y, z, pitch, yaw]) = view.fixed_pose {
            let (sp, cp) = (pitch as f32 * radians).sin_cos();
            let (sy, cy) = (yaw as f32 * radians).sin_cos();
            (
                world([x as f32, y as f32, z as f32]),
                Vec3::new(sy * cp, -sp, -cy * cp),
                Vec3::new(sy * sp, cp, -cy * sp),
            )
        } else {
            let (sin, cos) = (view.pitch as f32 * radians).sin_cos();
            (
                poses[slot].origin + yaw * world(view.offset.map(|v| v as f32)),
                yaw * Vec3::new(0., -sin, -cos),
                yaw * Vec3::new(0., cos, -sin),
            )
        };
        let top = if slot == 0 { 0 } else { height / 2 };
        let h = if slot == 0 { height / 2 } else { height - top };
        let focal = window.height() * 228. / 240.;
        for (mut transform, mut cam, mut projection, sky, second) in &mut cameras {
            if usize::from(second) != slot {
                continue;
            }
            cam.is_active = true;
            cam.viewport = (slot == 0).then_some(bevy::camera::Viewport {
                physical_position: UVec2::new(0, top),
                physical_size: UVec2::new(width, h),
                ..default()
            });
            if let Projection::Perspective(p) = &mut *projection {
                p.fov = 2. * ((h as f32 / window.scale_factor()) / (2. * focal)).atan();
            }
            *transform = Transform::from_translation(eye).looking_to(forward, up);
            if sky {
                transform.translation -= poses[slot].origin * Vec3::new(1., 0., 1.);
            }
        }
        let packets = draw_view(
            &arena,
            car,
            &duel.scene,
            slot != 0,
            &poses,
            &lights,
            (!view.show_body).then_some(slot),
            true,
            eye,
            forward,
            up,
            focal,
        );
        renderer.submit(&mut arena, slot != 0, packets);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sprite_axis_preserves_every_texel_across_page_wrap() {
        for origin in 0..=255u8 {
            for width in [0, 1, 8, 31, 128, 255] {
                let mut count = 0;
                for (offset, texel, length) in super::sprite_axis(origin, width) {
                    assert_eq!(offset, count);
                    assert!(texel + length <= 256);
                    for i in 0..length {
                        assert_eq!(texel + i, (i32::from(origin) + offset + i) % 256);
                    }
                    count += length;
                }
                assert_eq!(count, width);
            }
        }
    }

    use super::*;
    use tm1_core::drive::Vehicle;

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn speed_digits_match_jp_font_request() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        // Enter after the four-byte string copy, using the template from the executable.
        let template = cpu.word(0x8018c2b0).to_le_bytes();
        for speed in (0..=2010).chain([9999, 10000, i32::MAX]) {
            cpu.put(0x801ff020, &template);
            cpu.call_until(0x80145a70, &[0, 0, speed as u32], 0x8010b4b8);
            assert_eq!(&cpu.r[4..7], &[4, 102, 214]);
            let text = cpu.word(cpu.word(cpu.r[29] + 0x14)).to_le_bytes();
            assert_eq!(&text[..3], speed_text(speed).as_bytes(), "{speed}");
            assert_eq!(text[3], 0);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn counter_digits_match_jp_sprite_packets() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let icons = tm1_assets::scene::Dmd::parse(&data)
            .unwrap()
            .dashboard_icons()
            .unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let (sprites, table, ot) = (0x80010000, 0x80011000, 0x80012000);
        for (x, digits) in [(290, icons.digits), (243, icons.turbo_digits)] {
            for (i, f) in digits.iter().enumerate() {
                let [u, v] = f.uv[0];
                cpu.put(
                    table + 8 * i as u32,
                    &[
                        u,
                        v,
                        f.width.wrapping_add(1),
                        f.uv[2][1].wrapping_sub(v).wrapping_add(1),
                    ],
                );
            }
            for n in (0..=220).chain([999, 1000, 32767]) {
                cpu.put(ot, &u32::MAX.to_le_bytes());
                cpu.put(sprites + 8, &(x as u16).to_le_bytes());
                cpu.call(0x80138790, &[n as u32, sprites, table, ot]);
                let mut link = cpu.word(ot) & 0xffffff;
                let mut packets = Vec::new();
                while link != 0xffffff {
                    packets.push((cpu.word(link + 8) as u16 as i32, cpu.word(link + 12) as u16));
                    link = cpu.word(link) & 0xffffff;
                }
                packets.reverse();
                assert_eq!(
                    counter_digits(x, n, &digits)
                        .map(|(x, f)| (x, u16::from_le_bytes(f.uv[0])))
                        .collect::<Vec<_>>(),
                    packets,
                    "{n}"
                );
            }
        }
    }

    #[test]
    fn battle_face_winding_matches_projected_screen_area() {
        let points = [
            Vec3::new(-2., 3., -4.),
            Vec3::new(4., -1., -8.),
            Vec3::new(0., 2., -2.),
        ];
        for eye_x in [-10., 0., 12.] {
            let eye = Vec3::X * eye_x;
            for face in [points, [points[1], points[0], points[2]], [points[0]; 3]] {
                let [a, b, c] = face.map(|p| {
                    let v = p - eye;
                    Vec2::new(v.x, -v.y) * (200. / -v.z)
                });
                assert_eq!(back_facing(face, eye), (b - a).perp_dot(c - a) < 0.);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn blended_faces_follow_jp_packet_links() {
        use DrawPass::*;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let ot = 0x80010000;
        let packets = 0x80011000;
        cpu.put(ot, &[0xff; 16 * 4]);
        let mut batch = Batch::default();
        let mut ranges = Vec::new();
        // Mesh construction visits the hierarchy first. RT submits it last.
        use PacketMode::*;
        let faces = [
            (Hierarchy, 8, 4, Add),
            (Hierarchy, 2, 3, Subtract),
            (Explosions, 8, 4, QuarterAdd),
            (Explosions, 8, 4, Average),
            (Bullets, 12, 4, Opaque),
            (Bullets, 8, 4, Subtract),
            (Radar, 0, 4, UntexturedAverage),
            (HitSheet, 0, 4, UntexturedOpaque),
            (Bullets, 0, 3, Opaque),
        ];
        for (pass, bucket, n, mode) in faces {
            let start = batch.positions.len() as u32;
            for &corner in indices(n) {
                let offset = if n == 4 {
                    [-12., -8., -4., 0.][corner]
                } else {
                    0.
                };
                batch.push(
                    Vec3::Z * ((8 * bucket) as f32 + offset),
                    [batch.positions.len() as f32, 0.],
                    [1.; 4],
                );
            }
            let len = indices(n).len();
            if pass == Hierarchy {
                batch.order_geometry(len, Vec3::ZERO, Vec3::Z, 0, mode);
            } else {
                batch.order_packet(len, bucket, pass, mode);
            }
            ranges.push(start..start + len as u32);
        }
        for pass in [HitSheet, Bullets, Explosions, Radar, Hierarchy] {
            for (i, &(source, bucket, _, _)) in faces.iter().enumerate() {
                if source == pass {
                    cpu.call(0x80161a60, &[ot + 4 * bucket, packets + 4 * i as u32]);
                }
            }
        }
        let mut expected = Vec::new();
        for bucket in (0..16).rev() {
            let mut next = cpu.word(ot + 4 * bucket) & 0xffffff;
            while next != 0xffffff {
                let i = (next - (packets & 0xffffff)) / 4;
                let mode = faces[i as usize].3;
                let flags = match faces[i as usize].0 {
                    Radar | HitSheet => 0x10 | mode as u8,
                    _ => mode as u8,
                };
                expected.extend(ranges[i as usize].clone().map(|v| (v, flags)));
                if mode == Subtract {
                    expected.extend(
                        ranges[i as usize]
                            .clone()
                            .map(|v| (v, SubtractOpaque as u8)),
                    );
                }
                next = cpu.word(next) & 0xffffff;
            }
        }
        let actual: Vec<_> = batch
            .runs()
            .into_iter()
            .flat_map(|(subtract, run)| {
                run.uvs.into_iter().zip(run.colors).map(move |(uv, color)| {
                    let mode = color[3] as u8;
                    assert_eq!(subtract, mode == Subtract as u8);
                    (uv[0] as u32, mode)
                })
            })
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_geometry_matches_jp_depth_selection() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let jump = |address: u32| 0x08000000 | ((address >> 2) & 0x03ffffff);
        let words = |cpu: &mut crate::reference_cpu::Cpu, addr, code: &[u32]| {
            for (i, word) in code.iter().enumerate() {
                cpu.put(addr + 4 * i as u32, &word.to_le_bytes());
            }
        };
        // Run the JP instructions after the GTE depth reads. Skip color/UV
        // emission and return the selected bucket, or zero at geomNextPrim.
        words(&mut cpu, 0x800f7e64, &[jump(0x800f7fec), 0]);
        words(&mut cpu, 0x800f8014, &[0x03e00008, 0x01e01021]);
        words(&mut cpu, 0x800f8154, &[0x03e00008, 0x00001021]);
        for (n, entry) in [(3, 0x800f7c18), (4, 0x800f7d0c)] {
            // Copy the four input depths into s4..s7 and set t0 to the scratchpad.
            words(
                &mut cpu,
                0x80010000,
                &[
                    0x0080a021,
                    0x00a0a821,
                    0x00c0b021,
                    0x00e0b821,
                    0x3c081f80,
                    jump(entry),
                    0,
                ],
            );
            for depths in [
                [15; 4],
                [16; 4],
                [32767; 4],
                [32768; 4],
                [60, 240, 70, 40],
                [10, 9, 8, 255],
                [65, 2, 12, 0],
                [8, 2, 72, 0],
            ] {
                for fixed in [0u32, 1, 2047, 4095] {
                    cpu.put(0x1f80007c, &fixed.to_le_bytes());
                    let expected = cpu.call(0x80010000, &depths);
                    let mut batch = Batch::default();
                    for &i in indices(n) {
                        batch.push(Vec3::Z * depths[i] as f32, [0.; 2], [1.; 4]);
                    }
                    batch.order_geometry(
                        indices(n).len(),
                        Vec3::ZERO,
                        Vec3::Z,
                        fixed,
                        PacketMode::Opaque,
                    );
                    assert_eq!(batch.positions.is_empty(), expected == 0);
                    assert_eq!(
                        batch.packets.first().map_or(0, |p| p.bucket),
                        expected,
                        "{n} corners, depths {depths:?}, fixed {fixed}"
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn warehouse_lod_ranges_match_retail_traversal() {
        verify_lod_ranges(
            "UADMD/WH1.DMD",
            &[0x80042b40, 0x80035ec8, 0x80036638, 0x800367a8],
            3,
            12000,
            13,
        );
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn showroom_lod_ranges_match_retail_traversal() {
        // Sweet Tooth, Yellow Jacket, Pit Viper and a Hammerhead wheel.
        verify_lod_ranges(
            "UADMD/CARS.DMD",
            &[0x8002320c, 0x80024ef0, 0x800352a0, 0x80028280],
            8,
            96000,
            101,
        );
    }

    fn verify_lod_ranges(path: &str, nodes: &[u32], scale: u32, distance: i32, step_size: usize) {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read(path).unwrap();
        let mut db = tm1_assets::scene::Dmd::parse(&data).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(u32::from_le_bytes(data[12..16].try_into().unwrap()), &data);
        cpu.call(0x800fe9a0, &[scale]);
        // Capture the child passed to hierPush, bypassing the rest of the hierarchy.
        cpu.put(0x800fd7dc, &[8, 0, 224, 3, 0, 0, 0, 0]);
        let mut selections = std::collections::BTreeSet::new();
        for &node in nodes {
            db.roots = vec![(node, 1)];
            let scene = db.battle_world().unwrap();
            let lod = scene.lods.as_ref().unwrap().last().unwrap();
            let origin = [4, 8, 12].map(|off| cpu.word(node + off) as i32);
            let mut children = std::collections::BTreeMap::new();
            for i in 0..cpu.word(node + 16) {
                let record = cpu.word(node + 20 + 4 * i);
                let ptr = cpu.word(record + 8);
                db.roots = vec![(ptr, 1)];
                let child = db.battle_world().unwrap();
                children.insert(
                    ptr,
                    child
                        .polygons
                        .iter()
                        .map(|p| p.source_offset)
                        .collect::<Vec<_>>(),
                );
            }
            for step in (-distance..=distance).step_by(step_size) {
                let eye = [origin[0] + step, origin[1], origin[2]];
                for (i, pos) in eye.iter().enumerate() {
                    cpu.put(0x80011008 + 4 * i as u32, &(-pos).to_le_bytes());
                }
                let mut child = None;
                cpu.call_observed(0x800fe144, &[node, 0x80011000], 0x80000080, |cpu| {
                    if cpu.pc() == 0x800fd7dc {
                        child = Some(cpu.r[4]);
                    }
                });
                let mut hidden = vec![false; scene.polygons.len()];
                lod.hide_other_branches(eye.map(|v| v as f32), scale, &mut hidden);
                let visible: Vec<_> = scene
                    .polygons
                    .iter()
                    .zip(hidden)
                    .filter_map(|(p, h)| (!h).then_some(p.source_offset))
                    .collect();
                let expected = child
                    .map(|ptr| children[&ptr].as_slice())
                    .unwrap_or_default();
                assert_eq!(visible, expected, "node {node:x} eye {eye:?}");
                selections.insert(child);
            }
        }
        assert!(selections.len() > 4);
    }

    fn draw_jp_font_packets(cpu: &crate::reference_cpu::Cpu, ot: u32, tim: &[u8]) -> Vec<u8> {
        let bitmap = 8 + u32::from_le_bytes(tim[8..12].try_into().unwrap()) as usize;
        let stride =
            2 * u16::from_le_bytes(tim[bitmap + 8..bitmap + 10].try_into().unwrap()) as usize;
        let mut expected = vec![0; 320 * 240 * 4];
        let mut packet = cpu.word(ot) & 0xffffff;
        while packet != 0xffffff {
            let tag = cpu.word(packet);
            if tag >> 24 == 4 {
                let rgb = cpu.word(packet + 4).to_le_bytes();
                let xy = cpu.word(packet + 8);
                let uv = cpu.word(packet + 12);
                let wh = cpu.word(packet + 16);
                let (u, v) = (uv as u8 as usize, (uv >> 8) as u8 as usize - 160);
                for dy in 0..(wh >> 16) as usize {
                    let y = (xy >> 16) as usize + dy;
                    for dx in 0..(wh as u16) as usize {
                        let x = xy as u16 as usize + dx;
                        if x >= 320 || y >= 240 {
                            continue;
                        }
                        let texel = tim[bitmap + 12 + (v + dy) * stride + (u + dx) / 2];
                        let index = usize::from((texel >> (4 * ((u + dx) % 2))) & 15);
                        let at = 20 + index * 2;
                        let word = u16::from_le_bytes([tim[at], tim[at + 1]]);
                        if word == 0 {
                            continue;
                        }
                        let dst = &mut expected[4 * (320 * y + x)..][..4];
                        for ch in 0..3 {
                            let shade = (((word >> (ch * 5)) & 31) * u16::from(rgb[ch]) / 128)
                                .min(31) as u8;
                            dst[ch] = shade * 8 + shade / 4;
                        }
                        dst[3] = 255;
                    }
                }
            } else {
                assert_eq!(
                    tag >> 24,
                    2,
                    "packet={packet:#x} tag={tag:#x} head={:#x}",
                    cpu.word(ot)
                );
            }
            packet = tag & 0xffffff;
        }
        expected
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn remaining_rivals_match_jp_sprite_chain() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let fonts = tm1_assets::screen::Fonts::load(&disc).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let tim = disc.read("TIM/CPFONT.TIM").unwrap();
        const OT: u32 = 0x80006000;
        // fontInit's metrics, sprite and draw-mode templates for CP font 2.
        cpu.put(0x80198c14, &0x80170bc4u32.to_le_bytes());
        cpu.put(0x80198b64 + 2 * 20 + 3, &[4]);
        cpu.put(0x80198b64 + 2 * 20 + 7, &[0x64]);
        cpu.put(0x80198b10 + 2 * 12 + 3, &[2]);
        cpu.put(0x8018c338, &8u16.to_le_bytes());
        let roster: Vec<_> = Vehicle::CARD_ORDER
            .into_iter()
            .chain([Vehicle::Minion])
            .collect();
        for &vehicle in &roster {
            let name = cpu.call(0x80150cb4, &[vehicle as u32]);
            let name = std::ffi::CStr::from_bytes_until_nul(&cpu.ram[(name & 0x1fffff) as usize..])
                .unwrap();
            assert_eq!(
                fonts.car_name(vehicle.card_index()).as_bytes(),
                name.to_bytes()
            );
        }
        for start in [0, 5] {
            let cars = &roster[start..start + 8];
            for (enabled, living) in [(true, 0xffu16), (true, 0xad), (true, 0), (false, 0xff)] {
                cpu.put(OT, &0xffffffu32.to_le_bytes());
                cpu.put(0x8018c5e8, &u32::from(enabled).to_le_bytes());
                for (slot, &car) in cars.iter().enumerate() {
                    let at = 0x8019f7a8 + slot as u32 * 0x560;
                    cpu.put(at + 0x3c, &(car as u32).to_le_bytes());
                    cpu.put(
                        at + 0x24c,
                        &(if (living >> slot) & 1 != 0 {
                            100i32
                        } else {
                            -1
                        })
                        .to_le_bytes(),
                    );
                }
                cpu.call(0x801461e4, &[OT]);
                let expected = draw_jp_font_packets(&cpu, OT, &tim);
                let mut actual = vec![0; expected.len()];
                draw_rival_list(
                    &mut actual,
                    &fonts,
                    cars.iter().copied(),
                    if enabled { living } else { 0 },
                );
                assert_eq!(
                    actual.iter().zip(&expected).position(|(a, b)| a != b),
                    None,
                    "start={start} mask={living:#x} enabled={enabled}"
                );
            }
        }
        // Centering measures the final line. A line feed resets X, while a CR keeps it.
        let text = "YELLOW\x08 JACKET\nA\rB";
        const STRING: u32 = 0x80006100;
        cpu.put(STRING, text.as_bytes());
        cpu.put(STRING + text.len() as u32, &[0]);
        let face = tm1_assets::screen::Face::Sign;
        assert_eq!(
            fonts.width(face, text),
            cpu.call(0x8010b03c, &[2, STRING]) as i32
        );
        cpu.call(0x8010bf60, &[2, 36, 150, 36]);
        cpu.put(OT, &0xffffffu32.to_le_bytes());
        cpu.call(
            0x8010b4f4,
            &[2, 146, 33, 0x80006300, 0x80006400, STRING, OT],
        );
        let expected = draw_jp_font_packets(&cpu, OT, &tim);
        let mut actual = vec![0; expected.len()];
        fonts.sprite_centered_at(&mut actual, face, 146, 33, text);
        assert!(actual == expected);
    }
}
