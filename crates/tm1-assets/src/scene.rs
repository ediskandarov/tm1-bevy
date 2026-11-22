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
#[derive(Clone, Debug)]
pub struct Polygon {
    pub billboard: Option<[f32; 2]>,
    pub pickup: Option<u8>,
    pub object_id: u32,
    pub source_offset: usize,
    pub vertices: Vec<Vertex>,
    pub texture: Option<TextureKey>,
    pub double_sided: bool,
    pub semi_transparent: bool,
    pub fixed_ot: u32,
}
#[derive(Debug)]
pub struct Scene {
    pub version: u32,
    pub root_id: u32,
    pub polygons: Vec<Polygon>,
    pub object_ids: Vec<u32>,
    pub omitted_nodes: Vec<String>,
}

#[derive(Clone, Copy)]
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
    /// Read the three body-switch branches selected by UA's damage update.
    pub fn vehicle_bodies(&self, kind: u16) -> Result<[Scene; 3]> {
        Ok([
            self.extract(kind, Some(0))?,
            self.extract(kind, Some(1))?,
            self.extract(kind, Some(2))?,
        ])
    }
    fn extract(&self, kind: u16, body: Option<u8>) -> Result<Scene> {
        let roots: Vec<_> = self.roots.iter().filter(|(_, k)| *k == kind).collect();
        ensure!(
            roots.len() == 1,
            "expected exactly one root of kind {kind}, found {}",
            roots.len()
        );
        let root_id = roots[0].0;
        let mut selection = None;
        if let Some(stage) = body {
            // uaswInitCarSwitch follows child zero through two groups.
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
            selection = Some((node, stage));
        }
        let mut scene = Scene {
            version: self.version,
            root_id,
            polygons: vec![],
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
        selection: Option<(u32, u8)>,
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
                    list(d[p + 20] as usize, p + 24)?;
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
                children.push(u32le(d, p + 16 + 4 * d[p + 12] as usize)?);
                scene
                    .omitted_nodes
                    .push(format!("animation {ptr:#x}: initial frame only"));
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
                    .filter(|(node, _)| *node == ptr)
                    .map_or(d[p + 7], |(_, stage)| stage);
                ensure!(child < d[p + 6], "switch child out of range");
                children.push(u32le(d, p + 8 + 4 * child as usize)?);
            }
            6..=8 | 10 => {}
            11 => {
                scene.omitted_nodes.push(format!(
                    "visibility controller {ptr:#x}: camera-dependent branch pending"
                ));
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
                billboard,
                pickup,
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
