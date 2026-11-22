//! Fixed-step vehicle, terrain, collision and projectile simulation for the JP arena.
unsafe extern "C" {
    fn tm1_use_scene(scene: *mut std::ffi::c_void);
    fn tm1_bind_level(level: i32);
    fn tm1_bind_difficulty(difficulty: i32);
    fn tm1_campaign_player(state: *mut u8, table: *const i16, seed: *mut u32, new_life: i32);
    fn tm1_new_life(scene: *mut std::ffi::c_void);
    fn tm1_player_init(state: *mut u8, seed: *mut u32, kind: i32);
    fn tm1_drive_step(
        state: *mut u8,
        opponent: *mut u8,
        buttons: u32,
        fields: i16,
        frame: u32,
        table: *const i16,
        seed: *mut u32,
    );
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}
impl Difficulty {
    pub fn next(self) -> Self {
        match self {
            Self::Easy => Self::Medium,
            Self::Medium => Self::Hard,
            Self::Hard => Self::Easy,
        }
    }
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Arena,
    Warehouse,
    Freeway,
    CityPark,
    Cyburbia,
    Rooftops,
    Arena5,
}

impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Self::Arena => "Arena",
            Self::Warehouse => "Warehouse",
            Self::Freeway => "Freeway",
            Self::CityPark => "City Park",
            Self::Cyburbia => "Cyburbia",
            Self::Rooftops => "Rooftops",
            Self::Arena5 => "Arena 5",
        }
    }
    /// The five controller glyphs used by the JP password screen.
    pub fn password(self) -> Option<&'static str> {
        match self {
            Self::Arena => None,
            Self::Warehouse => Some("@<=@@"),
            Self::Freeway => Some(">==@<"),
            Self::CityPark => Some("><=@="),
            Self::Cyburbia => Some(">=<<<"),
            Self::Rooftops => Some("=<>@>"),
            Self::Arena5 => Some("=<@=="),
        }
    }
    pub fn from_stage_password(code: &str) -> Option<Self> {
        [
            Self::Warehouse,
            Self::Freeway,
            Self::CityPark,
            Self::Cyburbia,
            Self::Rooftops,
            Self::Arena5,
        ]
        .into_iter()
        .find(|level| level.password() == Some(code))
    }
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Arena => Some(Self::Warehouse),
            Self::Warehouse => Some(Self::Freeway),
            Self::Freeway => Some(Self::CityPark),
            Self::CityPark => Some(Self::Cyburbia),
            Self::Cyburbia => Some(Self::Rooftops),
            Self::Rooftops | Self::Arena5 => None,
        }
    }
    pub fn opponents(self, player: Vehicle, deck: &mut ArenaDeck, seed: &mut u32) -> Vec<Vehicle> {
        match self {
            Self::Arena => vec![deck.draw(player, seed)],
            Self::Arena5 => (0..5).map(|_| deck.draw(player, seed)).collect(),
            Self::Warehouse => player.warehouse_opponents(),
            Self::Freeway => player.freeway_opponents(),
            Self::Rooftops => player.opponents_from_masks([
                0x038, 0x038, 0x038, 0x034, 0x02c, 0x01c, 0xe00, 0xe00, 0xe00, 0xc40, 0xa40, 0x640,
            ]),
            Self::Cyburbia => player.opponents_from_masks([
                0x7ec, 0xf95, 0xf93, 0x7e5, 0xf87, 0x7cd, 0x7ad, 0x76d, 0x6ed, 0x5ed, 0x3ed, 0x797,
            ]),
            Self::CityPark => player.opponents_from_masks([
                0xe32, 0x1cd, 0x1cb, 0x1c7, 0xe23, 0xe13, 0x18f, 0x14f, 0xcf, 0xc33, 0xa33, 0x633,
            ]),
        }
    }
    pub fn database(self, vehicle: Vehicle) -> &'static str {
        use Vehicle::*;
        match self {
            Self::Arena | Self::Arena5 => "XARENA1",
            Self::Rooftops => match vehicle {
                SweetTooth | YellowJacket | Darkside | Outlaw | CrimsonFury | Thumper => "ROOF1",
                _ => "ROOF2",
            },
            Self::Cyburbia => match vehicle {
                YellowJacket | Darkside | Thumper | RoadKill => "SUBURB2",
                _ => "SUBURB1",
            },
            Self::CityPark => match vehicle {
                SweetTooth | Hammerhead | CrimsonFury | Thumper | Spectre | RoadKill => "PARK1",
                _ => "PARK2",
            },
            Self::Freeway => match vehicle {
                SweetTooth | YellowJacket | Darkside | Hammerhead | Spectre | RoadKill => "XFWY1",
                _ => "XFWY2",
            },
            Self::Warehouse => match vehicle {
                SweetTooth | YellowJacket | Darkside | Outlaw | CrimsonFury | Thumper => "WH1",
                _ => "WH2",
            },
        }
    }
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
    Minion = 130,
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
    // UA card order differs from the DMD vehicle IDs.
    pub const CARD_ORDER: [Vehicle; 12] = [
        Self::SweetTooth,
        Self::YellowJacket,
        Self::Darkside,
        Self::Outlaw,
        Self::Thumper,
        Self::CrimsonFury,
        Self::PitViper,
        Self::Warthog,
        Self::MrGrimm,
        Self::Spectre,
        Self::Hammerhead,
        Self::RoadKill,
    ];
    pub fn sound_programs(self) -> (usize, usize) {
        use Vehicle::*;
        // UA_SOUND.c sound IDs resolved through SOUND.c's VAB program table.
        match self {
            SweetTooth => (1, 41),
            YellowJacket => (0, 43),
            Darkside => (2, 40),
            Hammerhead => (3, 45),
            Warthog => (5, 48),
            Spectre => (6, 50),
            RoadKill => (10, 44),
            Thumper => (8, 49),
            Outlaw => (7, 39),
            CrimsonFury => (4, 42),
            MrGrimm => (11, 47),
            PitViper => (9, 46),
            Minion => (55, 56),
        }
    }

    pub fn warehouse_opponents(self) -> Vec<Self> {
        const MASKS: [u16; 12] = [
            0x016, 0x034, 0x032, 0x016, 0x026, 0x016, 0x980, 0x940, 0x8c0, 0x1c0, 0x1c0, 0x1c0,
        ];
        self.opponents_from_masks(MASKS)
    }
    pub fn freeway_opponents(self) -> Vec<Self> {
        self.opponents_from_masks([
            0xe06, 0xe05, 0xe03, 0x1f0, 0x1e8, 0x1d8, 0x1b8, 0x178, 0x0f8, 0xc07, 0xa07, 0x607,
        ])
    }
    fn opponents_from_masks(self, masks: [u16; 12]) -> Vec<Self> {
        let mask = masks[Self::CARD_ORDER.iter().position(|v| *v == self).unwrap()];
        Self::CARD_ORDER
            .into_iter()
            .enumerate()
            .filter_map(|(bit, car)| (mask & (1 << bit) != 0).then_some(car))
            .collect()
    }
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
            Self::Minion => "minion",
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
            Self::Minion => "Minion",
            Self::Thumper => "Thumper",
        }
    }
}
/// Arena draws exclude the player and previously drawn cars until the deck resets.
#[derive(Default, Clone, Copy)]
pub struct ArenaDeck {
    used: u16,
    draws: u8,
}
/// Advance the shared PS1 BIOS rand state and return its 15-bit value.
pub fn bios_rand(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (*seed >> 16) & 32767
}
impl ArenaDeck {
    pub fn draw(&mut self, player: Vehicle, seed: &mut u32) -> Vehicle {
        let index = Vehicle::CARD_ORDER
            .iter()
            .position(|car| *car == player)
            .unwrap();
        if self.draws >= 11 || self.used & (1 << index) != 0 {
            *self = Self::default();
        }
        loop {
            let card = bios_rand(seed) as usize % 12;
            if card != index && self.used & (1 << card) == 0 {
                self.used |= 1 << card;
                self.draws += 1;
                return Vehicle::CARD_ORDER[card];
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RadarSignal {
    Contact { x: i32, y: i32, rgb: [u8; 3] },
    Up,
    Down,
    Right,
    Left,
}

pub type Taxi = PlayerVehicle;
pub struct PlayerVehicle {
    cheat_controls: bool,
    difficulty: Difficulty,
    level: Level,
    vehicle: Vehicle,
    trig: Box<[i16]>,
    initial_seed: u32,
    initial_state: [u8; 0x4a4],
    used_starts: u16,
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
        Self::seeded(exe, vehicle, level, 1)
    }
    pub fn seeded(exe: &[u8], vehicle: Vehicle, level: Level, initial_seed: u32) -> Option<Self> {
        Self::contest(exe, vehicle, level, initial_seed, Difficulty::Medium)
    }
    pub fn contest(
        exe: &[u8],
        vehicle: Vehicle,
        level: Level,
        initial_seed: u32,
        difficulty: Difficulty,
    ) -> Option<Self> {
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
        let mut seed = initial_seed;
        {
            let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                tm1_bind_difficulty(difficulty as i32);
                tm1_bind_level(level as i32);
                tm1_player_init(state.as_mut_ptr(), &mut seed, vehicle as i32)
            };
        }
        Some(Self {
            cheat_controls: false,
            difficulty,
            level,
            vehicle,
            state,
            frame: 0,
            trig,
            initial_seed: seed,
            initial_state: state,
            used_starts: 0,
            seed,
        })
    }
}
impl PlayerVehicle {
    pub fn set_weapon_and_damage_modes(&mut self, infinite_weapons: bool, god_mode: bool) {
        self.cheat_controls = god_mode;
        // UASetInfiniteWeapons and UASetGodMode update these fields on playerInfo.
        for record in [&mut self.state, &mut self.initial_state] {
            record[0x458 + 0x49] = u8::from(infinite_weapons);
            record[0x138 + 0x4c..0x138 + 0x50].copy_from_slice(&i32::from(!god_mode).to_le_bytes());
        }
    }
    pub fn reset(&mut self) {
        self.seed = self.initial_seed;
        self.frame = 0;
        self.used_starts = 0;
        self.state = self.initial_state;
    }
    pub fn for_next_stage(&self, level: Level) -> Self {
        self.for_next_stage_seeded(level, self.seed)
    }
    pub fn for_next_stage_seeded(&self, level: Level, mut seed: u32) -> Self {
        let mut state = self.state;
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(self.difficulty as i32);
            tm1_bind_level(level as i32);
            tm1_campaign_player(state.as_mut_ptr(), self.trig.as_ptr(), &mut seed, 0);
        }
        Self {
            cheat_controls: self.cheat_controls,
            difficulty: self.difficulty,
            state,
            initial_state: state,
            initial_seed: seed,
            seed,
            used_starts: 0,
            frame: 0,
            level,
            vehicle: self.vehicle,
            trig: self.trig.clone(),
        }
    }
    pub fn set_pad_layout(&mut self, layout: u8) {
        assert!(layout < 4);
        unsafe extern "C" {
            fn tm1_pad_layout(state: *mut u8, mode: i32);
        }
        unsafe {
            tm1_pad_layout(self.state.as_mut_ptr(), layout.into());
            tm1_pad_layout(self.initial_state.as_mut_ptr(), layout.into());
        }
    }
    /// Translate keyboard actions using the currently selected UA controller layout.
    pub fn keyboard_buttons(&self, input: Input) -> u16 {
        const BITS: [u8; 15] = [0, 8, 11, 2, 0, 12, 14, 15, 13, 3, 1, 4, 6, 7, 5];
        [
            (input.accelerate, 29),
            (input.reverse, 25),
            (input.left, 27),
            (input.right, 28),
            (input.hard_turn, 31),
            (input.turbo, 33),
            (input.fire_guns, 34),
            (input.missile, 35),
            (input.cycle_weapon, 36),
        ]
        .into_iter()
        .filter(|(pressed, _)| *pressed)
        .fold(0, |bits, (_, i)| {
            bits | 1 << BITS[self.state[0xc8 + i] as usize]
        })
    }
    pub fn cheat_controls(&self) -> bool {
        self.cheat_controls
    }
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }
    pub fn level(&self) -> Level {
        self.level
    }
    pub fn vehicle(&self) -> Vehicle {
        self.vehicle
    }
    pub fn random(&mut self) -> u32 {
        bios_rand(&mut self.seed)
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
    pub fn hud_opponent<'a>(&self, opponents: &'a [Opponent]) -> Option<&'a Opponent> {
        let [x, y, _] = self.position();
        let mut nearest = opponents.first()?;
        let mut best = 0x7fff;
        for ai in opponents.iter().filter(|ai| ai.health().0 > 0) {
            let [ax, ay, _] = ai.position();
            let distance = x
                .wrapping_sub(ax)
                .wrapping_abs()
                .wrapping_add(y.wrapping_sub(ay).wrapping_abs());
            if distance < best {
                nearest = ai;
                best = distance;
            }
        }
        nearest.is_active().then_some(nearest)
    }
    pub fn ground_group(&self) -> i16 {
        i16::from_le_bytes(self.state[0xc0..0xc2].try_into().unwrap())
    }
    pub fn accepts_view_input(&self) -> bool {
        self.state[0xab] == 0 && self.state[0xa0] == 0
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
    /// HUD.c projects horizontal world offsets into the 72 by 44 radar field.
    pub fn radar_offset(&self, position: [i32; 3]) -> [i32; 2] {
        let origin = self.position();
        let dx = position[0].wrapping_sub(origin[0]);
        let dy = position[1].wrapping_sub(origin[1]);
        let sin = |angle: i32| {
            let phase = (angle & 4095) as usize;
            let i = phase & 1023;
            let v = i32::from(self.trig[if phase & 1024 == 0 { i } else { 1024 - i }]);
            if phase & 2048 == 0 { v } else { -v }
        };
        let s = sin(self.heading());
        let c = sin(self.heading().wrapping_add(1024));
        let x = dx.wrapping_mul(c).wrapping_sub(dy.wrapping_mul(s)) / 0x14d555;
        let y = dx
            .wrapping_mul(s)
            .wrapping_add(dy.wrapping_mul(c))
            .wrapping_neg()
            / 0x14d555;
        [x, y]
    }
    pub fn radar_signal(&self, position: [i32; 3], vehicle: Vehicle) -> RadarSignal {
        let [x, y] = self.radar_offset(position);
        if (-36..36).contains(&x) && (-22..22).contains(&y) {
            use Vehicle::*;
            let rgb = match vehicle {
                SweetTooth => [255, 144, 186],
                YellowJacket => [255, 255, 0],
                Darkside | Minion => [0, 0, 0],
                Hammerhead => [0, 233, 0],
                Outlaw => [0, 0, 255],
                CrimsonFury => [255, 0, 0],
                Warthog => [130, 202, 54],
                MrGrimm => [255, 102, 0],
                PitViper => [199, 113, 0],
                Thumper => [255, 0, 255],
                Spectre => [255, 255, 255],
                RoadKill => [155, 155, 50],
            };
            RadarSignal::Contact { x, y, rgb }
        } else if !(-36..36).contains(&x) && x.abs() > y.abs() {
            if x < 0 {
                RadarSignal::Left
            } else {
                RadarSignal::Right
            }
        } else if y > 0 {
            RadarSignal::Down
        } else {
            RadarSignal::Up
        }
    }
    /// UA.c GetPlayerSpeed, in displayed MPH.
    pub fn speed_mph(&self, fields: u8) -> i32 {
        assert!((1..=5).contains(&fields));
        (self.word(0x398) / 32) * 100 / (i32::from(fields) * 19)
    }

    pub fn cockpit_steering_angle(&self, previous: i32) -> i32 {
        let skid = &self.state[0xc8..];
        let pct = if skid[6] != 0 || skid[7] != 0 { 40 } else { 70 };
        let target = if skid[2] != 0 {
            -28
        } else if skid[3] != 0 {
            28
        } else {
            0
        };
        (previous * pct + target * (100 - pct)) / 100
    }

    pub fn dashboard_enabled(&self) -> bool {
        self.word(0x174) != 0
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
            tm1_bind_difficulty(self.difficulty as i32);
            tm1_bind_level(self.level as i32);
            tm1_use_scene(scene);
            tm1_drive_step(
                self.state.as_mut_ptr(),
                opponent.map_or(std::ptr::null_mut(), |car| car.state.as_mut_ptr()),
                u32::from(input.ps1_buttons()),
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
    fn tm1_place_boss(boss: *mut u8, player: *mut u8, table: *const i16, seed: *mut u32, fov: i32);
    fn tm1_ai_step(
        scene: *mut std::ffi::c_void,
        state: *mut u8,
        target: *mut u8,
        table: *const i16,
        seed: *mut u32,
        kind: i32,
        spawn_points: *mut u16,
        fields: i16,
        frame: u32,
    );
}
pub type PoliceCar = Opponent;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum CarView {
    Cockpit = 1,
    Chase,
    Far,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerView {
    Car(CarView),
    Helicopter,
}

#[derive(Default)]
pub struct RooftopPassage {
    return_view: Option<PlayerView>,
}

impl RooftopPassage {
    /// CheckVRMode processes the pad before checking the current ground group.
    pub fn update(
        &mut self,
        group: i16,
        before: PlayerView,
        selected: PlayerView,
        fixed: bool,
    ) -> Option<PlayerView> {
        if group == 7 && self.return_view.is_none() {
            self.return_view = Some(before);
        }
        let saved = self.return_view.as_mut()?;
        if selected != before {
            *saved = selected;
        }
        if group == 5 {
            return self.return_view.take();
        }
        if !fixed
            && matches!(
                selected,
                PlayerView::Car(CarView::Far) | PlayerView::Helicopter
            )
        {
            *saved = before;
            return Some(PlayerView::Car(CarView::Chase));
        }
        None
    }
}

pub struct VehicleCamera {
    pub mode: CarView,
    pub offset: [i32; 3],
    pub pitch: i32,
    pub show_body: bool,
    pub fixed_pose: Option<[i32; 5]>,
}

impl VehicleCamera {
    pub fn new(player: &PlayerVehicle, mode: CarView) -> Self {
        let mut view = Self {
            mode,
            offset: [0; 3],
            pitch: 0,
            show_body: true,
            fixed_pose: None,
        };
        view.update(player, true);
        view
    }

    pub fn cycle(&mut self, forward: bool) {
        use CarView::*;
        self.mode = match (self.mode, forward) {
            (Cockpit, true) | (Far, false) => Chase,
            (Chase, true) | (Cockpit, false) => Far,
            (Far, true) | (Chase, false) => Cockpit,
        };
    }

    pub fn update(&mut self, player: &PlayerVehicle, snap: bool) {
        unsafe extern "C" {
            fn tm1_update_car_view(
                state: *const u8,
                mode: i32,
                snap: i32,
                offset: *mut i32,
                pitch: *mut i32,
            ) -> i32;
        }
        let position = player.position();
        let falling = player.level() == Level::Rooftops && position[2] < 6800;
        let leaving_fixed = !falling && self.fixed_pose.take().is_some();
        if leaving_fixed {
            self.mode = CarView::Chase;
        }
        if !falling || self.fixed_pose.is_none() {
            self.show_body = unsafe {
                tm1_update_car_view(
                    player.state.as_ptr(),
                    self.mode as i32,
                    (snap || leaving_fixed).into(),
                    self.offset.as_mut_ptr(),
                    &mut self.pitch,
                ) != 0
            };
        }
        if falling {
            unsafe extern "C" {
                fn tm1_rooftop_camera(
                    pos: *const i32,
                    heading: i32,
                    pitch: i32,
                    offset: *const i32,
                    table: *const i16,
                    enter: i32,
                    pose: *mut i32,
                );
            }
            let enter = self.fixed_pose.is_none();
            let pose = self.fixed_pose.get_or_insert([0; 5]);
            let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                tm1_rooftop_camera(
                    position.as_ptr(),
                    player.heading(),
                    self.pitch,
                    self.offset.as_ptr(),
                    player.trig.as_ptr(),
                    enter.into(),
                    pose.as_mut_ptr(),
                );
            }
            self.offset = [0; 3];
            self.pitch = 0;
            self.show_body = player.dashboard_enabled();
        }
    }
}

pub struct Helicopter {
    state: [u8; 0x560],
    /// World XYZ followed by JP pitch and yaw in 4096 units per turn.
    pub pose: [i32; 5],
}
impl Helicopter {
    pub fn new(player: &mut PlayerVehicle) -> Self {
        let mut heli = Self {
            state: [0; 0x560],
            pose: [0; 5],
        };
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            let mut seed = player.initial_seed;
            tm1_ai_step(
                std::ptr::null_mut(),
                heli.state.as_mut_ptr(),
                player.state.as_mut_ptr(),
                player.trig.as_ptr(),
                &mut seed,
                -1,
                &mut 0,
                3,
                player.frame,
            );
        }
        heli
    }
    pub fn update(&mut self, player: &PlayerVehicle, fields: u8, enter: bool) {
        assert!((1..=5).contains(&fields));
        unsafe extern "C" {
            fn tm1_heli_camera(
                state: *mut u8,
                target: *const u8,
                table: *const i16,
                fields: i16,
                frame: u32,
                enter: i32,
                pose: *mut i32,
            );
        }
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            tm1_heli_camera(
                self.state.as_mut_ptr(),
                player.state.as_ptr(),
                player.trig.as_ptr(),
                fields.into(),
                player.frame,
                enter.into(),
                self.pose.as_mut_ptr(),
            );
        }
    }
}
pub struct Opponent {
    vehicle: Vehicle,
    state: [u8; 0x560],
}
impl Opponent {
    pub fn vehicle(&self) -> Vehicle {
        self.vehicle
    }
    pub fn spawn(player: &mut Taxi) -> Self {
        Self::for_vehicle(player, Vehicle::Outlaw)
    }
    pub fn for_vehicle(player: &mut Taxi, vehicle: Vehicle) -> Self {
        let [car] = Self::spawn_lineup(player, [vehicle]);
        car
    }
    pub fn spawn_lineup<const N: usize>(player: &mut Taxi, vehicles: [Vehicle; N]) -> [Self; N] {
        assert!(N <= 8);
        let mut used = 0;
        std::array::from_fn(|slot| Self::spawn_slot(player, vehicles[slot], slot, &mut used))
    }
    pub fn spawn_bout(player: &mut Taxi, vehicles: &[Vehicle]) -> Vec<Self> {
        assert!((1..=8).contains(&vehicles.len()));
        let mut used = 0;
        let cars = vehicles
            .iter()
            .enumerate()
            .map(|(slot, &vehicle)| Self::spawn_slot(player, vehicle, slot, &mut used))
            .collect();
        player.used_starts = used;
        cars
    }
    pub fn spawn_minion(player: &mut Taxi) -> Self {
        assert_eq!(player.level, Level::Rooftops);
        let mut used = player.used_starts;
        let mut boss = Self::spawn_slot(player, Vehicle::Minion, 3, &mut used);
        player.used_starts = used;
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            tm1_place_boss(
                boss.state.as_mut_ptr(),
                player.state.as_mut_ptr(),
                player.trig.as_ptr(),
                &mut player.seed,
                60,
            );
        }
        boss
    }
    fn spawn_slot(player: &mut Taxi, vehicle: Vehicle, slot: usize, used: &mut u16) -> Self {
        let mut car = Self {
            vehicle,
            state: [0; 0x560],
        };
        car.state[6..8].copy_from_slice(&(slot as i16).to_le_bytes());
        car.update(player, 3, Some(used), std::ptr::null_mut());
        car
    }
    pub fn step(&mut self, player: &mut Taxi, fields: u8) {
        assert!((1..=5).contains(&fields));
        self.update(player, fields, None, std::ptr::null_mut());
    }
    fn update(
        &mut self,
        player: &mut Taxi,
        fields: u8,
        spawn_points: Option<&mut u16>,
        scene: *mut std::ffi::c_void,
    ) {
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            tm1_ai_step(
                scene,
                self.state.as_mut_ptr(),
                player.state.as_mut_ptr(),
                player.trig.as_ptr(),
                &mut player.seed,
                self.vehicle as i32,
                spawn_points.map_or(std::ptr::null_mut(), |p| p),
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
    fn tm1_active_opponents(scene: *mut std::ffi::c_void, count: i32);
    fn tm1_scene_load(
        data: *const u8,
        size: usize,
        base: u32,
        world: u32,
        taxi: u32,
        opponents: *const u32,
        count: usize,
    ) -> *mut std::ffi::c_void;
    fn tm1_level_won(scene: *const std::ffi::c_void) -> bool;
    fn tm1_reset_projectiles(scene: *mut std::ffi::c_void);
    fn tm1_scene_free(scene: *mut std::ffi::c_void);
    fn tm1_bout_step(
        scene: *mut std::ffi::c_void,
        player_state: *mut u8,
        opponents: *const *mut u8,
        count: i16,
        battle: *mut i16,
        buttons: u32,
        cheat_controls: u8,
        fields: i16,
        frame: u32,
        tables: *const i16,
        seed: *mut u32,
    );
    #[cfg(test)]
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
pub struct CollisionScene {
    native: std::ptr::NonNull<std::ffi::c_void>,
    opponent_count: usize,
    battle: [i16; 3],
}
// The graph owns its allocations. Mutation requires exclusive access, and all
// calls that bind a scene to the original C globals hold the core mutex.
unsafe impl Send for CollisionScene {}
unsafe impl Sync for CollisionScene {}
impl CollisionScene {
    pub fn from_dmd(data: &[u8], world: u32, taxi: u32, police: u32) -> Option<Self> {
        Self::for_opponents(data, world, taxi, &[police])
    }
    pub fn for_opponents(data: &[u8], world: u32, taxi: u32, opponents: &[u32]) -> Option<Self> {
        let base = u32::from_le_bytes(data.get(12..16)?.try_into().ok()?);
        std::ptr::NonNull::new(unsafe {
            tm1_scene_load(
                data.as_ptr(),
                data.len(),
                base,
                world,
                taxi,
                opponents.as_ptr(),
                opponents.len(),
            )
        })
        .map(|native| Self {
            native,
            opponent_count: opponents.len(),
            battle: [0; 3],
        })
    }
    pub fn continue_stage_state(&mut self, previous: &Self) {
        unsafe extern "C" {
            fn tm1_continue_stage_state(
                scene: *mut std::ffi::c_void,
                previous: *const std::ffi::c_void,
            );
        }
        unsafe { tm1_continue_stage_state(self.native.as_ptr(), previous.native.as_ptr()) }
    }
    pub fn carry_notice_frames(&self) -> i32 {
        unsafe extern "C" {
            fn tm1_carry_notice_frames(scene: *const std::ffi::c_void) -> i32;
        }
        unsafe { tm1_carry_notice_frames(self.native.as_ptr()) }
    }
    pub fn battle_music_playing(&self) -> bool {
        unsafe extern "C" {
            fn tm1_battle_music_playing(scene: *const std::ffi::c_void) -> i32;
        }
        unsafe { tm1_battle_music_playing(self.native.as_ptr()) != 0 }
    }
    pub fn pedestrian_models(&self) -> Vec<u32> {
        unsafe extern "C" {
            fn tm1_ped_model(scene: *const std::ffi::c_void, slot: i32) -> u32;
        }
        (0..6)
            .map(|i| unsafe { tm1_ped_model(self.native.as_ptr(), i) })
            .take_while(|&addr| addr != 0)
            .collect()
    }
    pub fn start_pedestrians(&mut self, level: Level, seed: &mut u32) {
        unsafe extern "C" {
            fn tm1_start_ped(
                scene: *mut std::ffi::c_void,
                slot: i32,
                level: i32,
                route: i32,
                end: i32,
            );
        }
        for (slot, _) in self.pedestrian_models().iter().enumerate() {
            let route = bios_rand(seed);
            let end = bios_rand(seed);
            unsafe {
                tm1_start_ped(
                    self.native.as_ptr(),
                    slot as i32,
                    level as i32,
                    route as i32,
                    end as i32,
                )
            };
        }
    }
    pub fn pedestrian_pose(&self, slot: usize) -> Option<[i32; 14]> {
        if slot >= 6 {
            return None;
        }
        unsafe extern "C" {
            fn tm1_ped_pose(scene: *const std::ffi::c_void, slot: i32, pose: *mut i32) -> i32;
        }
        let mut pose = [0; 14];
        (unsafe { tm1_ped_pose(self.native.as_ptr(), slot as i32, pose.as_mut_ptr()) } != 0)
            .then_some(pose)
    }
    pub fn merc_pose(&self, slot: usize) -> Option<[i32; 5]> {
        if slot >= 2 {
            return None;
        }
        unsafe extern "C" {
            fn tm1_merc_pose(scene: *const std::ffi::c_void, slot: i32, out: *mut i32) -> i32;
        }
        let mut pose = [0; 5];
        (unsafe { tm1_merc_pose(self.native.as_ptr(), slot as i32, pose.as_mut_ptr()) } != 0)
            .then_some(pose)
    }
    pub fn damage_branch(&self, address: u32) -> u8 {
        unsafe extern "C" {
            fn tm1_damage_branch(scene: *const std::ffi::c_void, address: u32) -> i32;
        }
        unsafe { tm1_damage_branch(self.native.as_ptr(), address) as u8 }
    }
    pub fn arena_switch_branches(&self) -> [u8; 8] {
        unsafe extern "C" {
            fn tm1_arena_switch_states(scene: *const std::ffi::c_void) -> u64;
        }
        unsafe { tm1_arena_switch_states(self.native.as_ptr()) }.to_le_bytes()
    }
    /// Active and selected DMD branch bits for the ten health stand slots.
    pub fn health_stands(&self) -> (u16, u16) {
        unsafe extern "C" {
            fn tm1_stand_state(scene: *const std::ffi::c_void) -> u32;
        }
        let state = unsafe { tm1_stand_state(self.native.as_ptr()) };
        (state as u16, (state >> 16) as u16)
    }
    pub fn respawn_player(&mut self, player: &mut PlayerVehicle) {
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            tm1_campaign_player(
                player.state.as_mut_ptr(),
                player.trig.as_ptr(),
                &mut player.seed,
                1,
            );
            tm1_new_life(self.native.as_ptr());
        }
        self.battle = [0; 3];
    }
    pub fn level_won(&self) -> bool {
        unsafe { tm1_level_won(self.native.as_ptr()) }
    }
    pub fn reset_projectiles(&mut self) {
        self.battle = [0; 3];
        unsafe {
            tm1_reset_projectiles(self.native.as_ptr());
        }
    }
    pub fn step(&mut self, player: &mut Taxi, opponent: &mut PoliceCar, input: Input, fields: u8) {
        self.step_bout(player, std::slice::from_mut(opponent), input, fields);
    }
    pub fn step_bout(
        &mut self,
        player: &mut Taxi,
        opponents: &mut [Opponent],
        input: Input,
        fields: u8,
    ) {
        self.step_bout_with_pads(player, opponents, u32::from(input.ps1_buttons()), fields)
    }

    pub fn step_bout_with_pads(
        &mut self,
        player: &mut Taxi,
        opponents: &mut [Opponent],
        pads: u32,
        fields: u8,
    ) {
        assert!((1..=5).contains(&fields));
        assert!(!opponents.is_empty() && opponents.len() <= self.opponent_count);
        let mut records = [std::ptr::null_mut(); 8];
        for (slot, car) in opponents.iter_mut().enumerate() {
            assert_eq!(
                i16::from_le_bytes([car.state[6], car.state[7]]),
                slot as i16
            );
            records[slot] = car.state.as_mut_ptr();
        }
        player.frame = player.frame.wrapping_add(1);
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            tm1_active_opponents(self.native.as_ptr(), opponents.len() as i32);
            tm1_bout_step(
                self.native.as_ptr(),
                player.state.as_mut_ptr(),
                records.as_ptr(),
                opponents.len() as i16,
                self.battle.as_mut_ptr(),
                pads,
                u8::from(player.cheat_controls),
                fields.into(),
                player.frame,
                player.trig.as_ptr(),
                &mut player.seed,
            );
        }
    }
    #[cfg(test)]
    fn phase(&mut self, player: &mut Taxi, opponent: &mut PoliceCar, phase: u8, fields: u8) {
        assert!((1..=5).contains(&fields));
        let _guard = super::TIMER.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            tm1_bind_difficulty(player.difficulty as i32);
            tm1_bind_level(player.level as i32);
            tm1_contacts(
                self.native.as_ptr(),
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
        unsafe { tm1_scene_free(self.native.as_ptr()) }
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
            tm1_world_point(
                self.native.as_ptr(),
                point.as_mut_ptr(),
                result.as_mut_ptr(),
            );
        }
        result
    }
    pub fn flame_particles(&self, particles: &mut [[i32; 4]; 80]) -> usize {
        unsafe extern "C" {
            fn tm1_flame_particles(scene: *mut std::ffi::c_void, out: *mut [i32; 4]) -> usize;
        }
        unsafe { tm1_flame_particles(self.native.as_ptr(), particles.as_mut_ptr()) }
    }
    pub fn contrail_faces(&self, out: &mut [[i32; 14]; 180]) -> usize {
        unsafe extern "C" {
            fn tm1_contrail_faces(scene: *mut std::ffi::c_void, out: *mut [i32; 14]) -> usize;
        }
        unsafe { tm1_contrail_faces(self.native.as_ptr(), out.as_mut_ptr()) }
    }
    pub fn missile_poses(&self, out: &mut [[i32; 13]; 10]) -> usize {
        unsafe extern "C" {
            fn tm1_missile_poses(scene: *mut std::ffi::c_void, out: *mut [i32; 13]) -> usize;
        }
        unsafe { tm1_missile_poses(self.native.as_ptr(), out.as_mut_ptr()) }
    }
    pub fn taser_arcs(&self, arcs: &mut [[[i32; 3]; 16]; 3]) -> usize {
        unsafe extern "C" {
            fn tm1_taser_arcs(scene: *mut std::ffi::c_void, arcs: *mut [[i32; 3]; 16]) -> usize;
        }
        unsafe { tm1_taser_arcs(self.native.as_ptr(), arcs.as_mut_ptr()) }
    }
    pub fn bullet_segments(&self, segments: &mut [[[i32; 3]; 2]; 40]) -> usize {
        unsafe extern "C" {
            fn tm1_bullet_segments(scene: *mut std::ffi::c_void, out: *mut [[i32; 3]; 2]) -> usize;
        }
        unsafe { tm1_bullet_segments(self.native.as_ptr(), segments.as_mut_ptr()) }
    }
}

