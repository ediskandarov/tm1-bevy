use super::GeometryTransform;
use std::ops::Range;
use tm1_assets::scene::{HierarchyKind, Scene};
use tm1_core::drive::HierarchyTransform;

pub(super) struct BranchState<'a> {
    pub wheels: [i64; 14],
    pub rotation: [i16; 2],
    pub switches: [u8; 8],
    pub stands: u16,
    pub pickups: u64,
    pub damage: &'a dyn Fn(u32) -> u8,
}

pub(super) struct Object<S> {
    pub source: S,
    pub polygons: Range<usize>,
    pub slot: usize,
    pub position: [i32; 3],
}

pub(super) type Visits = std::collections::BTreeMap<u32, u64>;

pub(super) struct Pass<S> {
    pub visits: Visits,
    pub lod_scale: u32,
    matrices: Vec<HierarchyTransform>,
    pub lights: Vec<super::LightMatrix>,
    pub objects: Vec<Object<S>>,
    battle: bool,
    stack: Vec<(usize, usize, [i32; 3])>,
}

impl<S: Copy> Pass<S> {
    pub fn new(world: HierarchyTransform, battle: bool) -> Self {
        Self {
            visits: Visits::new(),
            lod_scale: 3,
            matrices: vec![world],
            lights: vec![super::WORLD_LIGHT],
            objects: Vec::with_capacity(220),
            battle,
            stack: Vec::new(),
        }
    }

    // Slot zero belongs to the world. Subsequent CS roots overwrite the last
    // slot at capacity, including roots that end up submitting no objects.
    pub fn coordinate_system(
        &mut self,
        root: HierarchyTransform,
        rotation: Option<super::LightMatrix>,
    ) -> usize {
        let light = rotation.map_or(super::WORLD_LIGHT, |r| {
            super::rotate_light(super::WORLD_LIGHT, r)
        });
        if self.battle && self.matrices.len() == 71 {
            self.lights[70] = light;
            self.matrices[70] = root;
            70
        } else {
            self.lights.push(light);
            self.matrices.push(root);
            self.matrices.len() - 1
        }
    }

    pub fn object(&mut self, source: S, polygons: Range<usize>, slot: usize, position: [i32; 3]) {
        if !self.battle || self.objects.len() < 220 {
            self.objects.push(Object {
                source,
                polygons,
                slot,
                position,
            });
        }
    }

    pub fn frame(&self, slot: usize, position: [i32; 3]) -> GeometryTransform {
        GeometryTransform::from(HierarchyTransform {
            position,
            ..self.matrices[slot]
        })
    }

