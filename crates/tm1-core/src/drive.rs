//! Fixed-step vehicle, terrain, collision and projectile simulation for the JP arena.
unsafe extern "C" {
    fn tm1_use_scene(scene: *mut std::ffi::c_void);
    fn tm1_bind_level(level: i32);
    fn tm1_player_init(state: *mut u8, seed: *mut u32, kind: i32);
    fn tm1_drive_step(
        state: *mut u8,
        opponent: *mut u8,
        buttons: u16,
        fields: i16,
        frame: u32,
        table: *const i16,
        seed: *mut u32,
    );
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Arena,
    Warehouse,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vehicle {
    SweetTooth = 10,
    YellowJacket = 20,
    Darkside = 30,
    Hammerhead = 40,
    Outlaw = 50,
    CrimsonFury = 60,
    Warthog = 70,
    MrGrimm = 80,
    PitViper = 90,
    Thumper = 100,
    Spectre = 110,
    RoadKill = 120,
}
impl Vehicle {
    pub const ROSTER: [Self; 12] = [
        Self::SweetTooth,
        Self::YellowJacket,
        Self::Darkside,
        Self::Hammerhead,
        Self::Outlaw,
        Self::CrimsonFury,
        Self::Warthog,
        Self::MrGrimm,
        Self::PitViper,
        Self::Thumper,
        Self::Spectre,
        Self::RoadKill,
    ];
    pub fn slug(self) -> &'static str {
        match self {
            Self::SweetTooth => "sweet-tooth",
            Self::YellowJacket => "yellow-jacket",
            Self::Darkside => "darkside",
            Self::Hammerhead => "hammerhead",
            Self::Outlaw => "outlaw",
            Self::CrimsonFury => "crimson-fury",
            Self::MrGrimm => "mr-grimm",
            Self::PitViper => "pit-viper",
            Self::Warthog => "warthog",
            Self::Spectre => "spectre",
            Self::RoadKill => "road-kill",
            Self::Thumper => "thumper",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::SweetTooth => "Sweet Tooth",
            Self::YellowJacket => "Yellow Jacket",
            Self::Darkside => "Darkside",
            Self::Hammerhead => "Hammerhead",
            Self::Outlaw => "Outlaw",
            Self::CrimsonFury => "Crimson Fury",
            Self::MrGrimm => "Mr. Grimm",
            Self::PitViper => "Pit Viper",
            Self::Warthog => "Warthog",
            Self::Spectre => "Spectre",
            Self::RoadKill => "Road Kill",
            Self::Thumper => "Thumper",
        }
    }
}
pub type Taxi = PlayerVehicle;
pub struct PlayerVehicle {
    level: Level,
    vehicle: Vehicle,
    trig: Box<[i16]>,
    seed: u32,
    state: [u8; 0x4a4],
    frame: u32,
}
#[derive(Default, Clone, Copy)]
pub struct Input {
    pub accelerate: bool,
    pub reverse: bool,
    pub left: bool,
    pub right: bool,
    pub hard_turn: bool,
    pub turbo: bool,
    pub fire_guns: bool,
    pub missile: bool,
    pub cycle_weapon: bool,
}
impl Input {
    pub fn ps1_buttons(self) -> u16 {
        ((self.accelerate as u16) << 7)
            | ((self.reverse as u16) << 5)
            | ((self.left as u16) << 15)
            | ((self.right as u16) << 13)
            | ((self.hard_turn as u16) << 6)
            | ((self.turbo as u16) << 4)
            | ((self.fire_guns as u16) << 1)
            | self.missile as u16
            | ((self.cycle_weapon as u16) << 2)
    }
}
impl PlayerVehicle {
    pub fn from_executable(exe: &[u8]) -> Option<Self> {
        Self::for_vehicle(exe, Vehicle::YellowJacket)
    }
    pub fn for_vehicle(exe: &[u8], vehicle: Vehicle) -> Option<Self> {
        Self::on_level(exe, vehicle, Level::Arena)
    }
    pub fn on_level(exe: &[u8], vehicle: Vehicle, level: Level) -> Option<Self> {
        if exe.get(..8)? != b"PS-X EXE" {
            return None;
        }
        let base = u32::from_le_bytes(exe.get(24..28)?.try_into().ok()?);
        let offset = 2048 + 0x80183f14u32.checked_sub(base)? as usize;
        let trig: Vec<_> = exe
            .get(offset..offset + (0x80188d58 - 0x80183f14) + 2050)?
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        let trig = trig.into_boxed_slice();

        let mut state = [0; 0x4a4];
        let mut seed = 1;
        {
            let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                tm1_bind_level(level as i32);
                tm1_player_init(state.as_mut_ptr(), &mut seed, vehicle as i32)
            };
        }
        Some(Self {
            level,
            vehicle,
            state,
            frame: 0,
            trig,
            seed,
        })
    }
}
impl PlayerVehicle {
    pub fn reset(&mut self) {
        self.seed = 1;
        self.frame = 0;
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_level(self.level as i32);
            tm1_player_init(self.state.as_mut_ptr(), &mut self.seed, self.vehicle as i32)
        }
    }
    pub fn vehicle(&self) -> Vehicle {
        self.vehicle
    }
    pub fn rng_state(&self) -> u32 {
        self.seed
    }

    /// Reproducible start behind the arena's 80-unit northbound ramp.
    pub fn reset_at_test_ramp(&mut self) {
        self.reset();
        for (offset, value) in [(0x388, 2080i32), (0x38c, 1590), (0x3a8, 0), (0x3c0, 0)] {
            self.state[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }
    pub fn position(&self) -> [i32; 3] {
        std::array::from_fn(|i| self.word(0x388 + i * 4))
    }
    pub fn rotation(&self) -> [[i16; 3]; 3] {
        std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                let offset = 0x3d0 + 2 * (3 * i + j);
                i16::from_le_bytes(self.state[offset..offset + 2].try_into().unwrap())
            })
        })
    }
    pub fn frame(&self) -> u32 {
        self.frame
    }
    pub fn heading(&self) -> i32 {
        self.word(0x3a8)
    }
    fn word(&self, i: usize) -> i32 {
        i32::from_le_bytes(self.state[i..i + 4].try_into().unwrap())
    }
    pub fn step(&mut self, input: Input, fields: u8) {
        self.step_in_scene(input, fields, std::ptr::null_mut(), None)
    }
    fn step_in_scene(
        &mut self,
        input: Input,
        fields: u8,
        scene: *mut std::ffi::c_void,
        opponent: Option<&mut PoliceCar>,
    ) {
        assert!((1..=5).contains(&fields));
        self.frame = self.frame.wrapping_add(1);
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_level(self.level as i32);
            tm1_use_scene(scene);
            tm1_drive_step(
                self.state.as_mut_ptr(),
                opponent.map_or(std::ptr::null_mut(), |car| car.state.as_mut_ptr()),
                input.ps1_buttons(),
                fields.into(),
                self.frame,
                self.trig.as_ptr(),
                &mut self.seed,
            );
            tm1_use_scene(std::ptr::null_mut());
        }
    }
}

