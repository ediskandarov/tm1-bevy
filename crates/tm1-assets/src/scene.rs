//! Read-only DMD extraction. No game state, collision, or animation is simulated here.
use crate::{bytes, u16le, u32le};
use anyhow::{Result, bail, ensure};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TextureKey {
    pub page: u16,
    pub clut: u16,
    pub window: u32,
}
#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}
#[derive(Clone, Copy, Debug)]
pub struct SpriteFrame {
    pub texture: TextureKey,
    pub uv: [[u8; 2]; 4],
    pub width: u8,
}
pub struct DashboardIcons {
    pub weapons: [SpriteFrame; 12],
    pub labels: [SpriteFrame; 5],
    pub digits: [SpriteFrame; 10],
    pub turbo_digits: [SpriteFrame; 10],
    pub life: SpriteFrame,
    pub hole: SpriteFrame,
}
impl DashboardIcons {
    pub fn frames(&self) -> impl Iterator<Item = (&SpriteFrame, bool)> {
        self.weapons
            .iter()
            .chain([&self.life, &self.hole])
            .map(|f| (f, true))
            .chain(
                self.labels
                    .iter()
                    .chain(&self.digits)
                    .chain(&self.turbo_digits)
                    .map(|f| (f, false)),
            )
    }
}
/// One branch of a looping HIER animation, with its initial DMD clock.
#[derive(Clone, Copy, Debug)]
pub struct AnimationFrame {
    pub node: u32,
    pub child: u8,
    lo: u8,
    hi: u8,
    initial: u8,
    rate: u16,
    timer: u16,
}
impl AnimationFrame {
    pub fn selected(self, visits: u32) -> u8 {
        if self.rate == u16::MAX {
            return self.initial;
        }
        let changes = (u64::from(visits) + u64::from(self.timer)) / (u64::from(self.rate) + 1);
        let count = u64::from(self.hi - self.lo) + 1;
        self.lo + ((u64::from(self.initial - self.lo) + changes) % count) as u8
    }
}
#[derive(Clone, Debug)]
pub struct Polygon {
    pub animation: Vec<AnimationFrame>,
    pub billboard: Option<[f32; 2]>,
    pub pickup: Option<u8>,
    pub arena_switches: Vec<(u8, u8)>,
    pub health_stand: Option<(u8, u8)>,
    pub object_id: u32,
    pub source_offset: usize,
    pub vertices: Vec<Vertex>,
    pub texture: Option<TextureKey>,
    pub double_sided: bool,
    pub semi_transparent: bool,
    pub fixed_ot: u32,
}
#[derive(Debug)]
struct VisibilityBox {
    matrix: [[i16; 3]; 3],
    lo: [i16; 3],
    hi: [i16; 3],
}
impl VisibilityBox {
    fn contains(&self, point: [i32; 3]) -> bool {
        (0..3).all(|i| {
            let projected = (0..3).fold(0i32, |sum, j| {
                sum.wrapping_add(point[j].wrapping_mul(i32::from(self.matrix[i][j])))
            }) / 4096;
            let distance = projected + i32::from(self.lo[i]);
            distance > 0 && distance < i32::from(self.hi[i])
        })
    }
}

#[derive(Debug)]
pub struct VisibilityController {
    pub node: u32,
    transform: Transform,
    boxes: Vec<VisibilityBox>,
    branches: [std::ops::Range<usize>; 2],
}
impl VisibilityController {
    /// HIER selects one child when the camera is inside any of the control boxes.
    pub fn camera_inside(&self, eye: [f32; 3]) -> bool {
        let point = std::array::from_fn(|i| {
            (0..3)
                .map(|j| self.transform.axes[j][i] * (eye[j] - self.transform.offset[j]))
                .sum::<f32>() as i32
        });
        self.boxes.iter().any(|b| b.contains(point))
    }
    pub fn hidden_polygons(&self, eye: [f32; 3]) -> std::ops::Range<usize> {
        self.branches[usize::from(!self.camera_inside(eye))].clone()
    }
}

#[derive(Debug)]
pub struct Scene {
    pub version: u32,
    pub root_id: u32,
    pub polygons: Vec<Polygon>,
    pub object_ids: Vec<u32>,
    pub omitted_nodes: Vec<String>,
    pub visibility: Vec<VisibilityController>,
    pub health_stands: Vec<(u8, [i32; 3])>,
}