    pub fn traverse(
        &mut self,
        source: S,
        scene: &Scene,
        root: usize,
        base: usize,
        slot: usize,
        depth: i32,
        shadow: bool,
        state: &BranchState,
    ) {
        self.stack.push((root, slot, self.matrices[slot].position));
        while let Some((i, mut slot, mut position)) = self.stack.pop() {
            let node = &scene.hierarchy[i];
            if self.battle
                && node
                    .bound
                    .is_some_and(|b| !self.frame(slot, position).contains(&scene.bounds[b], depth))
            {
                continue;
            }
            let mut branch = None;
            match &node.kind {
                HierarchyKind::Group => {}
                HierarchyKind::Shadow if !shadow => continue,
                HierarchyKind::DamageGroup if (state.damage)(node.address) != 0 => continue,
                HierarchyKind::Shadow | HierarchyKind::DamageGroup => {}
                HierarchyKind::Object(range) => {
                    self.object(source, base + range.start..base + range.end, slot, position);
                    continue;
                }
                HierarchyKind::Transform { index, pickup } => {
                    if pickup.is_some_and(|id| state.pickups & (1u64 << id) != 0) {
                        continue;
                    }
                    let n = &scene.transforms[*index - 1];
                    let rotation = if n.truck_wheel {
                        let [s, c] = state.rotation;
                        Some([[4096, 0, 0], [0, c, s], [0, -s, c]])
                    } else {
                        n.rotation
                    };
                    let parent = HierarchyTransform {
                        position,
                        ..self.matrices[slot]
                    };
                    let child = parent.child(n.offset, rotation);
                    position = child.position;
                    if let Some(rotation) = rotation {
                        if !self.battle || self.matrices.len() < 71 {
                            self.lights
                                .push(super::rotate_light(self.lights[slot], rotation));
                            self.matrices.push(child);
                        }
                        // A rotation that cannot allocate still transforms position.
                        // Its children inherit the global last index, even on a sibling path.
                        slot = self.matrices.len() - 1;
                    }
                }
                HierarchyKind::Lod(id) => {
                    let Some(active) =
                        scene.lods.as_ref().unwrap()[*id].selected_index(position, self.lod_scale)
                    else {
                        continue;
                    };
                    branch = Some(active);
                }
                HierarchyKind::Controller(id) => {
                    branch = Some(usize::from(
                        scene.visibility[*id].inside_at_position(position),
                    ))
                }
                HierarchyKind::Animation(f) => {
                    let visits = self.visits.entry(f.node).or_default();
                    *visits += 1;
                    branch = Some(usize::from(f.selected(*visits)));
                }
                HierarchyKind::Tire {
                    slot,
                    initial,
                    count,
                } => {
                    branch = Some(
                        (state.wheels[*slot] + i64::from(*initial)).rem_euclid(i64::from(*count))
                            as usize,
                    )
                }
                HierarchyKind::ArenaSwitch(n) => {
                    branch = Some(usize::from(state.switches[usize::from(*n)]))
                }
                HierarchyKind::Stand(n) => branch = Some(usize::from((state.stands >> n) & 1)),
                HierarchyKind::DamageSwitch => {
                    branch = Some(usize::from((state.damage)(node.address)))
                }
            }
            let pending = self.stack.len();
            let mut child = i + 1;
            while child < node.end {
                let next = &scene.hierarchy[child];
                if branch.is_none_or(|b| b == next.branch) {
                    self.stack.push((child, slot, position));
                }
                child = next.end;
            }
            self.stack[pending..].reverse();
            if self.battle {
                self.stack.truncate(150);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference_cpu::Cpu;
    use tm1_assets::{disc::Disc, scene::Dmd};

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn vehicle_vertex_lighting_matches_disc_environments() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
        // These SDK calls only copy the already constructed environment to COP2.
        for addr in [0x8015c770, 0x8015c7a0, 0x8015d298] {
            cpu.put(addr, &[8, 0, 224, 3, 0, 0, 0, 0]);
        }
        cpu.call(0x80103f0c, &[]);
        let matrix = |cpu: &Cpu, address| -> [[i16; 3]; 3] {
            std::array::from_fn(|i| {
                std::array::from_fn(|j| {
                    let a = address + 2 * (3 * i + j) as u32;
                    (cpu.word(a & !3) >> (8 * (a & 3))) as i16
                })
            })
        };
        let data = disc.read("UADMD/WH1.DMD").unwrap();
        let db = Dmd::parse(&data).unwrap();
        let bodies = db.vehicle_bodies_with_lights(20, true).unwrap();
        let mut samples = 0;
        let battle = (0..4).map(|id| (id, super::super::LightEnvironment::battle(id as u8), false));
        for (id, environment, garage) in
            battle.chain([(0, super::super::LightEnvironment::GARAGE, true)])
        {
            if garage {
                cpu.call_until(0x8013aa50, &[], 0x8013aa6c);
            }
            let addr = 0x801960b8 + 0x4c * id;
            let light = matrix(&cpu, addr + 0x2c);
            let colors = matrix(&cpu, addr + 0x0c);
            assert_eq!(light, super::super::WORLD_LIGHT);
            for p in bodies.iter().flat_map(|s| &s.polygons) {
                for v in &p.vertices {
                    let Some(normal) = v.normal else { continue };
                    let scale = if p.texture.is_some() { 128. } else { 255. };
                    let rgb: [i32; 3] =
                        std::array::from_fn(|i| (scale * v.color[i]).round() as i32);
                    let ir = light.map(|row| {
                        ((0..3)
                            .map(|j| i64::from(row[j]) * i64::from(normal[j]))
                            .sum::<i64>()
                            >> 12)
                            .clamp(0, 32767)
                    });
                    let expected = std::array::from_fn(|i| {
                        let background = i64::from(cpu.word(addr + 4 * i as u32)) << 16;
                        let sum =
                            (0..3).fold(background, |sum, j| sum + i64::from(colors[i][j]) * ir[j]);
                        let mac = ((sum >> 12).clamp(0, 32767) * (i64::from(rgb[i]) << 4)) >> 12;
                        (mac >> 4).clamp(0, 255) as i32
                    });
                    assert_eq!(environment.vertex_rgb(v, scale, light), expected);
                    samples += 1;
                }
            }
        }
        assert!(samples > 100);
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn shared_animation_advances_only_on_visible_hierarchy_visits() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let mut data = disc.read("UADMD/XARENA1.DMD").unwrap();
        let db = Dmd::parse(&data).unwrap();
        let scene = db.battle_world().unwrap();
        let frame = scene
            .hierarchy
            .iter()
            .find_map(|n| match n.kind {
                HierarchyKind::Animation(f) => Some(f),
                _ => None,
            })
            .unwrap();
        // A bounded group references the real crowd animation twice.
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let root = base + data.len() as u32;
        let mut group = [0; 32];
        group[0] = 1;
        group[20] = 2;
        group[24..28].copy_from_slice(&frame.node.to_le_bytes());
        group[28..32].copy_from_slice(&frame.node.to_le_bytes());
        data.extend(group);
        let mut db = Dmd::parse(&data).unwrap();
        db.roots = vec![(root, 1)];
        let scene = db.battle_world().unwrap();
        let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
        cpu.put(base, &data);
        const CS: u32 = 0x80008000;
        let mut visits = Visits::new();
        let mut expected = 0;
        for i in 0..64 {
            let visible = i % 5 < 3;
            let position = [if visible { 10i32 } else { -10 }, 0, 0];
            let mut normals = [[0; 3]; 6];
            normals[0][0] = 4096;
            let mut pass = Pass::new(
                HierarchyTransform {
                    position,
                    normals,
                    rows: [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
                },
                true,
            );
            pass.visits = visits;
            pass.traverse(
                (),
                &scene,
                0,
                0,
                0,
                1200,
                true,
                &super::super::tests::test_branches(),
            );
            visits = pass.visits;
            expected += if visible { 2 } else { 0 };
            assert_eq!(visits.get(&frame.node).copied().unwrap_or(0), expected);
            // Vary the battle update length while visiting the same shared animation.
            let fields = [3u16, 4, 5][i % 3];
            cpu.put(0x80170238, &fields.to_le_bytes());
            cpu.put(0x8017023a, &(60 / fields).to_le_bytes());
            cpu.put(CS + 0x68, &position.map(i32::to_le_bytes).concat());
            cpu.put(CS + 0xc4, &4096i16.to_le_bytes());
            cpu.put(0x8018c484, &0u32.to_le_bytes());
            cpu.call(0x800fd914, &[CS, root]);
            assert_eq!(cpu.word(frame.node + 12) as u8, frame.selected(expected));
            assert_eq!(cpu.word(0x8018c484) as usize, pass.objects.len());
            for (j, object) in pass.objects.iter().enumerate() {
                assert_eq!(
                    scene.polygons[object.polygons.start].object_id,
                    cpu.word(0x801acfe8 + 32 * j as u32)
                );
            }
        }
    }

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn warehouse_view_matches_retail_object_list_and_final_matrices() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let data = disc.read("UADMD/WH1.DMD").unwrap();
        let db = Dmd::parse(&data).unwrap();
        let world = db.battle_world().unwrap();
        let sky = db.battle_sky().unwrap();
        let mut models = vec![world, sky];
        for &(id, kind) in &db.roots {
            if (10..=130).contains(&kind) && kind % 10 == 0 {
                let mut raw = Dmd::parse(&data).unwrap();
                raw.roots = vec![(id, 1)];
                models.push(raw.battle_world().unwrap());
            }
        }
        let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
        let base = u32::from_le_bytes(data[12..16].try_into().unwrap());
        cpu.put(base, &data);
        // Keep animation clocks fixed and select initial branches for the constructed view.
        for model in &models {
            for n in &model.hierarchy {
                match n.kind {
                    HierarchyKind::Animation(_) => cpu.put(n.address + 6, &u16::MAX.to_le_bytes()),
                    HierarchyKind::Stand(_)
                    | HierarchyKind::DamageSwitch
                    | HierarchyKind::ArenaSwitch(_) => cpu.put(n.address + 7, &[0]),
                    _ => {}
                }
            }
        }
        let word = |cpu: &mut Cpu, addr: u32, value: u32| cpu.put(addr, &value.to_le_bytes());
        let put_rotation = |cpu: &mut Cpu, addr: u32, m: [[i32; 3]; 3]| {
            cpu.put(
                addr,
                &m.into_iter()
                    .flatten()
                    .flat_map(|v| (v as i16).to_le_bytes())
                    .collect::<Vec<_>>(),
            )
        };
        put_rotation(
            &mut cpu,
            0x80007000 + 0x2c,
            super::super::WORLD_LIGHT.map(|r| r.map(i32::from)),
        );
        let cs = 0x80008000;
        let identity = [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]];
        let mut saturated = false;
        for (instances, mirror) in [(3, false), (12, true), (84, false)] {
            let normals = if mirror {
                [
                    [-3454, 2200, 0],
                    [3454, 2200, 0],
                    [0, 4095, 0],
                    [0, 1765, -3695],
                    [0, 1765, 3695],
                    [0, -4095, 0],
                ]
            } else {
                [
                    [-3355, 2349, 0],
                    [3355, 2349, 0],
                    [0, 4095, 0],
                    [0, 1904, -3626],
                    [0, 1904, 3626],
                    [0, -4095, 0],
                ]
            };
            let world = HierarchyTransform {
                rows: [[4096, 0, 0], [0, 0, -3690], [0, 3690, 0]],
                position: [20, 650, -80],
                normals,
            };
            let mut pass = Pass::new(world, true);
            let state = super::super::tests::test_branches();
            word(&mut cpu, 0x8018c4b0, cs);
            word(&mut cpu, cs + 4, 0x80007000);
            put_rotation(&mut cpu, cs + 0x34, world.rows);
            cpu.put(cs + 0x68, &world.position.map(i32::to_le_bytes).concat());
            for (group, ns) in normals.chunks_exact(3).enumerate() {
                put_rotation(
                    &mut cpu,
                    cs + 0xc4 + 32 * group as u32,
                    std::array::from_fn(|i| std::array::from_fn(|j| ns[j][i])),
                );
            }
            word(&mut cpu, cs + 0x104, models[0].root_id);
            let which = u32::from(mirror);
            word(
                &mut cpu,
                0x8018c51c + 4 * which,
                if mirror { 13 } else { 6 },
            );
            word(
                &mut cpu,
                0x8018bf90 + 4 * which,
                if mirror { 0 } else { cs + 0x110 },
            );
            word(&mut cpu, 0x8018bf14, cs + 2 * 0x110);
            pass.traverse(
                0,
                &models[0],
                0,
                0,
                0,
                if mirror { 100 } else { 1200 },
                true,
                &state,
            );
            for i in 1..instances + 2 {
                let addr = cs + i * 0x110;
                let model = if i == 1 {
                    1
                } else {
                    2 + (i as usize - 2) % (models.len() - 2)
                };
                let pos = if instances == 84 && (2..33).contains(&i) {
                    [0, -25000, 0]
                } else if i == 1 {
                    [0; 3]
                } else {
                    [(i as i32 % 7 - 3) * 20, i as i32 * 11, 20]
                };
                let rotation = if i % 2 == 0 {
                    [[3784, -1567, 0], [1567, 3784, 0], [0, 0, 4096]]
                } else {
                    identity
                };
                word(&mut cpu, addr + 4, 0x80007000);
                word(&mut cpu, addr + 8, 1);
                word(&mut cpu, addr + 0x10, cs);
                put_rotation(&mut cpu, addr + 0x14, rotation);
                cpu.put(addr + 0x5c, &pos.map(i32::to_le_bytes).concat());
                word(&mut cpu, addr + 0x104, models[model].root_id);
                word(
                    &mut cpu,
                    addr + 0x108,
                    if i == instances + 1 { 0 } else { addr + 0x110 },
                );
                if mirror && i == 1 {
                    continue;
                }
                let frame = world.child(pos, Some(rotation.map(|r| r.map(|v| v as i16))));
                let slot =
                    pass.coordinate_system(frame, Some(rotation.map(|r| r.map(|v| v as i16))));
                pass.traverse(
                    model,
                    &models[model],
                    0,
                    0,
                    slot,
                    if mirror { 1000 } else { 1200 },
                    true,
                    &state,
                );
            }
            cpu.call_until(0x800fe57c, &[0x80007100, which], 0x800f78b8);
            let count = cpu.word(0x8018c484) as usize;
            let last = cpu.word(0x8018c488) as usize;
            assert_eq!(pass.matrices.len(), last + 1, "{instances} matrices");
            assert_eq!(pass.objects.len(), count, "{instances} objects");
            for (i, item) in pass.objects.iter().enumerate() {
                let rec = 0x801acfe8 + 32 * i as u32;
                let id = models[item.source].polygons[item.polygons.start].object_id;
                assert_eq!(id, cpu.word(rec), "object {i}/{count}");
                assert_eq!(item.slot, cpu.word(rec + 24) as usize);
                assert_eq!(item.position, [4, 8, 12].map(|d| cpu.word(rec + d) as i32));
            }
            for (slot, mat) in pass.matrices.iter().enumerate() {
                for (i, &val) in mat.rows.as_flattened().iter().enumerate() {
                    let a = 0x801aeb70 + 32 * slot as u32 + 2 * i as u32;
                    assert_eq!(
                        val,
                        (cpu.word(a & !3) >> (8 * (a & 3))) as i16 as i32,
                        "slot {slot}"
                    );
                }
            }
            for (slot, matrix) in pass.lights.iter().enumerate() {
                for (i, &value) in matrix.as_flattened().iter().enumerate() {
                    let a = 0x801f729c + 32 * slot as u32 + 2 * i as u32;
                    assert_eq!(
                        value,
                        (cpu.word(a & !3) >> (8 * (a & 3))) as i16,
                        "light slot {slot}"
                    );
                }
            }
            saturated |= count == 220 && last == 70 && pass.objects.iter().any(|o| o.slot == 70);
        }
        assert!(saturated);
    }
}
