use super::mips::Cpu;

// Run the retail DB traversal over every root, then replay its target callbacks.
// Unrelated database callbacks require the console audio and display subsystems.
pub fn register_retail_targets(cpu: &mut Cpu, roots: &[(u32, u16)]) -> Vec<(u32, u32)> {
    const PROCESS: u32 = 0x8010083c;
    let start = (PROCESS & 0x1fffff) as usize;
    let original = cpu.ram[start..start + 8].to_vec();
    // printf reports unsupported hierarchy opcodes and a full target table.
    cpu.put(0x801548b8, &[8, 0, 224, 3, 0, 0, 0, 0]);
    cpu.put(PROCESS, &[8, 0, 224, 3, 0, 0, 0, 0]);
    let mut calls = Vec::new();
    let mut pedestrians = Vec::new();
    let mut sprite_loads = Vec::new();
    for &(root, _) in roots {
        cpu.call_observed(0x801001f8, &[root], 0x80000080, |cpu| {
            if cpu.pc() != PROCESS {
                return;
            }
            let kind = cpu.r[4];
            if matches!(kind, 751 | 756 | 760 | 772 | 773 | 775..=778 | 781 | 782) {
                pedestrians.push((kind, cpu.r[5]));
            }
            let loader = match kind {
                650 => Some(0x80137748),
                651 => Some(0x8013785c),
                654 => Some(0x801377a4),
                1280 => Some(0x80144bf4),
                _ => None,
            };
            if let Some(loader) = loader {
                sprite_loads.push((loader, cpu.r[5], 0));
            }
            if matches!(kind, 301 | 302 | 308 | 309) {
                sprite_loads.push((0x8010c108, kind, cpu.r[5]));
            } else if matches!(kind, 670 | 671) {
                sprite_loads.push((0x80137aac, kind, cpu.r[5]));
            }
            let tracked = matches!(
                kind,
                600 | 601 | 610..=613 | 715 | 716 | 720 | 721 | 730 | 753..=755 | 770 | 902 | 920
                    | 921 | 930 | 931 | 1000..=1005 | 1008..=1010 | 1015 | 1017
            );
            if tracked {
                let pos = [0, 4, 8].map(|off| cpu.word(cpu.r[6] + off));
                calls.push((kind, cpu.r[5], pos));
            }
        });
    }
    cpu.put(PROCESS, &original);
    for (load, arg0, arg1) in sprite_loads {
        cpu.call(load, &[arg0, arg1]);
    }
    for i in 0..50 {
        cpu.put(0x801ac968 + i * 28 + 8, &u32::MAX.to_le_bytes());
    }
    for (kind, ptr, pos) in calls {
        for (i, v) in pos.into_iter().enumerate() {
            cpu.put(0x80011000 + i as u32 * 4, &v.to_le_bytes());
        }
        cpu.call(PROCESS, &[kind, ptr, 0x80011000, 0, 0x80011000]);
    }
    pedestrians
}