impl PlayerVehicle {
    pub fn body_stage(&self) -> usize {
        body_stage(self.health(), self.state[0x13f] as i8)
    }
    pub fn health(&self) -> (i32, i32) {
        (self.word(0x178), self.word(0x17c))
    }
    pub fn dashboard_weapon(&self) -> (usize, i16, i16) {
        let slot = self.word(0x460) as usize;
        let ammo = |i: usize| {
            i16::from_le_bytes(self.state[0x464 + 2 * i..0x466 + 2 * i].try_into().unwrap())
        };
        let special = ammo(11) / i16::from(self.state[0x4a0].max(1));
        (
            slot,
            match slot {
                0..=10 => ammo(slot),
                11 => special,
                _ => 0,
            },
            special,
        )
    }
    pub fn turbo_charge(&self) -> i32 {
        let tens = i16::from_le_bytes(self.state[0x49e..0x4a0].try_into().unwrap());
        (i32::from(tens) * 10 + self.word(0x230) / 18).clamp(0, 99)
    }
    pub fn gun_overheated(&self) -> bool {
        self.state[0x490] != 0
    }
    pub fn windshield_damage(pct: i32) -> (u8, usize) {
        match pct {
            ..10 => (0, 6),
            10..30 => (1, 3),
            30..50 => (2, 2),
            _ => (3, 0),
        }
    }
    pub fn windshield_holes(&mut self) -> [[i32; 2]; 6] {
        std::array::from_fn(|_| {
            [
                (self.random() % 295) as i32 + 20,
                (self.random() % 105) as i32 + 50,
            ]
        })
    }
    pub fn gun_heat(&self) -> i16 {
        i16::from_le_bytes(self.state[0x48e..0x490].try_into().unwrap())
    }
}
impl Opponent {
    pub fn is_active(&self) -> bool {
        self.state[0x41] != 0
    }
    pub fn health_tier(&self) -> u8 {
        self.state[2]
    }
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
        unsafe { tm1_effects(self.native.as_ptr(), out.as_mut_ptr()) }
    }
}
impl PlayerVehicle {
    pub fn missiles(&self) -> i16 {
        i16::from_le_bytes(self.state[0x464..0x466].try_into().unwrap())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactVoice {
    LightCrash,
    MediumCrash,
    HeavyCrash,
    Skid,
}
impl ContactVoice {
    pub const PROGRAMS: [usize; 4] = [30, 31, 32, 35];
}

fn contact_sounds(skid: bool, impact: i16) -> [Option<(ContactVoice, i32)>; 2] {
    let impact = i32::from(impact);
    let strength = impact + if impact >= 51 { 20 } else { 0 };
    let crash = if strength < 34 {
        (ContactVoice::LightCrash, strength * 3)
    } else if strength < 67 {
        (ContactVoice::MediumCrash, strength * 50 / 66 + 49)
    } else {
        (ContactVoice::HeavyCrash, strength * 50 / 99 + 49)
    };
    [
        skid.then_some((ContactVoice::Skid, 99)),
        (impact != 0).then_some(crash),
    ]
}

impl PlayerVehicle {
    pub fn contact_sounds(&self) -> [Option<(ContactVoice, i32)>; 2] {
        contact_sounds(
            self.state[0xa2] != 0,
            i16::from_le_bytes([self.state[0x26a], self.state[0x26b]]),
        )
    }
}
impl Opponent {
    pub fn contact_sounds(&self) -> [Option<(ContactVoice, i32)>; 2] {
        if self.state[0x40] == 0 {
            return [None; 2];
        }
        contact_sounds(
            self.state[0x68] != 0,
            i16::from_le_bytes([self.state[0x15a], self.state[0x15b]]),
        )
    }
}

fn engine_rev(speed: i32, gear_speed: i32, gear: u8, turbo: u8) -> i32 {
    if gear != 0 && turbo != 0 {
        return 99;
    }
    (speed.abs() * 49 / gear_speed.max(1) + if gear != 0 { 49 } else { 0 }).clamp(0, 99)
}
impl PlayerVehicle {
    pub fn rev(&self) -> i32 {
        engine_rev(
            self.word(0x398),
            self.word(0x1e0),
            self.state[0xa4],
            self.state[0xad],
        )
    }
}
impl Opponent {
    pub fn engine_rev(&self) -> Option<i32> {
        let word = |offset| i32::from_le_bytes(self.state[offset..offset + 4].try_into().unwrap());
        (self.state[0x40] != 0)
            .then(|| engine_rev(word(0x454), word(0x2b4), self.state[0x6a], self.state[0x73]))
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
        unsafe { tm1_hidden_pickups(self.native.as_ptr()) }
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
