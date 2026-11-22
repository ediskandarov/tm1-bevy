#[path = "support/mips.rs"]
mod mips;
use mips::Cpu;
use tm1_core::math::{self, Matrix};

fn rng(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

#[test]
#[ignore = "requires TM1_CUE pointing to the supported player-owned disc"]
fn native_math_matches_original_instructions() {
    let cue = std::env::var_os("TM1_CUE").expect("set TM1_CUE");
    let disc = tm1_assets::disc::Disc::open(cue).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    let mut seed = 0x544d_3101;
    for case in 0..2048 {
        let matrix = Matrix {
            rotation: std::array::from_fn(|_| std::array::from_fn(|_| rng(&mut seed) as i16)),
            translation: [123, -456, 789],
        };
        let input: [i32; 3] = std::array::from_fn(|_| rng(&mut seed) as i32);
        for row in 0..3 {
            for col in 0..3 {
                cpu.put(
                    0x8002_0000 + (row * 6 + col * 2) as u32,
                    &matrix.rotation[row][col].to_le_bytes(),
                );
            }
        }
        for i in 0..3 {
            cpu.put(
                0x8002_0014 + i as u32 * 4,
                &matrix.translation[i].to_le_bytes(),
            );
        }
        for (address, expected, short) in [
            (0x800f_e9c4, math::multiply(&matrix, input, false), false),
            (0x800f_eaf8, math::multiply(&matrix, input, true), false),
            (0x800f_ec2c, math::multiply_long(&matrix, input), false),
            (
                0x800f_ed60,
                math::multiply_short_transpose(&matrix, input.map(|v| v as i16)),
                true,
            ),
        ] {
            for (i, v) in input.iter().enumerate() {
                if short {
                    cpu.put(0x8002_0040 + i as u32 * 2, &(*v as i16).to_le_bytes());
                } else {
                    cpu.put(0x8002_0040 + i as u32 * 4, &v.to_le_bytes());
                }
            }
            cpu.call(address, &[0x8002_0000, 0x8002_0040, 0x8002_0080]);
            let actual: [i32; 3] =
                std::array::from_fn(|i| cpu.word(0x8002_0080 + i as u32 * 4) as i32);
            assert_eq!(actual, expected, "function {address:#x}, case {case}");
        }
    }
    for case in 0..4096 {
        let input = if case == 0 {
            [0; 3]
        } else {
            std::array::from_fn(|_| (rng(&mut seed) % 50_001) as i32 - 25_000)
        };
        for (i, v) in input.iter().enumerate() {
            cpu.put(0x8002_0040 + i as u32 * 4, &v.to_le_bytes());
        }
        cpu.call(0x800f_eedc, &[0x8002_0040, 0x8002_0080]);
        let actual: [i32; 3] = std::array::from_fn(|i| cpu.word(0x8002_0080 + i as u32 * 4) as i32);
        assert_eq!(
            actual,
            math::normalize(input).unwrap(),
            "normalize case {case}"
        );
    }
    // Cover every table entry, scaling boundary, and random positive input.
    let mut values: Vec<u32> = (0..=256).collect();
    for bit in 1..31 {
        let n = 1u32 << bit;
        values.extend([n - 1, n, n + 1]);
    }
    values.extend((0..4096).map(|_| rng(&mut seed) & 0x7fff_ffff));
    values.push(i32::MAX as u32);
    for value in &values {
        assert_eq!(
            cpu.call(0x8015_bf74, &[*value]),
            math::square_root(*value).unwrap(),
            "sqrt {value}"
        );
    }
    assert!(math::normalize([i32::MAX; 3]).is_none());
    eprintln!(
        "Original MIPS vs native C: 8192 matrix cases, 4096 normalization cases, {} square-root cases; seed 0x544d3101",
        values.len()
    );
}