unsafe extern "C" {
    fn tm1_police_step(
        scene: *mut std::ffi::c_void,
        state: *mut u8,
        target: *mut u8,
        table: *const i16,
        seed: *mut u32,
        init: u8,
        fields: i16,
        frame: u32,
    );
}
pub struct PoliceCar {
    state: [u8; 0x560],
}
impl PoliceCar {
    pub fn spawn(player: &mut Taxi) -> Self {
        let mut car = Self { state: [0; 0x560] };
        car.update(player, 3, true, std::ptr::null_mut());
        car
    }
    pub fn step(&mut self, player: &mut Taxi, fields: u8) {
        assert!((1..=5).contains(&fields));
        self.update(player, fields, false, std::ptr::null_mut());
    }
    fn update(&mut self, player: &mut Taxi, fields: u8, init: bool, scene: *mut std::ffi::c_void) {
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_level(player.level as i32);
            tm1_police_step(
                scene,
                self.state.as_mut_ptr(),
                player.state.as_mut_ptr(),
                player.trig.as_ptr(),
                &mut player.seed,
                init as u8,
                fields.into(),
                player.frame,
            );
        }
    }
    pub fn position(&self) -> [i32; 3] {
        std::array::from_fn(|i| {
            let at = 0x444 + 4 * i;
            i32::from_le_bytes(self.state[at..at + 4].try_into().unwrap())
        })
    }
    pub fn rotation(&self) -> [[i16; 3]; 3] {
        std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                let at = 0x48c + 2 * (3 * i + j);
                i16::from_le_bytes(self.state[at..at + 2].try_into().unwrap())
            })
        })
    }
}

