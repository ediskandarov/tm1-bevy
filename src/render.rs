//! Arena presentation in PlayStation ordering-table traversal order.
mod hierarchy;
use anyhow::{Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    ecs::system::SystemParam,
    image::ImageSampler,
    math::DVec3,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
};
use std::collections::BTreeMap;
use tm1_assets::{
    scene::{Scene, SpriteFrame, TextureKey, Wheel},
    texture::Vram,
};

use bevy::camera::visibility::RenderLayers;

#[derive(Clone, Copy)]
enum RegisteredModel {
    Backdrop,
    Car(usize),
    Merc(usize),
    Pedestrian(usize),
}

struct BodyGeometry {
    model: Scene,
    polygons: std::ops::Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CarPose {
    origin: Vec3,
    matrix: [[i16; 3]; 3],
    body_stage: usize,
    shadow_visible: bool,
}
impl CarPose {
    pub(crate) fn player(car: &tm1_core::drive::PlayerVehicle) -> Self {
        Self {
            origin: world(car.position().map(|v| v as f32)),
            matrix: car.rotation(),
            body_stage: car.body_stage(),
            shadow_visible: car.shadow_visible(),
        }
    }
    pub(crate) fn rival(car: &tm1_core::drive::Opponent) -> Self {
        Self {
            origin: world(car.position().map(|v| v as f32)),
            matrix: car.rotation(),
            body_stage: car.body_stage(),
            shadow_visible: car.shadow_visible(),
        }
    }
}

#[derive(Component)]
pub struct WorldCamera;

#[derive(Component)]
pub struct MirrorCamera;
#[derive(Component)]
pub struct MirrorDisplay;
const MIRROR_WIDTH: u32 = 101;
const MIRROR_HEIGHT: u32 = 30;
const MIRROR_FOCAL: f32 = 91.;
const BATTLE_FOCAL: f32 = 228.;

fn mirror_subview() -> bevy::camera::SubCameraView {
    let size = UVec2::new(MIRROR_WIDTH, MIRROR_HEIGHT);
    // The target is displayed flipped. Its optical axis must land at x=51
    // before reflection so the final 101-pixel view has the JP center x=50.
    bevy::camera::SubCameraView {
        full_size: UVec2::new(MIRROR_WIDTH.next_multiple_of(2), MIRROR_HEIGHT),
        size,
        offset: Vec2::ZERO,
    }
}
// VIEW scales the forward and vertical axes by 3690 and the horizontal axis by 4096.
const VIEW_SCALE_X: f32 = 3690. / 4096.;

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
    sky: Option<BodyGeometry>,
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
    registered_models: Vec<RegisteredModel>,
    body_ranges: Vec<[BodyGeometry; 3]>,
    pub(crate) has_animations: bool,
    animation_visits: hierarchy::Visits,
    view_start_visits: [hierarchy::Visits; 2],
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
        dmd: &tm1_assets::scene::Dmd,
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
        let mut registered = Vec::new();
        if scene.backdrop_start.is_some() {
            let root = dmd
                .roots
                .iter()
                .find(|r| r.1 == 3)
                .expect("extracted backdrop")
                .0;
            registered.push((root, RegisteredModel::Backdrop));
        }
        for (slot, stages) in std::iter::once(&vehicle).chain(&opponents).enumerate() {
            registered.push((stages[0].root_id, RegisteredModel::Car(slot)));
        }
        for (slot, frames) in merc_models.iter().enumerate() {
            if let Some(frame) = frames.first() {
                registered.push((frame.root_id, RegisteredModel::Merc(slot)));
            }
        }
        for (slot, frames) in pedestrians.iter().enumerate() {
            if let Some(frame) = frames.first() {
                registered.push((frame.root_id, RegisteredModel::Pedestrian(slot)));
            }
        }
        let mut registered_models = Vec::new();
        for root in dmd.battle_model_order()? {
            registered_models.extend(registered.iter().rev().filter(|r| r.0 == root).map(|r| r.1));
        }
        ensure!(
            registered_models.len() == registered.len(),
            "incomplete battle CS registration"
        );
        let has_animations = std::iter::once(&scene)
            .chain(&sky)
            .chain(&vehicle)
            .chain(opponents.iter().flatten())
            .chain(missile_models.iter().flatten())
            .chain(merc_models.iter().chain(&pedestrians).flatten())
            .any(|s| {
                s.hierarchy
                    .iter()
                    .any(|node| matches!(node.kind, tm1_assets::scene::HierarchyKind::Animation(_)))
            });
        let world_end = scene.polygons.len();
        let append = |scene: &mut Scene, mut model: Scene| {
            let start = scene.polygons.len();
            scene.polygons.append(&mut model.polygons);
            BodyGeometry {
                model,
                polygons: start..scene.polygons.len(),
            }
        };
        let sky = sky.map(|mut model| {
            // viewSetSky assigns the fixed bucket to the entire coordinate system.
            for polygon in &mut model.polygons {
                polygon.fixed_ot = 0xffe;
            }
            append(&mut scene, model)
        });
        let body_ranges = std::iter::once(vehicle)
            .chain(opponents)
            .map(|variants| variants.map(|model| append(&mut scene, model)))
            .collect();
        let mut tiles = BTreeMap::new();
        tiles.insert((None, false), 0);
        for p in scene
            .polygons
            .iter()
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
        for p in &scene.polygons[..world_end] {
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
            registered_models,
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
            has_animations,
            animation_visits: Default::default(),
            view_start_visits: Default::default(),
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

fn camera_pose(view: tm1_core::drive::SoundListener) -> (Vec3, Vec3, Vec3) {
    let (eye, forward, down) = view.camera_axes();
    let vector = |p: [i32; 3]| world(p.map(|v| v as f32));
    (
        vector(eye),
        vector(forward).normalize(),
        -vector(down).normalize(),
    )
}

pub fn setup(
    duel: Option<Res<crate::duel::Duel>>,
    mut commands: Commands,
    mut arena: ResMut<Arena>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut packet_materials: ResMut<Assets<crate::texture_blend::TextureBlend>>,
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
    let blend = packet_materials.add(crate::texture_blend::TextureBlend {
        atlas: texture,
        subtract: false,
        order: 0,
    });
    let mut mirror_image = Image::new_target_texture(
        MIRROR_WIDTH,
        MIRROR_HEIGHT,
        TextureFormat::Bgra8UnormSrgb,
        None,
    );
    mirror_image.sampler = ImageSampler::nearest();
    let mirror_target = images.add(mirror_image);
    commands.spawn((
        crate::campaign::StageEntity,
        WorldCamera,
        MirrorCamera,
        Camera2d,
        Camera {
            order: -2,
            sub_camera_view: duel.is_none().then(mirror_subview),
            is_active: false,
            ..default()
        },
        bevy::camera::RenderTarget::Image(mirror_target.clone().into()),
        Projection::Perspective(PerspectiveProjection {
            fov: 2. * (MIRROR_HEIGHT as f32 / (2. * MIRROR_FOCAL)).atan(),
            near: arena.radius * 0.005,
            far: arena.radius * 10.,
            ..default()
        }),
        RenderLayers::layer(3),
    ));
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
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 2. * (120. / BATTLE_FOCAL).atan(),
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
    mut cameras: Query<(&mut Transform, &mut Camera, Has<MirrorCamera>), With<WorldCamera>>,
    window: Single<&Window>,
    mut renderer: PacketRenderer,
) {
    let started = capture.demo.then(std::time::Instant::now);
    let direction = Vec3::new(
        arena.yaw.sin() * arena.pitch.cos(),
        arena.pitch.sin(),
        arena.yaw.cos() * arena.pitch.cos(),
    );
    let view = (!arena.overview).then_some(drive.listener);
    let (eye, forward, camera_up) = view.map(camera_pose).unwrap_or_else(|| {
        let eye = arena.center + direction * arena.radius * arena.zoom;
        (eye, (arena.center - eye).normalize(), Vec3::Y)
    });
    let mirror = drive.mirror && !arena.overview;
    let rear_view =
        mirror.then(|| tm1_core::drive::SoundListener::rear_view(&drive.player, &drive.camera));
    let rear_pose = rear_view.map(camera_pose);
    for (mut camera, mut settings, rear) in &mut cameras {
        if rear {
            settings.is_active = mirror;
            let Some((rear_eye, rear_forward, rear_up)) = rear_pose else {
                continue;
            };
            *camera = Transform::from_translation(rear_eye).looking_to(rear_forward, rear_up);
            camera.scale.x = VIEW_SCALE_X;
        } else {
            *camera = Transform::from_translation(eye).looking_to(forward, camera_up);
            if !arena.overview {
                camera.scale.x = VIEW_SCALE_X;
            }
        }
    }

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
        &drive.primary_cars,
        (drive.primary_hidden && !arena.overview).then_some(0),
        false,
        eye,
        forward,
        camera_up,
        view,
    );
    renderer.submit(&mut arena, false, packets);
    if let Some((rear_eye, rear_forward, rear_up)) = rear_pose {
        let poses: Vec<_> = std::iter::once(CarPose::player(&drive.player))
            .chain(drive.opponents.iter().map(CarPose::rival))
            .collect();
        let hide_player = !drive.camera.show_body;
        let packets = draw_view(
            &arena,
            &drive.player,
            &drive.contacts,
            true,
            &poses,
            hide_player.then_some(0),
            false,
            rear_eye,
            rear_forward,
            rear_up,
            rear_view,
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

type LightMatrix = [[i16; 3]; 3];
const WORLD_LIGHT: LightMatrix = [[617, -3494, 2046], [-3425, 1982, 1056], [3332, 1928, 1398]];

fn rotate_light(light: LightMatrix, rotation: LightMatrix) -> LightMatrix {
    light.map(|row| {
        rotation.map(|axis| {
            let dot = (0..3)
                .map(|i| i64::from(axis[i]) * i64::from(row[i]))
                .sum::<i64>();
            (dot >> 12).clamp(-32768, 32767) as i16
        })
    })
}

fn lit_rgb(rgb: [i32; 3], normal: [i16; 3], light: LightMatrix, env: u8) -> [i32; 3] {
    // NCCS clamps IR after both matrix products, then modulates the source RGB.
    let incidence = light.map(|row| {
        let dot = (0..3)
            .map(|i| i32::from(row[i]) * i32::from(normal[i]))
            .sum::<i32>();
        (dot >> 12).clamp(0, 32767)
    });
    let (ambient, colors) = match env {
        1 => (32, [[255; 3], [0; 3], [0; 3]]),
        2 => (32, [[60; 3], [140; 3], [170; 3]]),
        3 => (256, [[256, 0, 0]; 3]),
        _ => (32, [[200, 128, 60], [200, 75, 128], [200, 75, 60]]),
    };
    std::array::from_fn(|k| {
        let sum = (0..3).map(|i| colors[k][i] * incidence[i]).sum::<i32>();
        let intensity = (ambient * 16 + (sum >> 8)).clamp(0, 32767);
        (rgb[k] * intensity >> 12).min(255)
    })
}

fn packet_rgb([r, g, b]: [u8; 3]) -> [f32; 4] {
    [
        f32::from(r) / 255.,
        f32::from(g) / 255.,
        f32::from(b) / 255.,
        1.,
    ]
}

#[derive(Clone, Copy)]
enum Source<'a> {
    Scene { car: Option<usize>, sky: bool },
    Trail(&'a tm1_core::drive::ContrailModel),
    Actor(&'a Scene, [i32; 13]),
}

fn battle_hierarchy<'a>(
    arena: &'a Arena,
    player: &tm1_core::drive::PlayerVehicle,
    contacts: &tm1_core::drive::CollisionScene,
    secondary: bool,
    poses: &[CarPose],
    hidden_car: Option<usize>,
    split_screen: bool,
    root: Option<tm1_core::drive::HierarchyTransform>,
    eye: Vec3,
    trails: &'a [tm1_core::drive::ContrailModel],
    visits: hierarchy::Visits,
) -> hierarchy::Pass<Source<'a>> {
    let mirror = secondary && !split_screen;
    let actor_depth = if mirror { 1000 } else { 1200 };
    let sky_origin = poses[usize::from(split_screen && secondary)].origin;
    let wheel_steps = contacts.wheel_steps(secondary);
    let wheel_rotation = player.wheel_sin_cos(wheel_steps[4]);
    let hidden = contacts.hidden_pickups(secondary);
    let switches = contacts.arena_switch_branches(secondary);
    let (_, stand_branches) = contacts.health_stands(secondary);
    enum HierarchyItem<'a> {
        World,
        Backdrop,
        Car(usize),
        Trail(&'a tm1_core::drive::ContrailModel),
        Actor((&'a Scene, [i32; 13])),
    }
    let mut missiles_buf = [[0; 13]; 10];
    let count = contacts.missile_poses(secondary, &mut missiles_buf);
    let missiles = missiles_buf[..count].iter().rev().filter_map(|pose| {
        Some((
            arena.missile_models[usize::from(pose[0] == 312)].as_ref()?,
            *pose,
        ))
    });
    let registered = arena
        .registered_models
        .iter()
        .filter_map(|model| match *model {
            RegisteredModel::Backdrop => Some(HierarchyItem::Backdrop),
            RegisteredModel::Car(slot) => {
                (Some(slot) != hidden_car && slot < poses.len()).then_some(HierarchyItem::Car(slot))
            }
            RegisteredModel::Merc(slot) => {
                let [branch, x, y, z, _] = contacts.merc_pose(slot, secondary)?;
                Some(HierarchyItem::Actor((
                    arena.merc_models[slot].get(branch as usize)?,
                    [0, x, y, z, 4096, 0, 0, 0, 4096, 0, 0, 0, 4096],
                )))
            }
            RegisteredModel::Pedestrian(slot) => {
                let pose = contacts.pedestrian_pose(slot, secondary)?;
                if pose[13] == 0 {
                    return None;
                }
                Some(HierarchyItem::Actor((
                    arena.pedestrians[slot].get(pose[0] as usize)?,
                    pose[..13].try_into().unwrap(),
                )))
            }
        });
    // shellInitNewDatabase creates missile CSs, then contrail CSs after dbInit.
    // Each is prepended, so both pools run in descending slot order.
    let items = std::iter::once(HierarchyItem::World)
        .chain(
            trails
                .iter()
                .rev()
                .filter(|m| m.active())
                .map(HierarchyItem::Trail),
        )
        .chain(missiles.map(HierarchyItem::Actor))
        .chain(registered);
    let world_frame = root.unwrap_or(tm1_core::drive::HierarchyTransform {
        rows: [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
        normals: [[0; 3]; 6],
        position: [-eye.x as i32, eye.z as i32, -eye.y as i32],
    });
    let state = hierarchy::BranchState {
        wheels: wheel_steps,
        rotation: wheel_rotation,
        switches,
        stands: stand_branches,
        pickups: hidden,
        damage: &|address| contacts.damage_branch(address, secondary),
    };
    let mut pass = hierarchy::Pass::new(world_frame, root.is_some());
    pass.visits = visits;
    for item in items {
        match item {
            HierarchyItem::Car(car) => {
                let pose = &poses[car];
                let body = &arena.body_ranges[car][pose.body_stage];
                let position = [
                    pose.origin.x as i32,
                    -pose.origin.z as i32,
                    pose.origin.y as i32,
                ];
                let slot = pass.coordinate_system(
                    world_frame.child(position, Some(pose.matrix)),
                    Some(pose.matrix),
                );
                pass.traverse(
                    Source::Scene {
                        car: Some(car),
                        sky: false,
                    },
                    &body.model,
                    0,
                    body.polygons.start,
                    slot,
                    actor_depth,
                    pose.shadow_visible,
                    &state,
                );
            }
            HierarchyItem::World => {
                let depth = if mirror { 100 } else { 1200 };
                pass.traverse(
                    Source::Scene {
                        car: None,
                        sky: false,
                    },
                    &arena.scene,
                    0,
                    0,
                    0,
                    depth,
                    true,
                    &state,
                );
                if !mirror && let Some(sky) = &arena.sky {
                    let slot = pass.coordinate_system(
                        world_frame.child([sky_origin.x as i32, -sky_origin.z as i32, 0], None),
                        None,
                    );
                    pass.traverse(
                        Source::Scene {
                            car: None,
                            sky: true,
                        },
                        &sky.model,
                        0,
                        sky.polygons.start,
                        slot,
                        depth,
                        true,
                        &state,
                    );
                }
            }
            HierarchyItem::Backdrop => {
                let slot = pass.coordinate_system(world_frame, None);
                pass.traverse(
                    Source::Scene {
                        car: None,
                        sky: false,
                    },
                    &arena.scene,
                    arena.scene.hierarchy[0].end,
                    0,
                    slot,
                    actor_depth,
                    true,
                    &state,
                );
            }
            HierarchyItem::Trail(trail) => {
                let frame = world_frame.child(trail.origin, None);
                let slot = pass.coordinate_system(frame, None);
                pass.object(
                    Source::Trail(trail),
                    0..trail.faces().len(),
                    slot,
                    frame.position,
                );
            }
            HierarchyItem::Actor((model, pose)) => {
                let rotation =
                    std::array::from_fn(|j| std::array::from_fn(|k| pose[4 + 3 * j + k] as i16));
                let slot = pass.coordinate_system(
                    world_frame.child([pose[1], pose[2], pose[3]], Some(rotation)),
                    Some(rotation),
                );
                pass.traverse(
                    Source::Actor(model, pose),
                    model,
                    0,
                    0,
                    slot,
                    actor_depth,
                    true,
                    &state,
                );
            }
        }
    }
    pass
}

impl Arena {
    pub(crate) fn reset_animation(&mut self) {
        self.animation_visits.clear();
        self.view_start_visits = Default::default();
    }

    pub(crate) fn advance_view_animation(
        &mut self,
        player: &tm1_core::drive::PlayerVehicle,
        contacts: &tm1_core::drive::CollisionScene,
        secondary: bool,
        poses: &[CarPose],
        hidden_car: Option<usize>,
        split_screen: bool,
        view: tm1_core::drive::SoundListener,
    ) {
        self.view_start_visits[usize::from(secondary)].clone_from(&self.animation_visits);
        let visits = std::mem::take(&mut self.animation_visits);
        let trails = contacts.contrails(secondary);
        let mut pass = battle_hierarchy(
            self,
            player,
            contacts,
            secondary,
            poses,
            hidden_car,
            split_screen,
            Some(view.hierarchy_transform(split_screen || secondary)),
            Vec3::ZERO,
            &trails,
            visits,
        );
        self.animation_visits = std::mem::take(&mut pass.visits);
    }
}

fn draw_view(
    arena: &Arena,
    player: &tm1_core::drive::PlayerVehicle,
    contacts: &tm1_core::drive::CollisionScene,
    secondary: bool,
    poses: &[CarPose],
    hidden_car: Option<usize>,
    split_screen: bool,
    eye: Vec3,
    forward: Vec3,
    camera_up: Vec3,
    view: Option<tm1_core::drive::SoundListener>,
) -> Batch {
    let axis_scale = if arena.overview { 1. } else { VIEW_SCALE_X };
    let root = view.map(|v| v.hierarchy_transform(split_screen || secondary));
    let mirror = secondary && !split_screen;
    let depth = if let Some(root) = root {
        GeometryDepth::from_world_matrix(root.rows[2], root.translation()[2])
    } else {
        let axis = (forward * axis_scale).as_dvec3();
        GeometryDepth {
            axis,
            translation: -eye.as_dvec3().dot(axis),
        }
    };
    let wheel_steps = contacts.wheel_steps(secondary);
    let lights = contacts.car_lights(secondary);
    let wheel_rotation = player.wheel_sin_cos(wheel_steps[4]);
    let [sin, cos] = wheel_rotation.map(|n| f32::from(n) / 4096.);
    let wheel_vertex = |p, wheel| {
        if let Some(Wheel::Hammerhead { pivot, axis }) = wheel {
            let axis = Vec3::from_array(axis);
            let pivot = Vec3::from_array(pivot);
            let offset = Vec3::from_array(p) - pivot;
            (pivot + offset * cos + axis.cross(offset) * sin + axis * axis.dot(offset) * (1. - cos))
                .to_array()
        } else {
            p
        }
    };
    let right = forward.cross(camera_up).normalize();
    let up = right.cross(forward);
    let near = arena.radius * 0.005;
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
    let sky_origin = poses[usize::from(split_screen && secondary)].origin * Vec3::new(1., 0., 1.);
    let (lens, center) = if split_screen {
        (BATTLE_FOCAL as i32, [160, 60])
    } else if secondary {
        (
            MIRROR_FOCAL as i32,
            [MIRROR_WIDTH as i32 / 2, MIRROR_HEIGHT as i32 / 2],
        )
    } else {
        (BATTLE_FOCAL as i32, [160, 120])
    };
    let bucket_depth = |bucket: u32| (bucket as f32 * 8. / axis_scale).max(near + 0.01);
    // Mirror packets already contain reflected X. Its display sprite flips the render target.
    let pixel_right = if secondary && !split_screen {
        -right
    } else {
        right
    };
    let vertex = |[x, y]: [i32; 2], depth: f32| {
        let offset = pixel_right * (x as f32 * axis_scale) - up * y as f32;
        eye + forward * depth + offset * (depth / lens as f32)
    };
    let face_points = |p: &tm1_assets::scene::Polygon,
                       node: Option<GeometryTransform>,
                       origin: &dyn Fn([f32; 3]) -> Vec3| {
        let mut points = [Vec3::ZERO; 4];
        let mut depths = [0; 4];
        let mut screen = [[0; 2]; 4];

        for (i, v) in p.vertices.iter().enumerate() {
            if let Some(transform) = node {
                let (mut pixel, z) = transform.project(v.local, lens, center);
                depths[i] = z;
                if let Some(extent) = p.billboard {
                    let [w, h] = sprite_half_size(extent, lens, z);
                    pixel[0] += if i & 1 == 0 { -w } else { w };
                    pixel[1] += if i < 2 { -h } else { h };
                }
                screen[i] = pixel;
                points[i] = vertex(pixel, bucket_depth(z >> 3));
            } else {
                let pos = origin(v.position);
                depths[i] = depth.vertex(pos.to_array());
                points[i] = if let Some([w, h]) = p.billboard {
                    pos + right * f32::from(w) * if i & 1 == 0 { -1. } else { 1. }
                        + up * f32::from(h) * if i < 2 { 1. } else { -1. }
                } else {
                    pos
                };
            }
        }
        let visible = p.double_sided
            || p.billboard.is_some()
            || if node.is_some() {
                packet_front_facing(
                    [screen[0], screen[1], screen[2]],
                    secondary && !split_screen,
                )
            } else {
                !back_facing([points[0], points[1], points[2]], eye)
            };
        visible.then_some((points, depths, screen))
    };
    let mut subdivision_count = 0;
    let mut submit_face = |batch: &mut Batch,
                           p: &tm1_assets::scene::Polygon,
                           points: [Vec3; 4],
                           depths: [u32; 4],
                           screen: [[i32; 2]; 4],
                           slot: usize,
                           light: LightMatrix,
                           env: u8| {
        let n = p.vertices.len();
        let Some((raw, bucket)) = hierarchy_bucket(n, depths, p.fixed_ot) else {
            return;
        };
        let mode = PacketMode::polygon(p);
        let scale = if p.texture.is_some() { 128. } else { 255. };
        let corners: [PacketCorner; 4] = std::array::from_fn(|i| {
            let v = &p.vertices[i.min(n - 1)];
            let base = std::array::from_fn(|k| (v.color[k] * scale).round() as i32);
            PacketCorner {
                xy: screen[i],
                uv: v.uv.map(|u| u as i32),
                rgb: v
                    .normal
                    .map_or(base, |normal| lit_rgb(base, normal, light, env)),
            }
        });
        let subdivide =
            view.is_some() && queue_subdivision(&mut subdivision_count, p.subdivide_near, raw);
        let mut emit = |quad: [PacketCorner; 4], pass: DrawPass, child: bool| {
            let start = batch.positions.len();
            for tri in indices(n).chunks_exact(3) {
                if view.is_some()
                    && !triangle_fits_gpu([quad[tri[0]].xy, quad[tri[1]].xy, quad[tri[2]].xy])
                {
                    continue;
                }
                for &i in tri {
                    let c = quad[i];
                    let pos = if child {
                        vertex(c.xy, bucket_depth(raw))
                    } else {
                        points[i]
                    };
                    let uv = arena.uv(slot, Vec2::new(c.uv[0] as f32, c.uv[1] as f32));
                    let rgb = c.rgb.map(|k| k as f32 / scale);
                    batch.push(pos, uv, [rgb[0], rgb[1], rgb[2], 1.]);
                }
            }
            let count = batch.positions.len() - start;
            if count != 0 {
                batch.order_packet(count, bucket, pass, mode);
            }
        };
        if subdivide {
            if p.billboard.is_none() {
                let pass = if n == 3 {
                    DrawPass::SubTriangles
                } else {
                    DrawPass::SubQuads
                };
                for child in subdivided_faces(corners, n) {
                    emit(child, pass, true);
                }
            }
        } else {
            emit(corners, DrawPass::Hierarchy, false);
        }
    };
    let trails = contacts.contrails(secondary);
    let pass = battle_hierarchy(
        arena,
        player,
        contacts,
        secondary,
        poses,
        hidden_car,
        split_screen,
        root,
        eye,
        &trails,
        arena.view_start_visits[usize::from(secondary)].clone(),
    );
    for object in &pass.objects {
        let node_depth = root.map(|_| pass.frame(object.slot, object.position));
        match object.source {
            Source::Scene { car, sky } => {
                for i in object.polygons.clone() {
                    let p = &arena.scene.polygons[i];
                    let slot = arena.tiles[&(p.texture, p.semi_transparent)];
                    let Some((points, depths, screen)) = face_points(p, node_depth, &|position| {
                        let local = wheel_vertex(position, p.wheel);
                        if let Some(slot) = car {
                            let pose = &poses[slot];
                            pose.origin
                                + world(std::array::from_fn(|k| {
                                    (0..3)
                                        .map(|j| pose.matrix[j][k] as f32 * local[j] / 4096.)
                                        .sum()
                                }))
                        } else {
                            world(local) + if sky { sky_origin } else { Vec3::ZERO }
                        }
                    }) else {
                        continue;
                    };
                    let env = car.map_or(0, |slot| lights[slot]);
                    submit_face(
                        &mut batch,
                        p,
                        points,
                        depths,
                        screen,
                        slot,
                        pass.lights[object.slot],
                        env,
                    );
                }
            }
            Source::Trail(trail) => {
                let transform = node_depth;
                let tint = [16, 8, 0].map(|shift| ((trail.color >> shift) & 255) as f32 / 128.);
                for face in trail.faces() {
                    let frame = arena.contrail_frames[face.frame as usize];
                    let slot = arena.tiles[&(Some(frame.texture), true)];
                    let mut points = [Vec3::ZERO; 4];
                    let mut screen = [[0; 2]; 4];
                    let mut depths = [0; 4];
                    for (i, local) in face.vertices.into_iter().enumerate() {
                        if let Some(transform) = transform {
                            let (pixel, z) = transform.project(local, lens, center);
                            screen[i] = pixel;
                            depths[i] = z;
                            points[i] = vertex(screen[i], bucket_depth(z >> 3));
                        } else {
                            points[i] = world(std::array::from_fn(|k| {
                                trail.origin[k].wrapping_add(i32::from(local[k])) as f32
                            }));
                            depths[i] = depth.vertex(points[i].to_array());
                        }
                    }
                    let front = if transform.is_some() {
                        packet_front_facing([screen[0], screen[1], screen[2]], mirror)
                    } else {
                        !back_facing([points[0], points[1], points[2]], eye)
                    };
                    let Some((_, bucket)) = hierarchy_bucket(4, depths, 0) else {
                        continue;
                    };
                    if !front {
                        continue;
                    }
                    let start = batch.positions.len();
                    for tri in indices(4).chunks_exact(3) {
                        if transform.is_some()
                            && !triangle_fits_gpu([screen[tri[0]], screen[tri[1]], screen[tri[2]]])
                        {
                            continue;
                        }
                        for &i in tri {
                            let [u, v] = frame.uv[i ^ 1];
                            batch.push(
                                points[i],
                                arena.uv(slot, Vec2::new(u.into(), v.into())),
                                [tint[0], tint[1], tint[2], 1.],
                            );
                        }
                    }
                    let count = batch.positions.len() - start;
                    if count > 0 {
                        batch.order_packet(
                            count,
                            bucket,
                            DrawPass::Hierarchy,
                            PacketMode::texture(frame.texture),
                        );
                    }
                }
            }
            Source::Actor(model, pose) => {
                let origin = world([pose[1] as f32, pose[2] as f32, pose[3] as f32]);
                for p in &model.polygons[object.polygons.clone()] {
                    let Some((points, depths, screen)) = face_points(p, node_depth, &|position| {
                        origin
                            + world(std::array::from_fn(|k| {
                                (0..3)
                                    .map(|j| pose[4 + 3 * j + k] as f32 * position[j] / 4096.)
                                    .sum()
                            }))
                    }) else {
                        continue;
                    };
                    let slot = arena.tiles[&(p.texture, p.semi_transparent)];
                    submit_face(
                        &mut batch,
                        p,
                        points,
                        depths,
                        screen,
                        slot,
                        pass.lights[object.slot],
                        0,
                    );
                }
            }
        }
    }
    let coords = |p: [i32; 3]| {
        if let Some(view) = view {
            return view.effect_coordinates(p);
        }
        let d = world(p.map(|v| v as f32)) - eye;
        [
            d.dot(right) as i32,
            -d.dot(up) as i32,
            d.dot(forward) as i32,
        ]
    };
    let mut drops = [[0; 4]; 12];
    let drop_count = contacts.drawn_drops(secondary, &mut drops);
    for &[x, y, z, weapon] in &drops[..drop_count] {
        let camera = coords([x, y, z]);
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
                let dx = if i & 1 == 0 { -48 } else { 48 };
                let dy = if i < 2 { -48 } else { 48 };
                tm1_core::drive::drop_screen_point(coords([x + dx, y + dy, z]), [lens; 2])
            } else {
                billboard[i]
            };
            batch.push(
                vertex(xy, depth),
                arena.uv(tile, Vec2::from(sprite.uv[i].map(f32::from))),
                [1.; 4],
            );
        }
        batch.order_packet(6, bucket, DrawPass::Bullets, PacketMode::Opaque);
    }
    let white = arena.uv(0, Vec2::ZERO);
    let draw_line = |batch: &mut Batch, points, colors, bucket| {
        let start = batch.positions.len();
        let depth = bucket_depth(bucket);
        line_pixels(points, colors, center, |[x, y], rgb| {
            let color = packet_rgb(rgb);
            for &corner in indices(4) {
                let u = x + (corner & 1) as i32;
                let v = y + (corner >> 1) as i32;
                batch.push(vertex([u, v], depth), white, color);
            }
        });
        let count = batch.positions.len() - start;
        if count != 0 {
            batch.order_packet(
                count,
                bucket,
                DrawPass::Bullets,
                PacketMode::UntexturedOpaque,
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
            batch.push(
                vertex(corners[i], depth),
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
        let coords = coords([x, y, z]);
        let Some((bucket, corners)) = tm1_core::drive::project_fragment(coords, [lens; 2], center)
        else {
            continue;
        };
        let sprite = &arena.fragment_frames[variant as usize][phase as usize];
        let tile = arena.tiles[&(Some(sprite.texture), false)];
        let draw_depth = bucket_depth(bucket);
        for &i in indices(4) {
            batch.push(
                vertex(corners[i], draw_depth),
                arena.uv(tile, Vec2::from(sprite.uv[i].map(f32::from))),
                [1.; 4],
            );
        }
        batch.order_packet(6, bucket, DrawPass::Explosions, PacketMode::Opaque);
    }
    for arc in &arcs[..arc_count] {
        let points = arc.map(|p| {
            let coords = coords(p);
            let (bucket, [x, y]) = tm1_core::drive::project_taser_point(coords, [lens; 2], center)?;
            Some(([x, y], bucket))
        });
        for (j, segment) in points.windows(2).enumerate() {
            let (Some((a, _)), Some((b, bucket))) = (segment[0], segment[1]) else {
                continue;
            };
            let colors = [j, j + 1].map(|i| {
                let level = (250 - 12 * i) as u8;
                [level, level, 255]
            });
            draw_line(&mut batch, [a, b], colors, bucket);
        }
    }
    for &[x, y, z, px, py, pz, kind, life] in &bullets[..bullet_count] {
        let Some((bucket, primitive)) = tm1_core::bullet::project(
            kind,
            life,
            coords([x, y, z]),
            coords([px, py, pz]),
            [lens; 2],
            center,
        ) else {
            continue;
        };
        let depth = bucket_depth(bucket);
        use tm1_core::bullet::Primitive;
        match primitive {
            Primitive::Line { points, colors } => {
                draw_line(&mut batch, points, colors, bucket);
            }
            Primitive::Triangle { points, colors } => {
                for (point, color) in points.into_iter().zip(colors) {
                    batch.push(vertex(point, depth), white, packet_rgb(color));
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
                        vertex(point, depth),
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
    fn quad(&mut self, [x, y, w, h]: [i32; 4], uv: [[f32; 2]; 4], color: [f32; 4]) {
        for &i in indices(4) {
            let px = x + if i % 2 == 1 { w } else { 0 };
            let py = y + if i >= 2 { h } else { 0 };
            let pos = Vec2::new(px as f32 / 320. - 0.5, 0.5 - py as f32 / 240.) * self.size;
            self.batch.push(pos.extend(0.), uv[i], color);
        }
    }
    fn bar(&mut self, rect: [i32; 4], colors: [[u8; 3]; 2]) {
        let [x, y, width, height] = rect;
        let uv = [self.arena.uv(0, Vec2::ZERO); 4];
        for (offset, length, rgb) in bar_spans(width, colors) {
            let [r, g, b, _] = packet_rgb(rgb);
            let color = [r, g, b, PacketMode::UntexturedAverage as u8 as f32];
            self.quad([x + offset, y, length, height], uv, color);
        }
    }
    fn sprite(&mut self, x: i32, y: i32, f: SpriteFrame, tint: [u8; 3], semi: bool) {
        let tint = tint.map(|v| v as f32 / 128.);
        let mode = if semi {
            PacketMode::texture(f.texture)
        } else {
            PacketMode::Opaque
        };
        self.arena.sprite_rects(f, 0, semi, |[dx, dy, w, h], uv| {
            self.quad(
                [x + dx, y + dy, w, h],
                uv,
                [tint[0], tint[1], tint[2], mode as u8 as f32],
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
// Interpolate command RGB across logical framebuffer pixels before RGB555 truncation.
fn bar_spans(width: i32, [left, right]: [[u8; 3]; 2]) -> impl Iterator<Item = (i32, i32, [u8; 3])> {
    let step: [i32; 3] =
        std::array::from_fn(|i| ((i32::from(right[i]) - i32::from(left[i])) << 12) / width.max(1));
    let color_at = move |x: i32| {
        std::array::from_fn(|i| {
            let channel = (i32::from(left[i]) << 12) + 2048 + step[i] * x;
            (channel >> 12) as u8 & 0xf8
        })
    };
    let mut start = 0;
    std::iter::from_fn(move || {
        if start >= width {
            return None;
        }
        let color = color_at(start);
        let mut end = if left == right { width } else { start + 1 };
        while end < width && color_at(end) == color {
            end += 1;
        }
        let span = (start, end - start, color);
        start = end;
        Some(span)
    })
}

fn rival_health_width(hp: i32, maximum: i32) -> i32 {
    (hp.wrapping_mul(86) / maximum.max(1)).clamp(1, 86)
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
            [0., 0., 0., PacketMode::UntexturedOpaque as u8 as f32],
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
            [103, 22, rival_health_width(hp, max), 13],
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
fn packet_front_facing([a, b, c]: [[i32; 2]; 3], mirror: bool) -> bool {
    let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    (area < 0) == mirror
}

fn back_facing([a, b, c]: [Vec3; 3], eye: Vec3) -> bool {
    (b - a).cross(c - a).dot(eye - a) > 0.
}

// A LINE_G2 includes both endpoints, even when both project onto the same pixel.
fn line_pixels(
    mut points: [[i32; 2]; 2],
    mut colors: [[u8; 3]; 2],
    center: [i32; 2],
    mut emit: impl FnMut([i32; 2], [u8; 3]),
) {
    // GPU command coordinates are signed eleven-bit framebuffer coordinates.
    for point in &mut points {
        for axis in 0..2 {
            point[axis] = ((point[axis] + center[axis]) << 21 >> 21) - center[axis];
        }
    }
    let width = points[0][0].abs_diff(points[1][0]);
    let height = points[0][1].abs_diff(points[1][1]);
    if width >= 1024 || height >= 512 {
        return;
    }
    let steps = width.max(height) as i32;
    if steps > 0 && points[0][0] >= points[1][0] {
        points.swap(0, 1);
        colors.swap(0, 1);
    }
    let divisor = steps.max(1);
    let color_step: [i32; 3] = std::array::from_fn(|c| {
        ((i32::from(colors[1][c]) - i32::from(colors[0][c])) << 12) / divisor
    });
    for i in 0..=steps {
        let point = std::array::from_fn(|axis| {
            let delta = points[1][axis] - points[0][axis];
            // X ties round down. Y ties follow the direction of travel.
            let bias = divisor - i32::from(axis == 0 || delta < 0);
            points[0][axis] + (2 * i * delta + bias).div_euclid(2 * divisor)
        });
        let color = std::array::from_fn(|c| {
            (i32::from(colors[0][c]) + ((2048 + i * color_step[c]) >> 12)) as u8
        });
        emit(point, color);
    }
}

// RTPS uses a normalized reciprocal with one refinement and a 17-bit quotient.
fn gte_screen_offset(xy: [i32; 2], depth: u32, lens: i32, center: [i32; 2]) -> [i32; 2] {
    let quotient = if lens as u32 >= 2 * depth {
        0x1ffff
    } else {
        let shift = (depth as u16).leading_zeros();
        let divisor = (depth << shift) as i32;
        let index = (divisor - 0x7fc0) >> 7;
        let seed = ((0x40000 / (index + 0x100) + 1) / 2).clamp(0x101, 0x200);
        let correction = (0x80 - divisor * seed) >> 8;
        let reciprocal = (seed * (0x20000 + correction) + 0x80) >> 8;
        (((i64::from(lens) << shift) * i64::from(reciprocal) + 0x8000) >> 16).min(0x1ffff)
    };
    std::array::from_fn(|axis| {
        let ir = xy[axis].clamp(-32768, 32767);
        let pixel = ((i64::from(ir) * quotient) >> 16) as i32 + center[axis];
        pixel.clamp(-1024, 1023) - center[axis]
    })
}

// geomSprite divides signed DMD extents by SZ and clamps each half-size independently.
fn sprite_half_size(extent: [i16; 2], lens: i32, depth: u32) -> [i32; 2] {
    extent.map(|v| (i32::from(v) * lens / depth.max(1) as i32).clamp(1, 500))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PacketCorner {
    xy: [i32; 2],
    uv: [i32; 2],
    rgb: [i32; 3],
}

impl PacketCorner {
    fn midpoint(self, b: Self) -> Self {
        fn avg<const N: usize>(a: [i32; N], b: [i32; N]) -> [i32; N] {
            std::array::from_fn(|i| (a[i] + b[i]) >> 1)
        }
        Self {
            xy: avg(self.xy, b.xy),
            uv: avg(self.uv, b.uv),
            rgb: avg(self.rgb, b.rgb),
        }
    }
}

fn queue_subdivision(count: &mut usize, marked: bool, bucket: u32) -> bool {
    if marked && bucket < 100 && *count < 200 {
        *count += 1;
        true
    } else {
        false
    }
}

fn subdivided_faces([a, b, c, d]: [PacketCorner; 4], count: usize) -> [[PacketCorner; 4]; 4] {
    let ab = a.midpoint(b);
    let ac = a.midpoint(c);
    if count == 3 {
        let bc = b.midpoint(c);
        [
            [a, ac, ab, ab],
            [bc, b, ab, ab],
            [bc, ac, c, c],
            [bc, ac, ab, ab],
        ]
    } else {
        let bd = b.midpoint(d);
        let cd = c.midpoint(d);
        let mid = ac.midpoint(bd);
        [
            [a, ab, ac, mid],
            [ac, mid, c, cd],
            [ab, b, mid, bd],
            [mid, bd, cd, d],
        ]
    }
}

fn triangle_fits_gpu(p: [[i32; 2]; 3]) -> bool {
    [1023, 511].into_iter().enumerate().all(|(axis, max)| {
        let [a, b, c] = p.map(|q| q[axis]);
        a.max(b).max(c) - a.min(b).min(c) <= max
    })
}

fn hierarchy_bucket(n: usize, depths: [u32; 4], fixed: u32) -> Option<(u32, u32)> {
    let raw = face_bucket(n, depths);
    (2..=4095)
        .contains(&raw)
        .then_some((raw, if fixed == 0 { raw } else { fixed }))
}

fn face_bucket(n: usize, depths: [u32; 4]) -> u32 {
    let far = depths[0].max(depths[2]);
    (if n == 4 {
        far.max(depths[1]).max(depths[3])
    } else {
        far
    }) >> 3
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
    SubTriangles,
    SubQuads,
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
#[derive(Clone, Copy)]
struct GeometryDepth {
    axis: DVec3,
    translation: f64,
}

impl GeometryDepth {
    fn from_world_matrix(row: [i32; 3], translation: i32) -> Self {
        Self {
            axis: world(row.map(|v| v as f32)).as_dvec3() / 4096.,
            translation: f64::from(translation),
        }
    }

    fn vertex(self, point: [f32; 3]) -> u32 {
        let z = self.axis.dot(Vec3::from_array(point).as_dvec3()) + self.translation;
        z.clamp(0., 65535.) as u32
    }
}

#[derive(Clone, Copy)]
struct GeometryTransform {
    position: [i32; 3],
    normals: [[i32; 3]; 6],
    rows: [[i32; 3]; 3],
    translation: [i32; 3],
}

impl From<tm1_core::drive::HierarchyTransform> for GeometryTransform {
    fn from(frame: tm1_core::drive::HierarchyTransform) -> Self {
        Self {
            normals: frame.normals,
            position: frame.position,
            rows: frame.rows,
            translation: frame.translation(),
        }
    }
}

impl GeometryTransform {
    #[cfg(test)]
    fn hierarchy(
        root: tm1_core::drive::HierarchyTransform,
        nodes: &[tm1_assets::scene::NodeTransform],
        [sin, cos]: [i16; 2],
    ) -> Vec<Self> {
        let mut frames = Vec::with_capacity(nodes.len() + 1);
        frames.push(root);
        for node in nodes {
            let rotation = if node.truck_wheel {
                Some([[4096, 0, 0], [0, cos, sin], [0, -sin, cos]])
            } else {
                node.rotation
            };
            frames.push(frames[node.parent].child(node.offset, rotation));
        }
        frames.into_iter().map(Self::from).collect()
    }

    fn contains(&self, bound: &tm1_assets::scene::HierarchyBound, depth: i32) -> bool {
        let point: [i32; 3] =
            std::array::from_fn(|i| self.position[i].wrapping_add(bound.center[i]));
        self.normals.iter().enumerate().all(|(i, normal)| {
            let distance = normal
                .iter()
                .zip(point)
                .fold(0i32, |sum, (a, b)| sum.wrapping_add(a.wrapping_mul(b)))
                / 4096;
            let limit = if i == 5 { -8 * depth } else { 0 };
            distance >= limit.wrapping_sub(bound.radius)
        })
    }

    fn coordinate(&self, axis: usize, point: [i16; 3]) -> i32 {
        let product: i64 = self.rows[axis]
            .into_iter()
            .zip(point)
            .map(|(a, b)| i64::from(a) * i64::from(b))
            .sum();
        self.translation[axis].wrapping_add((product >> 12) as i32)
    }
    fn project(&self, point: [i16; 3], lens: i32, center: [i32; 2]) -> ([i32; 2], u32) {
        let depth = self.depth(point);
        let xy = [0, 1].map(|axis| self.coordinate(axis, point));
        (gte_screen_offset(xy, depth, lens, center), depth)
    }
    fn depth(&self, point: [i16; 3]) -> u32 {
        self.coordinate(2, point).clamp(0, 65535) as u32
    }
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
            Has<MirrorCamera>,
        ),
        With<WorldCamera>,
    >,
) {
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
        [0., 0., 0., PacketMode::UntexturedOpaque as u8 as f32],
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
    let current_cars = duel.cars.each_ref().map(CarPose::player);
    for slot in 0..2 {
        let car = &duel.cars[slot];
        let view = &duel.cameras[slot];
        let (transform, poses, hidden) = if slot == 0 {
            (duel.primary_view, &duel.primary_cars, duel.primary_hidden)
        } else {
            (
                tm1_core::drive::SoundListener::new(car, view, None),
                &current_cars,
                !view.show_body,
            )
        };
        let (eye, forward, up) = camera_pose(transform);
        let top = if slot == 0 { 0 } else { height / 2 };
        let h = if slot == 0 { height / 2 } else { height - top };
        let focal = window.height() * BATTLE_FOCAL / 240.;
        for (mut transform, mut cam, mut projection, second) in &mut cameras {
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
            transform.scale.x = VIEW_SCALE_X;
        }
        let packets = draw_view(
            &arena,
            car,
            &duel.scene,
            slot != 0,
            poses,
            hidden.then_some(slot),
            true,
            eye,
            forward,
            up,
            Some(transform),
        );
        renderer.submit(&mut arena, slot != 0, packets);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn campaign_animation_clocks_follow_simulation_steps_and_restart() {
        use crate::campaign::{Entry, Stage};
        use std::time::Duration;
        use tm1_core::drive::{Level, Vehicle};
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut app = App::new();
        app.init_resource::<Time<Real>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(crate::smoke::Capture::new(None, false, false))
            .add_systems(Update, crate::driving::advance);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        Stage::load(
            &disc,
            Level::Arena,
            Vehicle::Outlaw,
            Entry::New(Default::default()),
        )
        .unwrap()
        .install(app.world_mut());
        let tick = |app: &mut App, ms| {
            app.world_mut()
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_millis(ms));
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        };
        tick(&mut app, 1);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyB);
        tick(&mut app, 200);
        assert!(app.world().resource::<crate::driving::Drive>().mirror);
        let snapshot = |app: &App| {
            let arena = app.world().resource::<Arena>();
            (
                arena.animation_visits.clone(),
                arena.view_start_visits.clone(),
            )
        };
        let after_four = snapshot(&app);
        assert!(!after_four.0.is_empty());
        tick(&mut app, 0);
        assert_eq!(after_four, snapshot(&app));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        tick(&mut app, 50);
        assert!(snapshot(&app).0.is_empty());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        if !app.world().resource::<crate::driving::Drive>().mirror {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyB);
        }
        for _ in 0..4 {
            tick(&mut app, 50);
        }
        assert_eq!(after_four, snapshot(&app));
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_projection_matches_jp_view_table() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        for (entry, focal, width, height) in [
            (6, BATTLE_FOCAL, 320, 240),
            (11, BATTLE_FOCAL, 320, 120),
            (12, BATTLE_FOCAL, 320, 120),
            (13, MIRROR_FOCAL, MIRROR_WIDTH, MIRROR_HEIGHT),
        ] {
            let at = 0x80170270 + 96 * entry;
            assert_eq!([cpu.word(at + 8), cpu.word(at + 12)], [focal as u32; 2]);
            assert_eq!([cpu.word(at + 24), cpu.word(at + 28)], [width, height]);
            let axes = cpu.word(at + 40);
            let horizontal = (axes as i16).unsigned_abs();
            assert_eq!(VIEW_SCALE_X, (axes >> 16) as f32 / f32::from(horizontal));
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn reflected_mirror_projection_matches_jp_pixel_edges() {
        use bevy::camera::CameraProjection;
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(0x8018c520, &13u32.to_le_bytes());
        cpu.call(0x8010aed4, &[1, 0x80007000, 0x80007004]);
        let cx = cpu.word(0x80007000) as f32;
        let cy = cpu.word(0x80007004) as f32;
        let size = mirror_subview().size.as_vec2();
        let proj = PerspectiveProjection {
            fov: 2. * (size.y / (2. * MIRROR_FOCAL)).atan(),
            ..default()
        }
        .get_clip_from_view_for_sub(&mirror_subview());
        for depth in [128., 4096.] {
            for x in 0..=MIRROR_WIDTH {
                for y in [0., cy, size.y] {
                    let px = x as f32;
                    let view = Vec3::new(cx - px, cy - y, -MIRROR_FOCAL) * (depth / MIRROR_FOCAL);
                    let ndc = proj.project_point3(view);
                    let displayed = (Vec2::ONE - ndc.truncate()) * 0.5 * size;
                    assert!(
                        displayed.abs_diff_eq(Vec2::new(px, y), 0.001),
                        "{displayed:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn gouraud_line_coverage_and_color() {
        let pixels = |points, colors| {
            let mut result = Vec::new();
            super::line_pixels(points, colors, [160, 120], |p, c| result.push((p, c)));
            result
        };
        let colors = [[24; 3], [223; 3]];
        assert_eq!(pixels([[1, 2]; 2], colors), [([1, 2], colors[0])]);
        for (end, middle) in [([2, 1], [1, 1]), ([2, -1], [1, -1]), ([1, 2], [0, 1])] {
            let expected = [([0, 0], colors[0]), (middle, [124; 3]), (end, colors[1])];
            assert_eq!(pixels([[0, 0], end], colors), expected);
            assert_eq!(pixels([end, [0, 0]], [colors[1], colors[0]]), expected);
        }
        assert!(pixels([[-512, 0], [512, 0]], colors).is_empty());
        assert!(pixels([[0, -256], [0, 256]], colors).is_empty());
    }

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
    #[ignore = "requires the supported TM1_CUE"]
    fn dashboard_sprites_fit_forward_blend_pass() {
        use tm1_core::drive::{Level::*, Vehicle};
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut paths = std::collections::BTreeSet::new();
        for level in [Arena, Warehouse, Freeway, CityPark, Cyburbia, Rooftops] {
            paths.insert(format!("UA2PLAY/{}.DMD", level.duel_database().unwrap()));
            for car in Vehicle::ROSTER {
                paths.insert(format!("UADMD/{}.DMD", level.database(car)));
            }
        }
        for path in paths {
            let data = disc.read(&path).unwrap();
            let icons = tm1_assets::scene::Dmd::parse(&data)
                .unwrap()
                .dashboard_icons()
                .unwrap();
            for (sprite, semi) in icons.frames() {
                assert!(
                    !semi || PacketMode::texture(sprite.texture) != PacketMode::Subtract,
                    "{path} page {:x}",
                    sprite.texture.page
                );
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn sky_allocation_matches_jp_view_modes() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        for split in [false, true] {
            let mut cpu = crate::reference_cpu::Cpu::new(&exe);
            let sky = 0x80008000;
            let node = 0x80009000u32;
            cpu.put(sky + 0x104, &node.to_le_bytes());
            cpu.call(0x800ffad8, &[u32::from(split)]);
            cpu.call(0x8010ac38, &[sky]);
            assert_eq!(cpu.call(0x8010aca0, &[0]), sky);
            assert_eq!(cpu.word(sky), 0xffe);
            let second = cpu.call(0x8010aca0, &[1]);
            assert_eq!(second != 0, split);
            if split {
                assert_ne!(second, sky);
                assert_eq!(cpu.word(second + 0x104), node);
                assert_eq!(cpu.word(second), 0xffe);
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn fixed_bucket_skies_use_opaque_polygons() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        for path in [
            "UADMD/WH1",
            "UADMD/WH2",
            "UADMD/XFWY1",
            "UADMD/XFWY2",
            "UADMD/PARK1",
            "UADMD/PARK2",
            "UADMD/SUBURB1",
            "UADMD/SUBURB2",
            "UADMD/ROOF1",
            "UADMD/ROOF2",
            "UA2PLAY/2PWH",
            "UA2PLAY/2PFWY",
            "UA2PLAY/2PPARK",
            "UA2PLAY/2PCANALS",
            "UA2PLAY/2PROOF",
        ] {
            let data = disc.read(&format!("{path}.DMD")).unwrap();
            let sky = tm1_assets::scene::Dmd::parse(&data)
                .unwrap()
                .battle_sky()
                .unwrap();
            assert!(!sky.polygons.is_empty());
            assert!(sky.visibility.is_empty(), "{path}");
            assert_eq!(
                sky.polygons
                    .iter()
                    .filter(|p| p.first_in_object)
                    .map(|p| p.object_id)
                    .collect::<Vec<_>>(),
                sky.object_ids,
                "{path}"
            );
            assert!(
                sky.polygons.iter().all(|p| p.fixed_ot == 0
                    && matches!(
                        PacketMode::polygon(p),
                        PacketMode::Opaque | PacketMode::UntexturedOpaque
                    )),
                "{path}"
            );
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn rival_health_bar_matches_jp_dashboard_packets() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let ai = 0x8019f7a8;
        let bar = 0x8019d578;
        let ot = 0x80007000u32;
        cpu.put(0x8018c5f4, &ot.to_le_bytes());
        cpu.put(0x8018c334, &1u16.to_le_bytes());
        cpu.put(0x8018c338, &1u16.to_le_bytes());
        cpu.put(ai + 0x250, &130i32.to_le_bytes());
        cpu.put(bar + 8, &103u16.to_le_bytes());
        cpu.put(bar + 24, &103u16.to_le_bytes());
        // Skip font endpoints while retaining target selection, bar calculation and AddPrim.
        for entry in [0x8010bf60, 0x8010b4f4, 0x8010bf8c] {
            cpu.put(entry, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        for hp in [-1, 0, 1, 13, 39, 65, 129, 130, 131, i32::MAX] {
            for tier in 0..=3 {
                for active in [false, true] {
                    cpu.put(ai + 2, &[tier]);
                    cpu.put(ai + 0x41, &[u8::from(active)]);
                    cpu.put(ai + 0x24c, &hp.to_le_bytes());
                    cpu.put(ot, &0xffffffu32.to_le_bytes());
                    cpu.call_until(0x80142ea4, &[0], 0x80143068);
                    let head = cpu.word(ot) & 0xffffff;
                    if !active {
                        assert_eq!(head, 0xffffff);
                        continue;
                    }
                    assert_eq!(head, bar & 0xffffff);
                    let right = 103 + rival_health_width(hp, 130);
                    for (corner, color) in [0, 1, 0, 1].into_iter().enumerate() {
                        let vertex = bar + 8 * corner as u32;
                        assert_eq!(
                            cpu.word(vertex + 4).to_le_bytes()[..3],
                            health_colors(tier)[color]
                        );
                        assert_eq!(
                            i32::from(cpu.word(vertex + 8) as u16),
                            if color == 0 { 103 } else { right }
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn dashboard_gouraud_rounds_command_color_before_rgb555_conversion() {
        let ramp = [[0, 31, 112], [31, 0, 112]];
        assert_eq!(
            bar_spans(2, ramp).collect::<Vec<_>>(),
            [(0, 1, [0, 24, 112]), (1, 1, [16, 16, 112])]
        );
        assert_eq!(
            bar_spans(88, [[10; 3]; 2]).collect::<Vec<_>>(),
            [(0, 88, [8; 3])]
        );
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
                let mut z = [0u32; 4];
                for (&corner, pos) in indices(n).iter().zip(&batch.positions[start as usize..]) {
                    z[corner] = pos[2] as u32;
                }
                let (_, bucket) = hierarchy_bucket(n, z, 0).unwrap();
                batch.order_packet(len, bucket, Hierarchy, mode);
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
                    assert_eq!(
                        hierarchy_bucket(n, depths, fixed).map_or(0, |(_, bucket)| bucket),
                        expected,
                        "{n} corners, depths {depths:?}, fixed {fixed}"
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn world_vertex_depth_matches_jp_rot_trans() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let matrix = 0x80010000;
        let input = matrix + 32;
        let output = input + 8;
        cpu.put(matrix, &[0; 32]);
        for row in [[0i16, 3690, 0], [1513, -3097, 947], [-3671, 113, -284]] {
            cpu.put(matrix + 12, &row.map(i16::to_le_bytes).concat());
            cpu.call(0x8015c740, &[matrix]);
            for translation in [-1033i32, 17, 6017] {
                cpu.put(matrix + 28, &translation.to_le_bytes());
                cpu.call(0x8015c7d0, &[matrix]);
                let depth = GeometryDepth::from_world_matrix(row.map(i32::from), translation);
                for x in -64i16..=64 {
                    let p = [x * 9, 193 - x * 5, x - 701];
                    cpu.put(input, &p.map(i16::to_le_bytes).concat());
                    cpu.call(0x8015d414, &[input, output, output + 12]);
                    let z = cpu.word(output + 8) as i32;
                    assert_eq!(
                        depth.vertex(world(p.map(f32::from)).to_array()),
                        z.max(0) as u32
                    );
                }
            }
        }
    }

    #[test]
    fn gpu_triangle_span_limits_include_the_boundary() {
        assert!(triangle_fits_gpu([[-500, -250], [523, -250], [523, 261]]));
        assert!(!triangle_fits_gpu([[-501, -250], [523, -250], [523, 261]]));
        assert!(!triangle_fits_gpu([[-500, -251], [523, -250], [523, 261]]));
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn subdivided_packets_match_jp_vertices_and_links() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let db: u32 = 0x80010000;
        let flags = db + 64;
        let record = db + 128;
        let packet = db + 192;
        let extra = db + 256;
        let ot = db + 1024;
        let corners = std::array::from_fn(|i| PacketCorner {
            xy: [[-711, -102], [317, -35], [-161, 446], [402, 310]][i],
            uv: [[8, 253], [143, 16], [201, 33], [237, 204]][i],
            rgb: [[115, 170, 23], [204, 5, 39], [91, 36, 216], [249, 249, 174]][i],
        });
        for (n, routine) in [(3, 0x80108ae4), (4, 0x801090c4)] {
            let words = (n * 3) as u32;
            cpu.put(db, &ot.to_le_bytes());
            cpu.put(db + 8, &extra.to_le_bytes());
            cpu.put(db + 12, &(extra + 600).to_le_bytes());
            cpu.put(record, &flags.to_le_bytes());
            cpu.put(record + 4, &7u32.to_le_bytes());
            cpu.put(record + 8, &packet.to_le_bytes());
            cpu.put(ot + 28, &0xffffffu32.to_le_bytes());
            cpu.put(flags, &[n as u8, n as u8, 0, words as u8]);
            cpu.put(flags + 14, &[0xc0 | n as u8]);
            for (i, c) in corners[..n].iter().enumerate() {
                let base = packet + 4 + 12 * i as u32;
                cpu.put(base, &c.rgb.map(|v| v as u8));
                cpu.put(base + 4, &c.xy.map(|v| (v as i16).to_le_bytes()).concat());
                cpu.put(base + 8, &c.uv.map(|v| v as u8));
            }
            cpu.call(routine, &[db, record, 1]);
            let mut next = cpu.word(ot + 28) & 0xffffff;
            for (index, child) in subdivided_faces(corners, n).iter().enumerate().rev() {
                let addr = if index == 0 {
                    packet
                } else {
                    extra + (index as u32 - 1) * 4 * (words + 1)
                };
                assert_eq!(next, addr & 0xffffff);
                for (i, c) in child[..n].iter().enumerate() {
                    let base = addr + 4 + 12 * i as u32;
                    let xy = cpu.word(base + 4);
                    assert_eq!(c.xy, [xy as i16 as i32, (xy >> 16) as i16 as i32]);
                    assert_eq!(
                        c.uv,
                        [0, 8].map(|s| ((cpu.word(base + 8) >> s) & 255) as i32)
                    );
                    assert_eq!(
                        c.rgb,
                        [0, 8, 16].map(|s| ((cpu.word(base) >> s) & 255) as i32)
                    );
                }
                next = cpu.word(addr) & 0xffffff;
            }
            assert_eq!(next, 0xffffff);
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn model_face_winding_matches_retail_geom_proc() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let stub = 0x80010000;
        let words = |code: &[u32]| {
            code.iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()
        };
        // Return 1 when the JP branch reaches packet emission, 0 at geomNextPrim.
        cpu.put(0x800f8154, &words(&[0x03e00008, 0x00001021]));
        for (start, accept, area_reg, mirror_reg) in [
            (0x800f7bd4, 0x800f7bf4, 22, 23),
            (0x800f7cb0, 0x800f7cd0, 23, 2),
        ] {
            cpu.put(accept, &words(&[0x03e00008, 0x24020001]));
            cpu.put(
                stub,
                &words(&[
                    0x00800021 | (area_reg << 11),
                    0x08000000 | ((start >> 2) & 0x03ffffff),
                    0x00a00021 | (mirror_reg << 11),
                ]),
            );
            for points in [
                [[0, 0], [1, 0], [0, 1]],
                [[0, 0], [0, 1], [1, 0]],
                [[6, 1], [6, 1], [17, 8]],
                [[-1024, -1024], [1023, 1023], [-1024, 1023]],
            ] {
                let [a, b, c] = points;
                // NCLIP's signed shoelace sum uses the three projected SXY entries.
                let nclip: i32 = a[0] * b[1] + b[0] * c[1] + c[0] * a[1]
                    - a[0] * c[1]
                    - b[0] * a[1]
                    - c[0] * b[1];
                for mirror in [false, true] {
                    assert_eq!(
                        packet_front_facing(points, mirror),
                        cpu.call(stub, &[nclip as u32, u32::from(mirror)]) != 0
                    );
                }
            }
        }
    }

    #[test]
    fn projected_centers_match_gte_reference() {
        // Reference checksums use DuckStation's 257-entry UNR table, RTPS IR/SXY saturation and native aspect.
        for (lens, center, expected) in [
            (91, [50, 15], 0xb59ff7ad),
            (228, [160, 60], 0xc103de2b),
            (228, [160, 120], 0x0b9809c7),
        ] {
            let mut hash = 0x811c9dc5u32;
            for z in 0..=65535 {
                for xy in [[-32769, 32768], [(z % 511) as i32 - 255, 119]] {
                    for offset in gte_screen_offset(xy, z, lens, center) {
                        hash = (hash ^ offset as u32).wrapping_mul(0x01000193);
                    }
                }
            }
            assert_eq!(hash, expected, "lens {lens} center {center:?}");
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn sprite_half_sizes_match_jp_geom_sprite() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let record = 0x80010000;
        // Restore t1 (polygon), s2 (lens), s3 (SZ) and the signed width loaded in the branch delay slot.
        let jump = 0x08000000 | ((0x800f7d90u32 >> 2) & 0x03ffffff);
        cpu.put(
            0x80011000,
            &[0x00804821, 0x00a09021, 0x00c09821, jump, 0x85340006]
                .map(u32::to_le_bytes)
                .concat(),
        );
        for extent in [
            [-21i16, 0],
            [1, 2],
            [61, 97],
            [499, 501],
            [500, 500],
            [32767, -32768],
        ] {
            cpu.put(record + 6, &extent.map(i16::to_le_bytes).concat());
            for lens in [91, 228] {
                for depth in [16, 17, 90, 91, 92, 227, 228, 229, 1000, 32767] {
                    cpu.call_until(0x80011000, &[record, lens, depth], 0x800f7dfc);
                    assert_eq!(
                        sprite_half_size(extent, lens as i32, depth),
                        [cpu.r[22] as i32, cpu.r[23] as i32]
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn hammerhead_wheel_depth_matches_hier_traverse() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let db = tm1_assets::scene::Dmd::parse(&data).unwrap();
        let [body, _, _] = db.vehicle_bodies(40).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        cpu.put(base, &data);
        let cs = 0x80008000;
        let parent = tm1_core::drive::HierarchyTransform {
            normals: [[0; 3]; 6],
            rows: [[3678, 1780, -292], [-202, 1050, 3525], [1513, -3097, 947]],
            position: [77, -420, 151],
        };
        for (i, v) in parent.rows.as_flattened().iter().copied().enumerate() {
            cpu.put(cs + 0x34 + 2 * i as u32, &(v as i16).to_le_bytes());
        }
        for (i, v) in parent.position.into_iter().enumerate() {
            cpu.put(cs + 0x68 + 4 * i as u32, &v.to_le_bytes());
        }
        // Isolate traversal from frustum rejection and use the regular vehicle LOD setting.
        cpu.put(0x800fe034, &[8, 0, 224, 3, 1, 0, 2, 36]);
        cpu.call(0x800fe9a0, &[3]);
        let root = db.roots.iter().find(|r| r.1 == 40).unwrap().0;
        // Enter geomProc's EO translation block with the matrix in s5 and object in a1.
        let jump = 0x08000000 | ((0x800f79a4u32 >> 2) & 0x03ffffff);
        cpu.put(
            0x80007000,
            &[jump, 0x0080a821].map(u32::to_le_bytes).concat(),
        );
        for wheel in [[0i16, 4096], [1567, 3784], [-2896, 2896]] {
            let [sin, cos] = wheel;
            let rot = [4096, 0, 0, 0, cos, sin, 0, -sin, cos]
                .map(i16::to_le_bytes)
                .concat();
            for i in (0..data.len() - 40).step_by(4) {
                if data[i] == 5
                    && data[i + 2..i + 4] == 42u16.to_le_bytes()
                    && (1..=4).contains(&u16::from_le_bytes(
                        data[i + 36..i + 38].try_into().unwrap(),
                    ))
                {
                    cpu.put(base + i as u32 + 4, &rot);
                }
            }
            cpu.put(0x8018c484, &[0; 12]);
            cpu.call(0x800fd914, &[cs, root]);
            let depths = GeometryTransform::hierarchy(parent, &body.transforms, wheel);
            let mut pivots = std::collections::BTreeSet::new();
            for i in 0..cpu.word(0x8018c484) {
                let record = 0x801acfe8 + 32 * i;
                let id = cpu.word(record);
                let matrix = 0x801aeb70 + 32 * cpu.word(record + 24);
                // geomProc installs the translation from this object's recorded position.
                cpu.call_until(0x80007000, &[matrix, record], 0x800f7a94);
                let rows = std::array::from_fn(|i| {
                    std::array::from_fn(|j| {
                        let k = 2 * (3 * i + j) as u32;
                        (cpu.word(matrix + (k & !3)) >> (8 * (k & 3))) as i16 as i32
                    })
                });
                let translation = [20, 24, 28].map(|k| cpu.word(matrix + k) as i32);
                for p in body.polygons.iter().filter(|p| p.object_id == id) {
                    let Some(Wheel::Hammerhead { pivot, .. }) = p.wheel else {
                        continue;
                    };
                    if depths[p.transform_index].rows == rows
                        && depths[p.transform_index].translation == translation
                    {
                        pivots.insert(pivot.map(|v| v as i32));
                    }
                }
            }
            assert_eq!(pivots.len(), 4, "wheel {wheel:?}");
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn arena_object_order_follows_jp_hierarchy_stack() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        for name in ["XARENA1", "WH1", "XFWY1", "PARK1", "SUBURB1", "ROOF1"] {
            let data = disc.read(&format!("UADMD/{name}.DMD")).unwrap();
            let db = tm1_assets::scene::Dmd::parse(&data).unwrap();
            let scene = db.battle_world().unwrap();
            let mut cpu = crate::reference_cpu::Cpu::new(&exe);
            cpu.put(u32::from_le_bytes(data[12..16].try_into().unwrap()), &data);
            let cs = 0x80008000;
            for offset in [0x34, 0x3c, 0x44] {
                cpu.put(cs + offset, &4096i16.to_le_bytes());
            }
            cpu.call(0x800fe9a0, &[3]);
            // Keep all frustum branches open to exercise the original 220-object list.
            cpu.put(0x800fe034, &[8, 0, 224, 3, 1, 0, 2, 36]);
            let mut total = 0;
            for position in [[0i32; 3], [752, -246, 157], [-5056, 3145, 1600]] {
                cpu.put(cs + 0x68, &position.map(i32::to_le_bytes).concat());
                cpu.put(0x8018c484, &[0; 12]);
                cpu.call(0x800fd914, &[cs, scene.root_id]);
                let count = cpu.word(0x8018c484);
                assert!(count <= 220);
                total += count;
                // Extraction retains mutually exclusive switch and LOD branches.
                // Every active occurrence must appear in order, including repeated pointers.
                let mut objects = scene.object_ids.iter().copied();
                for i in 0..count {
                    let id = cpu.word(0x801acfe8 + 32 * i);
                    assert!(
                        objects.any(|ptr| ptr == id),
                        "{name} {position:?} object {i}/{count}: {id:#x}"
                    );
                }
            }
            assert!(total > 0, "{name} has no reference objects");
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_geometry_has_a_first_face_for_each_object_occurrence() {
        use tm1_core::drive::{Level::*, Vehicle};
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut paths = std::collections::BTreeSet::new();
        for level in [
            Arena, Warehouse, Freeway, CityPark, Cyburbia, Rooftops, Arena5,
        ] {
            if let Some(name) = level.duel_database() {
                paths.insert(format!("UA2PLAY/{name}.DMD"));
            }
            for car in Vehicle::ROSTER {
                paths.insert(format!("UADMD/{}.DMD", level.database(car)));
            }
        }
        for path in paths {
            let data = disc.read(&path).unwrap();
            let db = tm1_assets::scene::Dmd::parse(&data).unwrap();
            let check = |model: tm1_assets::scene::Scene| {
                assert_eq!(
                    model
                        .polygons
                        .iter()
                        .filter(|p| p.first_in_object)
                        .map(|p| p.object_id)
                        .collect::<Vec<_>>(),
                    model.object_ids,
                    "{path} root {:#x}",
                    model.root_id
                );
            };
            check(
                db.battle_geometry(
                    db.roots.iter().any(|r| r.1 == 3) && !db.roots.iter().any(|r| r.1 == 2),
                )
                .unwrap(),
            );
            for &(_, kind) in &db.roots {
                if (10..=130).contains(&kind) && kind % 10 == 0 {
                    for stage in db.vehicle_bodies_with_lights(kind, true).unwrap() {
                        check(stage);
                    }
                } else if matches!(kind, 201 | 312) {
                    check(db.extract_kind(kind).unwrap());
                }
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn coordinate_system_order_matches_jp_database_scan_and_weapon_pools() {
        use tm1_core::drive::{Level::*, Vehicle};
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let mut paths = std::collections::BTreeSet::new();
        for level in [
            Arena, Warehouse, Freeway, CityPark, Cyburbia, Rooftops, Arena5,
        ] {
            if let Some(name) = level.duel_database() {
                paths.insert(format!("UA2PLAY/{name}.DMD"));
            }
            for car in Vehicle::ROSTER {
                paths.insert(format!("UADMD/{}.DMD", level.database(car)));
            }
        }
        for path in paths {
            let data = disc.read(&path).unwrap();
            let db = tm1_assets::scene::Dmd::parse(&data).unwrap();
            let mut cpu = crate::reference_cpu::Cpu::new(&exe);
            cpu.put(u32::from_le_bytes(data[12..16].try_into().unwrap()), &data);
            // Capture DB's dispatch sequence while isolating model registration from other game setup.
            cpu.put(0x8010083c, &[8, 0, 224, 3, 0, 0, 0, 0]);
            let mut created = Vec::new();
            for &(root, _) in &db.roots {
                cpu.call_observed(0x801001f8, &[root], 0x80000080, |cpu| {
                    if cpu.pc() != 0x8010083c {
                        return;
                    }
                    let kind = cpu.r[4];
                    if kind == 3
                        || ((10..=130).contains(&kind) && kind % 10 == 0)
                        || matches!(kind, 750 | 752) && cpu.word(cpu.r[5]) as u8 == 1
                        || matches!(
                            kind,
                            751 | 756 | 760 | 772 | 773 | 775 | 776 | 777 | 778 | 781 | 782
                        )
                    {
                        created.push(cpu.r[5]);
                    }
                });
            }
            created.reverse();
            let models: std::collections::HashSet<_> = created.iter().copied().collect();
            assert_eq!(
                db.battle_model_order()
                    .unwrap()
                    .into_iter()
                    .filter(|id| models.contains(id))
                    .collect::<Vec<_>>(),
                created,
                "{path}"
            );
        }
        let mut cpu = crate::reference_cpu::Cpu::new(&exe);
        let prior = 0x80008000;
        cpu.call(0x800ff34c, &[]);
        cpu.call(0x800ff1f8, &[prior]);
        cpu.call(0x8010ecd4, &[]);
        cpu.call(0x8010e414, &[]);
        let mut cs = cpu.call(0x800ff1ec, &[]);
        for member in [0x100, 0x24] {
            for slot in (0..10).rev() {
                let missile = 0x801a9c10 + 0x3e8 * slot;
                assert_eq!(cs, cpu.word(missile + member));
                if member == 0x100 {
                    let object = cpu.word(cs + 0x104);
                    assert_eq!(cpu.word(object + 24), 0x2000);
                }
                cs = cpu.word(cs + 0x108);
            }
        }
        assert_eq!(cs, prior);
    }

    pub(super) fn test_branches() -> hierarchy::BranchState<'static> {
        hierarchy::BranchState {
            wheels: [0; 14],
            rotation: [0, 4096],
            switches: [0; 8],
            stands: 0,
            pickups: 0,
            damage: &|_| 0,
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn shared_object_limit_matches_retail_traversal() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut data = disc.read("UADMD/WH1.DMD").unwrap();
        let db = tm1_assets::scene::Dmd::parse(&data).unwrap();
        let world = db.battle_world().unwrap();
        let ids: Vec<_> = world
            .polygons
            .iter()
            .filter(|p| p.first_in_object)
            .map(|p| p.object_id)
            .collect();
        assert_eq!(ids, world.object_ids);
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        // A group of repeated Warehouse objects is traversed four times as separate CS roots.
        // Bypass spatial rejection via the normal object and group flags.
        let count = 79;
        let offset = data.len().next_multiple_of(4);
        let group = base + offset as u32;
        data.resize(offset + 24 + 4 * count, 0);
        data[24..28].copy_from_slice(&group.to_le_bytes());
        data[offset] = 1;
        data[offset + 2..offset + 4].copy_from_slice(&0xfffeu16.to_le_bytes());
        data[offset + 20] = count as u8;
        data[offset + 21] = 4;
        for i in 0..count {
            let id = ids[(i / 3) % ids.len()];
            let obj = (id - base) as usize;
            data[obj + 25] |= 0x20;
            let child = offset + 24 + i * 4;
            data[child..child + 4].copy_from_slice(&id.to_le_bytes());
        }
        let scene = tm1_assets::scene::Dmd::parse(&data)
            .unwrap()
            .extract_kind(0xfffe)
            .unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(base, &data);
        let root = tm1_core::drive::HierarchyTransform {
            rows: [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
            normals: [[0; 3]; 6],
            position: [0; 3],
        };
        let mut pass = hierarchy::Pass::new(root, true);
        for _ in 0..4 {
            cpu.call(0x800fd914, &[0x80008000, group]);
            let slot = pass.coordinate_system(root, None);
            pass.traverse((), &scene, 0, 0, slot, 1200, true, &test_branches());
            let count = cpu.word(0x8018c484);
            assert_eq!(pass.objects.len(), count as usize);
            for (i, obj) in pass.objects.iter().enumerate() {
                assert_eq!(
                    scene.polygons[obj.polygons.start].object_id,
                    cpu.word(0x801acfe8 + 32 * i as u32)
                );
            }
        }
        assert_eq!(pass.objects.len(), 220);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn arena_frustum_matches_retail_bound_tests() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        for name in ["XARENA1", "WH1", "XFWY1", "PARK1", "SUBURB1", "ROOF1"] {
            let data = disc.read(&format!("UADMD/{name}.DMD")).unwrap();
            let scene = tm1_assets::scene::Dmd::parse(&data)
                .unwrap()
                .battle_geometry(name == "XARENA1")
                .unwrap();
            let mut outcomes = [0; 2];
            for (fov, depth, position) in [
                (6, 1200i32, [0, 0, 0]),
                (11, 1200, [752, -246, 157]),
                (13, 100, [-5056, 3145, 1600]),
                (13, 1000, [0, 0, -149]),
            ] {
                let normals = std::array::from_fn(|i| {
                    std::array::from_fn(|j| {
                        let addr = 0x170270 + 96 * fov + 48 + 8 * i + 2 * j;
                        i32::from(i16::from_le_bytes(
                            cpu.ram[addr..addr + 2].try_into().unwrap(),
                        ))
                    })
                });
                let root = tm1_core::drive::HierarchyTransform {
                    rows: [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
                    position,
                    normals,
                };
                let frames = GeometryTransform::hierarchy(root, &scene.transforms, [0, 4096]);
                cpu.put(0x8018bf04, &depth.to_le_bytes());
                for bound in &scene.bounds {
                    let frame = frames[bound.transform];
                    cpu.put(0x80008000, &bound.center.map(i32::to_le_bytes).concat());
                    cpu.put(0x80008010, &frame.position.map(i32::to_le_bytes).concat());
                    for (plane, n) in frame.normals.iter().enumerate() {
                        for (axis, x) in n.iter().enumerate() {
                            cpu.put(
                                0x80008020 + (32 * (plane / 3) + 6 * axis + 2 * (plane % 3)) as u32,
                                &(*x as i16).to_le_bytes(),
                            );
                        }
                    }
                    let expected = cpu.call(
                        0x800fe034,
                        &[
                            0x80008000,
                            bound.radius as u32,
                            0x80008010,
                            0x80008020,
                            0x80008040,
                        ],
                    ) != 0;
                    assert_eq!(
                        frame.contains(bound, depth),
                        expected,
                        "{name} entry {fov} center {:?}",
                        bound.center
                    );
                    outcomes[usize::from(expected)] += 1;
                }
            }
            assert!(
                outcomes.iter().all(|&count| count > 0),
                "{name}: {outcomes:?}"
            );
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn warehouse_subdivision_admission_matches_full_retail_queue() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/WH1.DMD").unwrap();
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let scene = tm1_assets::scene::Dmd::parse(&data)
            .unwrap()
            .battle_world()
            .unwrap();
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(base, &data);
        let scratch = 0x1f800000;
        let list = 0x80008000u32;
        cpu.put(scratch + 0x4c, &list.to_le_bytes());
        let words = |code: &[u32]| {
            code.iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()
        };
        // Restore geomProc registers at the end of UV copying. Untextured faces enter geomAddPrim directly.
        for (stub, target) in [(0x80006000, 0x800f7f60u32), (0x80006100, 0x800f7f68)] {
            cpu.put(
                stub,
                &words(&[
                    0x3c081f80,
                    0x00807021,
                    0x00a04821,
                    0x00c07821,
                    0x08000000 | ((target >> 2) & 0x03ffffff),
                    0x00008821,
                ]),
            );
        }
        cpu.put(0x800f7fec, &words(&[0x03e00008, 0x00001021]));
        cpu.put(0x800f8154, &words(&[0x03e00008, 0x24020001]));
        let mut count = 0;
        let mut overflow = 0;
        for (i, p) in scene
            .polygons
            .iter()
            .cycle()
            .take(scene.polygons.len() * 8)
            .enumerate()
        {
            let bucket = [99, 100, 101, 2, 87][i % 5];
            let addr = base + p.source_offset as u32;
            let stub = if data[p.source_offset] & 0x7f != 0 {
                0x80006000
            } else {
                0x80006100
            };
            let queued = cpu.call(stub, &[p.object_id, addr, bucket]) != 0;
            assert_eq!(
                queue_subdivision(&mut count, p.subdivide_near, bucket),
                queued
            );
            assert_eq!(count, cpu.word(scratch + 0x38) as usize);
            if queued {
                let record = list + (count as u32 - 1) * 20;
                assert_eq!(cpu.word(record), addr);
                assert_eq!(cpu.word(record + 4), bucket);
            } else if count == 200 && p.subdivide_near && bucket < 100 {
                overflow += 1;
                if overflow == 20 {
                    break;
                }
            }
        }
        assert_eq!((count, overflow), (200, 20));
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn battle_branches_match_jp_stack_during_world_traversal() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let mut counts = [0; 2];
        for name in ["XARENA1", "WH1", "XFWY1", "PARK1", "SUBURB1", "ROOF1"] {
            let data = disc.read(&format!("UADMD/{name}.DMD")).unwrap();
            let scene = tm1_assets::scene::Dmd::parse(&data)
                .unwrap()
                .battle_world()
                .unwrap();
            let mut cpu = crate::reference_cpu::Cpu::new(&exe);
            cpu.put(u32::from_le_bytes(data[12..16].try_into().unwrap()), &data);
            let cs = 0x80008000;
            cpu.call(0x800fe9a0, &[3]);
            // Include branches behind the camera. Keep the traversal's real capacity and switch state.
            cpu.put(0x800fe034, &[8, 0, 224, 3, 1, 0, 2, 36]);
            for position in [
                [0, 0, 0],
                [752, -246, 157],
                [-5056, 3145, 1600],
                [2013, 10008, -348],
            ] {
                let root = tm1_core::drive::HierarchyTransform {
                    rows: [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
                    position,
                    normals: [[0; 3]; 6],
                };
                let frames = GeometryTransform::hierarchy(root, &scene.transforms, [0, 4096]);
                cpu.put(cs + 0x68, &position.map(i32::to_le_bytes).concat());
                cpu.put(0x8018c484, &[0; 12]);
                let mut pending = None;
                cpu.call_observed(0x800fd914, &[cs, scene.root_id], 0x80000080, |cpu| {
                    if let Some((ret, expected, actual)) = pending {
                        if cpu.pc() == ret {
                            assert_eq!(expected, actual, "{name} at {position:?}");
                            pending = None;
                        } else if cpu.pc() == 0x800fd7dc {
                            pending = Some((ret, expected, Some(cpu.r[4])));
                        }
                    }
                    let ctrl = cpu.pc() == 0x800fe2a4;
                    if cpu.pc() != 0x800fe144 && !ctrl {
                        return;
                    }
                    let node = cpu.r[4];
                    let pos = [8, 12, 16].map(|d| cpu.word(cpu.r[5] + d) as i32);
                    let expected = if ctrl {
                        let c = scene
                            .visibility
                            .iter()
                            .find(|c| c.node == node && frames[c.transform_index()].position == pos)
                            .expect("controller stack position");
                        let addr = cpu.word(node + if c.inside_at_position(pos) { 16 } else { 12 });
                        (addr != 0).then_some(addr)
                    } else {
                        let l = scene
                            .lods
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find(|l| l.node == node && frames[l.transform_index()].position == pos)
                            .expect("LOD stack position");
                        l.selected_child(pos, 3)
                    };
                    counts[usize::from(ctrl)] += 1;
                    pending = Some((cpu.r[31], expected, None));
                });
                assert!(pending.is_none());
            }
        }
        assert!(counts.iter().all(|&n| n > 0), "{counts:?}");
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn nested_lod_uses_stack_position_even_when_retail_matrix_slots_fill() {
        let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut data = disc.read("UADMD/WH1.DMD").unwrap();
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let source = tm1_assets::scene::Dmd::parse(&data)
            .unwrap()
            .battle_world()
            .unwrap();
        let object = source.object_ids[0];
        let other = *source.object_ids.iter().find(|&&id| id != object).unwrap();
        for id in [object, other] {
            data[(id - base) as usize + 25] |= 0x20;
        }
        // Two oblique rotations around a two-range LOD using real Warehouse objects.
        let start = data.len().next_multiple_of(4);
        data.resize(start + 144, 0);
        let root = base + start as u32;
        data[24..28].copy_from_slice(&root.to_le_bytes());
        let block = &mut data[start..];
        for (offset, rot, trans, child) in [
            (
                0usize,
                [[3784i16, -1567, 0], [1567, 3784, 0], [0, 0, 4096]],
                [41i32, -93, 51],
                root + 44,
            ),
            (
                44,
                [[4096, 0, 0], [0, 2896, -2896], [0, 2896, 2896]],
                [-15, 27, 33],
                root + 88,
            ),
        ] {
            block[offset] = 5;
            block[offset + 4..offset + 22].copy_from_slice(
                &rot.as_flattened()
                    .iter()
                    .flat_map(|n| n.to_le_bytes())
                    .collect::<Vec<_>>(),
            );
            block[offset + 24..offset + 36].copy_from_slice(&trans.map(i32::to_le_bytes).concat());
            block[offset + 38] = 1;
            block[offset + 40..offset + 44].copy_from_slice(&child.to_le_bytes());
        }
        block[2..4].copy_from_slice(&0xfffeu16.to_le_bytes());
        block[88] = 2;
        for (ofs, w) in [
            (104, 2),
            (108, root + 116),
            (112, root + 128),
            (116, 10000),
            (120, 0),
            (124, object),
            (128, u32::MAX),
            (132, 10000),
            (136, other),
        ] {
            block[ofs..ofs + 4].copy_from_slice(&w.to_le_bytes());
        }
        let mut db = tm1_assets::scene::Dmd::parse(&data).unwrap();
        db.roots = vec![(root, 1)];
        let scene = db.battle_world().unwrap();
        let lod = &scene.lods.as_ref().unwrap()[0];
        let mut cpu = crate::reference_cpu::Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(base, &data);
        let cs = 0x80008000;
        let identity = [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]];
        for offset in [0x34, 0x3c, 0x44] {
            cpu.put(cs + offset, &4096i16.to_le_bytes());
        }
        cpu.call(0x800fe9a0, &[3]);
        let mut choices = std::collections::BTreeSet::new();
        let mut different = 0;
        let mut inherited_rows = None;
        for initial in [0u32, 69, 70] {
            for y in -8..8 {
                for x in 735..760 {
                    let position = [x, y, 93];
                    let frame = tm1_core::drive::HierarchyTransform {
                        rows: identity,
                        position,
                        normals: [[0; 3]; 6],
                    };
                    let frames = GeometryTransform::hierarchy(frame, &scene.transforms, [0, 4096]);
                    cpu.put(cs + 0x68, &position.map(i32::to_le_bytes).concat());
                    cpu.put(0x8018c484, &[0; 12]);
                    cpu.put(0x8018c488, &initial.to_le_bytes());
                    let mut seen = None;
                    cpu.call_observed(0x800fd914, &[cs, root], 0x80000080, |cpu| {
                        if cpu.pc() == 0x800fe144 {
                            let e = cpu.r[5];
                            seen = Some([8, 12, 16].map(|d| cpu.word(e + d) as i32));
                        }
                    });
                    let pos = frames[lod.transform_index()].position;
                    assert_eq!(seen, Some(pos));
                    assert_eq!(cpu.word(0x8018c484), 1);
                    let slot = (initial + 2).min(70);
                    assert_eq!(cpu.word(0x801acfe8 + 24), slot);
                    let rows =
                        [0, 4, 8, 12, 16].map(|offset| cpu.word(0x801aeb70 + 32 * slot + offset));
                    if x == 735 && y == -8 {
                        if let Some(previous) = inherited_rows {
                            assert_ne!(rows, previous);
                        }
                        inherited_rows = Some(rows);
                    }
                    let id = cpu.word(0x801acfe8);
                    let mut hidden = vec![false; scene.polygons.len()];
                    lod.hide_at_position(pos, 3, &mut hidden);
                    assert!(
                        scene
                            .polygons
                            .iter()
                            .zip(&hidden)
                            .all(|(p, &h)| h == (p.object_id != id))
                    );
                    choices.insert(id);
                    hidden.fill(false);
                    lod.hide_other_branches(position.map(|v| -v as f32), 3, &mut hidden);
                    different += usize::from(
                        scene
                            .polygons
                            .iter()
                            .zip(hidden)
                            .any(|(p, h)| h != (p.object_id != id)),
                    );
                }
            }
        }
        // hierProc keeps the index at 70 for the next coordinate system. hierTraverse
        // writes the CS matrix into that slot even though earlier object records refer to it.
        let next_cs = 0x80008400;
        let matrix = [[0i16, -4096, 0], [4096, 0, 0], [0, 0, 4096]];
        let bytes = matrix
            .as_flattened()
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        let old_position = [4, 8, 12].map(|d| cpu.word(0x801acfe8 + d));
        cpu.put(next_cs + 0x34, &bytes);
        cpu.call(0x800fd914, &[next_cs, object]);
        assert_eq!(cpu.word(0x8018c484), 2);
        assert_eq!([0, 32].map(|d| cpu.word(0x801acfe8 + d + 24)), [70, 70]);
        assert_eq!([4, 8, 12].map(|d| cpu.word(0x801acfe8 + d)), old_position);
        let last_slot = (0x1aeb70 + 32 * 70) as usize;
        assert_eq!(&cpu.ram[last_slot..last_slot + 18], bytes);
        assert_eq!(choices.len(), 2);
        assert!(
            different > 0,
            "fixture must cross a boundary where composed floating transforms disagree"
        );
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