#[derive(Clone, Copy, Debug)]
struct Transform {
    axes: [[f32; 3]; 3],
    offset: [f32; 3],
}
impl Transform {
    const IDENTITY: Self = Self {
        axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        offset: [0.; 3],
    };
    fn rotate(self, point: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|i| (0..3).map(|j| self.axes[i][j] * point[j]).sum())
    }
    fn apply(self, point: [f32; 3]) -> [f32; 3] {
        let rotated = self.rotate(point);
        std::array::from_fn(|i| self.offset[i] + rotated[i])
    }
}

#[derive(Clone, Copy)]
struct BodySelection {
    switch: Option<(u32, u8)>,
    headlights_kind: u16,
    headlights: bool,
}

pub struct Dmd<'a> {
    data: &'a [u8],
    base: u32,
    pub version: u32,
    pub roots: Vec<(u32, u16)>,
}
impl<'a> Dmd<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        ensure!(
            u32le(data, 0)? == 0x50535844 && u32le(data, 4)? == 0x43,
            "unsupported DMD header"
        );
        let mut d = Self {
            data,
            base: u32le(data, 12)?,
            version: u32le(data, 8)?,
            roots: Vec::new(),
        };
        let n = u32le(data, 20)? as usize;
        bytes(
            data,
            24,
            n.checked_mul(4)
                .ok_or_else(|| anyhow::anyhow!("root count overflow"))?,
        )?;
        for i in 0..n {
            let ptr = u32le(data, 24 + i * 4)?;
            let p = d.offset(ptr)?;
            d.roots.push((ptr, u16le(data, p + 2)?));
        }
        Ok(d)
    }
    fn offset(&self, ptr: u32) -> Result<usize> {
        let offset = ptr
            .checked_sub(self.base)
            .ok_or_else(|| anyhow::anyhow!("PS1 pointer {ptr:#x} below DMD base"))?
            as usize;
        bytes(self.data, offset, 4)?;
        Ok(offset)
    }
    pub fn extract_kind(&self, kind: u16) -> Result<Scene> {
        self.extract(kind, None)
    }
    /// Read the 16 Aburst frames used by explodeStoreExplosionAnimation.
    pub fn explosion_frames(&self) -> Result<[SpriteFrame; 16]> {
        self.sprite_frames(654, true)
    }
    pub fn flame_frames(&self) -> Result<[SpriteFrame; 10]> {
        self.sprite_frames(650, true)
    }
    pub fn contrail_frames(&self) -> Result<[SpriteFrame; 8]> {
        self.sprite_frames(652, true)
    }
    /// HUD.c uses four individual 2D sprites in up, down, right, left order.
    pub fn radar_arrows(&self) -> Result<[SpriteFrame; 4]> {
        let icon = |kind| self.sprite_frames::<1>(kind, false).map(|[frame]| frame);
        Ok([icon(1290)?, icon(1292)?, icon(1291)?, icon(1293)?])
    }
    fn sprite_frames<const N: usize>(&self, kind: u16, additive: bool) -> Result<[SpriteFrame; N]> {
        let p = self.sprite_node(0, kind)?;
        self.read_sprite_frames(p, additive, N)
    }
    pub fn dashboard_icons(&self) -> Result<DashboardIcons> {
        let group = self.sprite_node(1, 1260)?;
        ensure!(
            bytes(self.data, group + 20, 1)?[0] >= 8,
            "incomplete dashboard group"
        );
        let child = |i: usize| self.offset(u32le(self.data, group + 24 + i * 4)?);
        Ok(DashboardIcons {
            weapons: self.read_sprite_frames(child(0)?, false, 13)?,
            labels: self.read_sprite_frames(child(2)?, false, 5)?,
            digits: self.read_sprite_frames(child(3)?, false, 10)?,
            turbo_digits: self.read_sprite_frames(child(4)?, false, 10)?,
            life: self.read_sprite_frames::<1>(child(6)?, false, 1)?[0],
            hole: self.read_sprite_frames::<1>(child(7)?, false, 4)?[0],
        })
    }
    fn read_sprite_frames<const N: usize>(
        &self,
        p: usize,
        additive: bool,
        available: usize,
    ) -> Result<[SpriteFrame; N]> {
        ensure!(
            self.data[p] == 0 && u32le(self.data, p + 16)? as usize == available && N <= available,
            "invalid sprite frame count"
        );
        let mut poly = self.offset(u32le(self.data, p + 12)?)?;
        let mut frames = Vec::with_capacity(N);
        for _ in 0..N {
            let header = bytes(self.data, poly, 16)?;
            let len = usize::from(header[2]) * 4;
            let record = bytes(self.data, poly, len)?;
            let flags = u32le(record, 12)?;
            let uv = bytes(record, 16 + 4 * ((flags >> 28) & 7) as usize, 16)?;
            let width = uv[12].wrapping_sub(uv[8]);
            let height = uv[13].wrapping_sub(uv[5]);
            let (u, v) = (uv[0], uv[1]);
            let page = u16le(uv, 6)?;
            frames.push(SpriteFrame {
                texture: TextureKey {
                    page: if additive { (page & 31) | 32 } else { page },
                    clut: u16le(uv, 2)?,
                    window: 0,
                },
                width,
                uv: [
                    [u, v],
                    [u.wrapping_add(width), v],
                    [u, v.wrapping_add(height)],
                    [u.wrapping_add(width), v.wrapping_add(height)],
                ],
            });
            poly += len;
        }
        frames
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid sprite frames"))
    }
    /// SCREEN.c screenInitCarOccupants uses 64 × 90 portraits in card order.
    pub fn occupant_portraits(&self, textures: &crate::texture::Vram) -> Result<Vec<Vec<u8>>> {
        let p = self.sprite_node(0, 1300)?;
        ensure!(u32le(self.data, p + 16)? == 12, "invalid occupant count");
        let mut poly = self.offset(u32le(self.data, p + 12)?)?;
        let mut portraits = Vec::new();
        for _ in 0..12 {
            let len = usize::from(bytes(self.data, poly + 2, 1)?[0]) * 4;
            let record = bytes(self.data, poly, len)?;
            let uv = bytes(
                record,
                16 + 4 * ((u32le(record, 12)? >> 28) & 7) as usize,
                16,
            )?;
            let key = TextureKey {
                page: u16le(uv, 6)?,
                clut: u16le(uv, 2)?,
                window: 0,
            };
            let (u, v) = (uv[8], uv[1]);
            ensure!(
                uv[0].checked_sub(u) == Some(63) && uv[5].checked_sub(v) == Some(89),
                "unexpected occupant rectangle"
            );
            let mut portrait = Vec::with_capacity(64 * 90 * 4);
            for y in 0..90 {
                for x in 0..64 {
                    let mut color = textures.sample_primitive(key, u + x, v + y, true)?;
                    for c in &mut color[..3] {
                        *c = (u16::from(*c) * 191 / 128).min(255) as u8;
                    }
                    portrait.extend(color);
                }
            }
            portraits.push(portrait);
            poly += len;
        }
        Ok(portraits)
    }
    pub fn cockpit_rectangles(&self) -> Result<[[i32; 4]; 7]> {
        let p = self.sprite_node(0, 1240)?;
        ensure!(
            u32le(self.data, p + 16)? == 7,
            "unexpected dashboard panel count"
        );
        let verts = self.offset(u32le(self.data, p + 4)?)?;
        let mut poly = self.offset(u32le(self.data, p + 12)?)?;
        let mut rects = [[0; 4]; 7];
        for rect in &mut rects {
            let coord = |i: usize, axis: usize| -> Result<i32> {
                let idx = usize::from(u16le(self.data, poly + 4 + 2 * i)?);
                Ok(i32::from(
                    u16le(self.data, verts + 8 * idx + 2 * axis)? as i16
                ))
            };
            let x = coord(0, 0)?;
            let y = coord(0, 1)?;
            *rect = [
                x / 8,
                -y / 8,
                (coord(1, 0)? - x) / 8,
                (y - coord(2, 1)?) / 8,
            ];
            poly += 4 * usize::from(bytes(self.data, poly + 2, 1)?[0]);
        }
        Ok(rects)
    }
    fn sprite_node(&self, op: u8, kind: u16) -> Result<usize> {
        let mut pending: Vec<_> = self
            .roots
            .iter()
            .filter(|r| r.1 == 0)
            .map(|r| r.0)
            .collect();
        let mut seen = std::collections::HashSet::new();
        while let Some(ptr) = pending.pop() {
            if !seen.insert(ptr) {
                continue;
            }
            let p = self.offset(ptr)?;
            if self.data[p] == op && u16le(self.data, p + 2)? == kind {
                return Ok(p);
            }
            let (count_at, children_at) = match self.data[p] {
                1 => (20, 24),
                4 => (18, 20),
                _ => continue,
            };
            let count = bytes(self.data, p + count_at, 1)?[0] as usize;
            for i in 0..count {
                pending.push(u32le(self.data, p + children_at + 4 * i)?);
            }
        }
        bail!("missing sprite animation {kind}")
    }
    /// Read the three body-switch branches selected by UA's damage update.
    pub fn vehicle_bodies(&self, kind: u16) -> Result<[Scene; 3]> {
        self.vehicle_bodies_with_lights(kind, true)
    }
    pub fn vehicle_bodies_with_lights(&self, kind: u16, headlights: bool) -> Result<[Scene; 3]> {
        Ok([
            self.extract(kind, Some((Some(0), headlights)))?,
            self.extract(kind, Some((Some(1), headlights)))?,
            self.extract(kind, Some((Some(2), headlights)))?,
        ])
    }
    pub fn garage_vehicle(&self, kind: u16) -> Result<Scene> {
        self.extract(kind, Some((None, false)))
    }
    fn extract(&self, kind: u16, body: Option<(Option<u8>, bool)>) -> Result<Scene> {
        let roots: Vec<_> = self.roots.iter().filter(|(_, k)| *k == kind).collect();
        ensure!(
            roots.len() == 1,
            "expected exactly one root of kind {kind}, found {}",
            roots.len()
        );
        let root_id = roots[0].0;
        let mut selection = None;
        if let Some((stage, headlights)) = body {
            // uaswInitCarSwitch follows child zero through two groups.
            let switch = if let Some(stage) = stage {
                let mut node = root_id;
                for _ in 0..2 {
                    let offset = self.offset(node)?;
                    ensure!(
                        bytes(self.data, offset, 24)?[0] == 1 && self.data[offset + 20] > 0,
                        "missing vehicle body group"
                    );
                    node = u32le(self.data, offset + 24)?;
                }
                ensure!(
                    self.data[self.offset(node)?] == 9,
                    "missing vehicle body switch"
                );
                Some((node, stage))
            } else {
                None
            };
            selection = Some(BodySelection {
                switch,
                headlights_kind: kind + 4,
                headlights,
            });
        }
        let mut scene = Scene {
            version: self.version,
            root_id,
            polygons: vec![],
            visibility: vec![],
            health_stands: vec![],
            object_ids: vec![],
            omitted_nodes: vec![],
        };
        self.visit(
            root_id,
            Transform::IDENTITY,
            selection,
            None,
            &mut Vec::new(),
            &mut scene,
        )?;
        ensure!(
            !scene.polygons.is_empty(),
            "selected root has no supported polygons"
        );
        Ok(scene)
    }
    fn visit(
        &self,
        ptr: u32,
        transform: Transform,
        selection: Option<BodySelection>,
        mut pickup: Option<u8>,
        ancestors: &mut Vec<u32>,
        scene: &mut Scene,
    ) -> Result<()> {
        ensure!(
            ancestors.len() < 128 && !ancestors.contains(&ptr),
            "cyclic/deep DMD graph at {ptr:#x}"
        );
        let p = self.offset(ptr)?;
        ancestors.push(ptr);
        let d = self.data;
        let mut children = Vec::new();
        let mut transform = transform;
        let mut list = |count: usize, start: usize| -> Result<()> {
            bytes(
                d,
                start,
                count
                    .checked_mul(4)
                    .ok_or_else(|| anyhow::anyhow!("child count overflow"))?,
            )?;
            for i in 0..count {
                children.push(u32le(d, start + 4 * i)?);
            }
            Ok(())
        };
        match d[p] {
            0 => self.object(ptr, p, transform, pickup, scene)?,
            1 => {
                bytes(d, p, 24)?;
                if d[p + 21] & 2 == 0 {
                    let kind = u16le(d, p + 2)?;
                    let sub = u16le(d, p + 22)?;
                    let count =
                        match selection {
                            Some(body)
                                if body.headlights_kind == kind && (1..=3).contains(&sub) =>
                            {
                                if body.headlights { 2 } else { 0 }
                            }
                            _ => usize::from(d[p + 20]),
                        };
                    list(count, p + 24)?;
                }
            }
            2 => {
                // Static inspection chooses the nearest LOD; it does not implement game LOD selection.
                let n = u32le(d, p + 16)? as usize;
                bytes(
                    d,
                    p + 20,
                    n.checked_mul(4)
                        .ok_or_else(|| anyhow::anyhow!("LOD count overflow"))?,
                )?;
                let mut choice = None;
                for i in 0..n {
                    let r = self.offset(u32le(d, p + 20 + 4 * i)?)?;
                    let near = u32le(d, r + 4)?;
                    if choice.is_none_or(|(best, _)| near < best) {
                        choice = Some((near, u32le(d, r + 8)?));
                    }
                }
                if let Some((_, child)) = choice {
                    children.push(child);
                }
            }
            3 => {
                bytes(d, p, 16)?;
                ensure!(d[p + 12] < d[p + 11], "animation child out of range");
                let (lo, hi, initial) = (d[p + 8], d[p + 9], d[p + 12]);
                let (rate, timer) = (u16le(d, p + 6)?, u16le(d, p + 14)?);
                if d[p + 10] == 1
                    || !(lo <= initial && initial <= hi && hi < d[p + 11] && timer <= rate)
                {
                    children.push(u32le(d, p + 16 + 4 * usize::from(initial))?);
                    scene
                        .omitted_nodes
                        .push(format!("animation {ptr:#x}: initial frame only"));
                } else {
                    for child in lo..=hi {
                        let node = u32le(d, p + 16 + 4 * usize::from(child))?;
                        let start = scene.polygons.len();
                        self.visit(node, transform, selection, pickup, ancestors, scene)?;
                        for polygon in &mut scene.polygons[start..] {
                            polygon.animation.push(AnimationFrame {
                                node: ptr,
                                child,
                                lo,
                                hi,
                                initial,
                                rate,
                                timer,
                            });
                        }
                    }
                }
            }
            4 => {
                bytes(d, p, 20)?;
                if d[p + 19] & 2 == 0 {
                    let mut offset = [0.; 3];
                    for (axis, value) in offset.iter_mut().enumerate() {
                        *value = u32le(d, p + 4 + 4 * axis)? as i32 as f32;
                    }
                    if (401..=450).contains(&u16le(d, p + 2)?)
                        && (1..=50).contains(&u16le(d, p + 16)?)
                    {
                        pickup = Some((u16le(d, p + 16)? - 1) as u8);
                    }
                    transform.offset = transform.apply(offset);
                    list(d[p + 18] as usize, p + 20)?;
                }
            }
            5 => {
                bytes(d, p, 40)?;
                if d[p + 39] & 2 == 0 {
                    let parent = transform;
                    let mut offset = [0.; 3];
                    for (axis, value) in offset.iter_mut().enumerate() {
                        *value = u32le(d, p + 24 + 4 * axis)? as i32 as f32;
                        let mut column = [0.; 3];
                        for (row, c) in column.iter_mut().enumerate() {
                            *c = u16le(d, p + 4 + 2 * (axis * 3 + row))? as i16 as f32 / 4096.;
                        }
                        for (row, c) in parent.rotate(column).into_iter().enumerate() {
                            transform.axes[row][axis] = c;
                        }
                    }
                    transform.offset = parent.apply(offset);
                    list(d[p + 38] as usize, p + 40)?;
                }
            }
            9 => {
                bytes(d, p, 8)?;
                let child = selection
                    .and_then(|body| body.switch)
                    .filter(|(node, _)| *node == ptr)
                    .map_or(d[p + 7], |(_, stage)| stage);
                ensure!(child < d[p + 6], "switch child out of range");
                let kind = u16le(d, p + 2)?;
                if matches!(kind, 700 | 1016 | 1018) || (kind == 920 && d[p + 6] == 2) {
                    let id = u16le(d, p + 4)?;
                    let slot = match kind {
                        1016 => 4,
                        1018 => 5,
                        _ => {
                            ensure!(
                                (1..=if kind == 700 { 4 } else { 10 }).contains(&id),
                                "invalid switch ID"
                            );
                            (id - 1) as u8
                        }
                    };
                    if kind == 920 {
                        scene
                            .health_stands
                            .push((slot, transform.offset.map(|x| x as i32)));
                    }
                    for branch in 0..usize::from(d[p + 6]) {
                        let start = scene.polygons.len();
                        self.visit(
                            u32le(d, p + 8 + 4 * branch)?,
                            transform,
                            selection,
                            pickup,
                            ancestors,
                            scene,
                        )?;
                        for poly in &mut scene.polygons[start..] {
                            let state = (slot, branch as u8);
                            if kind != 920 {
                                poly.arena_switches.push(state);
                            } else {
                                poly.health_stand = Some(state);
                            }
                        }
                    }
                } else {
                    children.push(u32le(d, p + 8 + 4 * child as usize)?);
                }
            }
            6..=8 | 10 => {}
            11 => {
                bytes(d, p, 20)?;
                if u32le(d, p + 4)? & 2 == 0 {
                    let mut boxes = Vec::new();
                    for i in 0..usize::from(d[p + 10]) {
                        let offset = self.offset(u32le(d, p + 20 + i * 4)?)?;
                        let data = bytes(d, offset, 48)?;
                        let half = |i: usize| i16::from_le_bytes([data[i], data[i + 1]]);
                        boxes.push(VisibilityBox {
                            matrix: std::array::from_fn(|r| {
                                std::array::from_fn(|c| half(2 * (3 * r + c)))
                            }),
                            lo: std::array::from_fn(|j| half(32 + 2 * j)),
                            hi: std::array::from_fn(|j| half(40 + 2 * j)),
                        });
                    }
                    let mut branches = [0..0, 0..0];
                    for (i, range) in branches.iter_mut().enumerate() {
                        let child = u32le(d, p + 12 + i * 4)?;
                        let start = scene.polygons.len();
                        if child != 0 {
                            self.visit(child, transform, selection, pickup, ancestors, scene)?;
                        }
                        *range = start..scene.polygons.len();
                    }
                    scene.visibility.push(VisibilityController {
                        node: ptr,
                        transform,
                        boxes,
                        branches,
                    });
                }
            }
            op => bail!("unsupported DMD opcode {op} at {ptr:#x}"),
        }
        for child in children {
            self.visit(child, transform, selection, pickup, ancestors, scene)?;
        }
        ancestors.pop();
        Ok(())
    }
    fn object(
        &self,
        ptr: u32,
        p: usize,
        transform: Transform,
        pickup: Option<u8>,
        scene: &mut Scene,
    ) -> Result<()> {
        bytes(self.data, p, 44)?;
        let verts = self.offset(u32le(self.data, p + 4)?)?;
        let mut poly = self.offset(u32le(self.data, p + 12)?)?;
        let count = u32le(self.data, p + 16)? as usize;
        ensure!(count < 100_000, "unreasonable polygon count");
        scene.object_ids.push(ptr);
        for _ in 0..count {
            let header = bytes(self.data, poly, 16)?;
            let len = header[2] as usize * 4;
            ensure!(len >= 16, "invalid polygon size");
            let record = bytes(self.data, poly, len)?;
            let flags = u32le(record, 12)?;
            let n = ((flags >> 16) & 7) as usize;
            // geomSprite uses the first vertex as center. The two following
            // halfwords hold the billboard half-width and half-height.
            let billboard = if n == 3 || n == 4 {
                None
            } else {
                Some([
                    u16le(record, 6)? as i16 as f32,
                    u16le(record, 8)? as i16 as f32,
                ])
            };
            let n = if billboard.is_some() { 4 } else { n };
            let colors = header[1] as usize;
            ensure!(
                colors == 1 || colors == n,
                "unsupported color count {colors}"
            );
            let command = bytes(record, 16, 4)?[3];
            let uv_offset = 16 + ((flags >> 28) & 7) as usize * 4;
            let textured = command & 4 != 0;
            let texture = if textured {
                let uv = bytes(record, uv_offset, n * 4)?;
                let window = if header[0] & 0x80 != 0 {
                    u32le(record, uv_offset + n * 4)?
                } else {
                    0
                };
                Some(TextureKey {
                    page: u16le(uv, 6)?,
                    clut: u16le(uv, 2)?,
                    window,
                })
            } else {
                None
            };
            let mut vertices = Vec::new();
            for i in 0..n {
                let index = u16le(record, if billboard.is_some() { 4 } else { 4 + 2 * i })? as i16;
                ensure!(index >= 0, "negative vertex index");
                let v = bytes(self.data, verts + index as usize * 8, 8)?;
                let position = transform.apply(std::array::from_fn(|axis| {
                    i16::from_le_bytes([v[2 * axis], v[2 * axis + 1]]) as f32
                }));
                let c = bytes(record, 16 + if colors == 1 { 0 } else { i * 4 }, 4)?;
                let divisor = if textured { 128. } else { 255. };
                let color = if textured && command & 1 != 0 {
                    [1.; 4]
                } else {
                    [
                        c[0] as f32 / divisor,
                        c[1] as f32 / divisor,
                        c[2] as f32 / divisor,
                        1.,
                    ]
                };
                let uv = if textured {
                    let u = bytes(record, uv_offset + 4 * i, 2)?;
                    [u[0] as f32, u[1] as f32]
                } else {
                    [0., 0.]
                };
                vertices.push(Vertex {
                    position,
                    uv,
                    color,
                });
            }
            scene.polygons.push(Polygon {
                animation: Vec::new(),
                billboard,
                pickup,
                arena_switches: Vec::new(),
                health_stand: None,
                object_id: ptr,
                source_offset: poly,
                vertices,
                texture,
                double_sided: flags >> 31 != 0,
                semi_transparent: command & 2 != 0,
                fixed_ot: (u32le(self.data, p + 24)? >> 19) & 0xfff,
            });
            poly += len;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_bad_headers_and_addresses() {
        assert!(Dmd::parse(&[]).is_err());
        let mut d = vec![0; 28];
        for (offset, value) in [
            (0, 0x50535844u32),
            (4, 0x43),
            (12, 0x800188b8),
            (20, 1),
            (24, 0x80000000),
        ] {
            d[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        assert!(Dmd::parse(&d).is_err());
    }
    #[test]
    fn polygon_offsets_are_relative_to_payload() {
        let base = 0x800188b8u32;
        let mut d = vec![0; 180];
        let mut put =
            |offset: usize, value: u32| d[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        for (offset, value) in [
            (0, 0x50535844),
            (4, 0x43),
            (12, base),
            (20, 1),
            (24, base + 32),
            (32, 1 << 16),
            (36, base + 96),
            (44, base + 128),
            (48, 1),
        ] {
            put(offset, value);
        }
        // Object root, three source vertices and a textured triangle.
        for (i, xyz) in [[10i16, 20, 30], [40, 50, 60], [70, 80, 90]]
            .into_iter()
            .enumerate()
        {
            for (axis, value) in xyz.into_iter().enumerate() {
                let p = 96 + i * 8 + axis * 2;
                d[p..p + 2].copy_from_slice(&value.to_le_bytes());
            }
        }
        d[128..132].copy_from_slice(&[3, 1, 8, 7]);
        d[134..136].copy_from_slice(&1u16.to_le_bytes());
        d[136..138].copy_from_slice(&2u16.to_le_bytes());
        d[140..144].copy_from_slice(&((3u32 << 16) | (1 << 28)).to_le_bytes());
        d[144..148].copy_from_slice(&[128, 128, 128, 0x24]);
        d[148..160].copy_from_slice(&[1, 2, 4, 0, 3, 4, 0, 0, 5, 6, 0, 0]);
        let db = Dmd::parse(&d).unwrap();
        let scene = db.extract_kind(1).unwrap();
        let p = &scene.polygons[0];
        assert_eq!(p.vertices[2].position, [70., 80., 90.]);
        assert_eq!(p.vertices[0].uv, [1., 2.]);
        assert_eq!(p.vertices[1].uv, [3., 4.]);
        assert_eq!(p.texture.unwrap().clut, 4);
        assert_eq!(p.vertices[0].color, [1.; 4]);
        d[134..136].copy_from_slice(&(-1i16).to_le_bytes());
        assert!(Dmd::parse(&d).unwrap().extract_kind(1).is_err());
    }
}