unsafe extern "C" {
    fn tm1_scene_load(
        data: *const u8,
        size: usize,
        base: u32,
        world: u32,
        taxi: u32,
        police: u32,
    ) -> *mut std::ffi::c_void;
    fn tm1_reset_projectiles(scene: *mut std::ffi::c_void);
    fn tm1_scene_free(scene: *mut std::ffi::c_void);
    fn tm1_contacts(
        scene: *mut std::ffi::c_void,
        player: *mut u8,
        ai: *mut u8,
        is_player: u8,
        tables: *const i16,
        seed: *mut u32,
        fields: i16,
        frame: u32,
    );
}
pub struct CollisionScene(std::ptr::NonNull<std::ffi::c_void>);
// The graph owns its allocations. Mutation requires exclusive access, and all
// calls that bind a scene to the original C globals hold the core mutex.
unsafe impl Send for CollisionScene {}
unsafe impl Sync for CollisionScene {}
impl CollisionScene {
    pub fn from_dmd(data: &[u8], world: u32, taxi: u32, police: u32) -> Option<Self> {
        let base = u32::from_le_bytes(data.get(12..16)?.try_into().ok()?);
        std::ptr::NonNull::new(unsafe {
            tm1_scene_load(data.as_ptr(), data.len(), base, world, taxi, police)
        })
        .map(Self)
    }
    pub fn reset_projectiles(&mut self) {
        unsafe {
            tm1_reset_projectiles(self.0.as_ptr());
        }
    }
    pub fn step(&mut self, player: &mut Taxi, opponent: &mut PoliceCar, input: Input, fields: u8) {
        self.phase(player, opponent, 2, fields);
        player.step_in_scene(input, fields, self.0.as_ptr(), Some(opponent));
        self.phase(player, opponent, 1, fields);
        opponent.update(player, fields, false, self.0.as_ptr());
        self.phase(player, opponent, 0, fields);
    }
    fn phase(&mut self, player: &mut Taxi, opponent: &mut PoliceCar, phase: u8, fields: u8) {
        assert!((1..=5).contains(&fields));
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_level(player.level as i32);
            tm1_contacts(
                self.0.as_ptr(),
                player.state.as_mut_ptr(),
                opponent.state.as_mut_ptr(),
                phase,
                player.trig.as_ptr(),
                &mut player.seed,
                fields.into(),
                player.frame,
            );
        }
    }
}
impl Drop for CollisionScene {
    fn drop(&mut self) {
        unsafe { tm1_scene_free(self.0.as_ptr()) }
    }
}

impl CollisionScene {
    pub fn world_point(&mut self, mut point: [i32; 3]) -> [i32; 4] {
        unsafe extern "C" {
            fn tm1_world_point(scene: *mut std::ffi::c_void, point: *mut i32, result: *mut i32);
        }
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        let mut result = [0; 4];
        unsafe {
            tm1_world_point(self.0.as_ptr(), point.as_mut_ptr(), result.as_mut_ptr());
        }
        result
    }
    pub fn flame_particles(&self, particles: &mut [[i32; 4]; 80]) -> usize {
        unsafe extern "C" {
            fn tm1_flame_particles(scene: *mut std::ffi::c_void, out: *mut [i32; 4]) -> usize;
        }
        unsafe { tm1_flame_particles(self.0.as_ptr(), particles.as_mut_ptr()) }
    }
    pub fn contrail_faces(&self, out: &mut [[i32; 14]; 180]) -> usize {
        unsafe extern "C" {
            fn tm1_contrail_faces(scene: *mut std::ffi::c_void, out: *mut [i32; 14]) -> usize;
        }
        unsafe { tm1_contrail_faces(self.0.as_ptr(), out.as_mut_ptr()) }
    }
    pub fn missile_poses(&self, out: &mut [[i32; 13]; 10]) -> usize {
        unsafe extern "C" {
            fn tm1_missile_poses(scene: *mut std::ffi::c_void, out: *mut [i32; 13]) -> usize;
        }
        unsafe { tm1_missile_poses(self.0.as_ptr(), out.as_mut_ptr()) }
    }
    pub fn taser_arcs(&self, arcs: &mut [[[i32; 3]; 16]; 3]) -> usize {
        unsafe extern "C" {
            fn tm1_taser_arcs(scene: *mut std::ffi::c_void, arcs: *mut [[i32; 3]; 16]) -> usize;
        }
        unsafe { tm1_taser_arcs(self.0.as_ptr(), arcs.as_mut_ptr()) }
    }
    pub fn bullet_segments(&self, segments: &mut [[[i32; 3]; 2]; 40]) -> usize {
        unsafe extern "C" {
            fn tm1_bullet_segments(scene: *mut std::ffi::c_void, out: *mut [[i32; 3]; 2]) -> usize;
        }
        unsafe { tm1_bullet_segments(self.0.as_ptr(), segments.as_mut_ptr()) }
    }
}

