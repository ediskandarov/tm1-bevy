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

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn spu_note_and_fine_pitch_match_retail() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    for note in 0..=127 {
        for fine in 0..=127 {
            assert_eq!(
                u32::from(tm1_core::sound::spu_pitch(note, fine)),
                cpu.call(0x8016fc60, &[60, 0, u32::from(note), u32::from(fine)]),
                "note={note} fine={fine}"
            );
        }
    }
}

#[test]
#[ignore = "requires the supported TM1_CUE"]
fn vab_channel_levels_match_retail_sdk_calls() {
    let disc = tm1_assets::disc::Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
    let mut cpu = Cpu::new(&disc.read("SIPS_600.07").unwrap());
    let bank = disc.read("SND/UACORE.VAB").unwrap();
    let system = 0x8016fecc;
    let sound = 0x801963a8;
    let tone = 0x80009000;
    cpu.put(system + 24, &(3357u32 * 3357).to_le_bytes());
    cpu.put(sound + 20, &1u32.to_le_bytes()); // already loaded instrument
    cpu.put(sound + 24, &u32::to_le_bytes(tone));
    cpu.put(tone + 0x40, &2u32.to_le_bytes());
    cpu.put(tone + 0x54, &1u32.to_le_bytes()); // voice already playing
    for prog in 0..59 {
        for t in tm1_assets::sound::program(&bank, prog).unwrap() {
            cpu.put(sound + 5, &[t.program_volume]);
            cpu.put(tone + 8, &[t.volume]);
            for (volume, percent) in [(0, 99), (33, 99), (33, 84), (57, 25), (57, 109), (127, 99)] {
                cpu.put(system + 4, &u32::from(volume).to_le_bytes());
                cpu.put(0x8018bf7c, &u32::MAX.to_le_bytes());
                // Full-right request. JP soundPlayId negates the horizontal offset.
                cpu.call(0x80106d60, &[0, 0, 160, 8, u32::from(percent), 0]);
                cpu.call_until(0x80105ebc, &[], 0x80106720);
                cpu.call_until(0x80104ad8, &[tone], 0x8016c494);
                let expected = [volume, 2 * volume].map(|v| {
                    u32::from(tm1_core::sound::voice_volume(
                        t.program_volume,
                        t.volume,
                        v,
                        percent,
                    ))
                });
                assert_eq!(
                    [cpu.r[5], cpu.r[6]],
                    expected,
                    "program {prog} tone volume {} channel {volume} scale {percent}",
                    t.volume
                );
            }
        }
    }
}