impl PlayerVehicle {
    pub fn body_stage(&self) -> usize {
        body_stage(self.health(), self.state[0x13f] as i8)
    }
    pub fn health(&self) -> (i32, i32) {
        (self.word(0x178), self.word(0x17c))
    }
    pub fn gun_heat(&self) -> i16 {
        i16::from_le_bytes(self.state[0x48e..0x490].try_into().unwrap())
    }
}
impl PoliceCar {
    pub fn body_stage(&self) -> usize {
        body_stage(self.health(), self.state[0x213] as i8)
    }
    pub fn health(&self) -> (i32, i32) {
        let word = |at: usize| i32::from_le_bytes(self.state[at..at + 4].try_into().unwrap());
        (word(0x24c), word(0x250))
    }
}

impl CollisionScene {
    pub fn effects(&self, out: &mut [[i32; 5]; 122]) -> usize {
        unsafe extern "C" {
            fn tm1_effects(scene: *mut std::ffi::c_void, out: *mut [i32; 5]) -> usize;
        }
        unsafe { tm1_effects(self.0.as_ptr(), out.as_mut_ptr()) }
    }
}
impl PlayerVehicle {
    pub fn missiles(&self) -> i16 {
        i16::from_le_bytes(self.state[0x464..0x466].try_into().unwrap())
    }
}

impl PlayerVehicle {
    pub fn rev(&self) -> i32 {
        let gear = self.state[0xa0 + 4] != 0;
        if gear && self.state[0xa0 + 13] != 0 {
            return 99;
        }
        (self.word(0x398).abs() * 49 / self.word(0x1e0).max(1) + if gear { 49 } else { 0 })
            .clamp(0, 99)
    }
}

#[cfg(test)]
#[path = "drive_reference.rs"]
mod reference;

impl CollisionScene {
    pub fn hidden_pickups(&self) -> u64 {
        unsafe extern "C" {
            fn tm1_hidden_pickups(scene: *mut std::ffi::c_void) -> u64;
        }
        unsafe { tm1_hidden_pickups(self.0.as_ptr()) }
    }
}

impl PlayerVehicle {
    pub fn weapon(&self) -> (&'static str, i16) {
        let names = [
            "Fire missile",
            "Freeze missile",
            "Homing missile",
            "Power missile",
            "Singing missile",
            "Rear missile",
            "Catapult",
            "Flamethrower",
            "Bomb",
            "Oil",
            "Spike",
            if self.vehicle == Vehicle::Hammerhead {
                "Crush (ram)"
            } else {
                "Special"
            },
        ];
        let slot = self.word(0x460) as usize;
        if slot >= names.len() {
            return ("Machine gun", 0);
        }
        let at = 0x464 + 2 * slot;
        (
            names[slot],
            i16::from_le_bytes(self.state[at..at + 2].try_into().unwrap()),
        )
    }
}

#[cfg(test)]
mod graph_tests {
    use super::CollisionScene;
    #[test]
    fn rejects_truncated_and_cyclic_collision_records() {
        let base = 0x8001_88b8u32;
        let node = base + 16;
        let mut data = [0u8; 44];
        data[12..16].copy_from_slice(&base.to_le_bytes());
        data[16] = 1;
        data[36] = 1;
        data[40..].copy_from_slice(&node.to_le_bytes());
        assert!(CollisionScene::from_dmd(&data, node, node, node).is_none());
        assert!(CollisionScene::from_dmd(&data[..38], node, node, node).is_none());
    }
}

// UA.c uses integer health percentages for the body switch on both cars.
fn body_stage((health, maximum): (i32, i32), threshold: i8) -> usize {
    let pct = health * 100 / maximum.max(1);
    if pct > i32::from(threshold) {
        0
    } else if pct >= 11 {
        1
    } else {
        2
    }
}
#[test]
fn vehicle_body_thresholds() {
    for (hp, max, expected) in [
        (100, 100, 0),
        (76, 100, 0),
        (75, 100, 1),
        (11, 100, 1),
        (10, 100, 2),
        (0, 100, 2),
        (83, 110, 1),
        (84, 110, 0),
    ] {
        assert_eq!(body_stage((hp, max), 75), expected);
    }
}
