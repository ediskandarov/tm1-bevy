//! Layout of the JP font atlases. Pixel indices and palettes come from the disc TIMs.
use super::Glyph;

const fn glyph(u: usize, v: usize, w: usize, h: usize, y: i32) -> Glyph {
    Glyph { u, v, w, h, y }
}

pub(super) const TEXT: [Glyph; 77] = [
    glyph(210, 58, 14, 19, 0),
    glyph(224, 58, 9, 19, 0),
    glyph(236, 58, 14, 19, 0),
    glyph(153, 77, 14, 19, 0),
    glyph(166, 77, 14, 19, 0),
    glyph(181, 77, 14, 19, 0),
    glyph(194, 77, 14, 19, 0),
    glyph(207, 77, 14, 19, 0),
    glyph(220, 77, 14, 19, 0),
    glyph(233, 77, 14, 19, 0),
    glyph(41, 40, 6, 10, 0),
    glyph(42, 44, 6, 6, 0),
    glyph(101, 82, 13, 8, 0),
    glyph(138, 98, 12, 3, -12),
    glyph(58, 95, 15, 15, 0),
    glyph(138, 77, 14, 19, 0),
    glyph(115, 79, 12, 12, 0),
    glyph(153, 1, 14, 19, 0),
    glyph(166, 1, 14, 19, 0),
    glyph(179, 1, 14, 19, 0),
    glyph(192, 1, 14, 19, 0),
    glyph(205, 1, 13, 19, 0),
    glyph(217, 1, 13, 19, 0),
    glyph(229, 1, 14, 19, 0),
    glyph(152, 21, 14, 19, 0),
    glyph(166, 21, 7, 19, 0),
    glyph(173, 21, 14, 19, 0),
    glyph(187, 21, 16, 19, 0),
    glyph(202, 21, 13, 19, 0),
    glyph(214, 21, 21, 19, 0),
    glyph(234, 21, 16, 19, 0),
    glyph(150, 39, 14, 19, 0),
    glyph(163, 39, 14, 19, 0),
    glyph(176, 39, 14, 19, 0),
    glyph(189, 39, 14, 19, 0),
    glyph(202, 39, 13, 19, 0),
    glyph(214, 39, 13, 19, 0),
    glyph(226, 39, 14, 19, 0),
    glyph(239, 39, 14, 19, 0),
    glyph(151, 58, 21, 19, 0),
    glyph(171, 58, 14, 19, 0),
    glyph(184, 58, 13, 19, 0),
    glyph(197, 58, 14, 19, 0),
    glyph(79, 38, 7, 16, 2),
    glyph(247, 77, 7, 19, 0),
    glyph(85, 38, 7, 16, 2),
    glyph(64, 44, 6, 8, 1),
    glyph(92, 42, 12, 6, -3),
    glyph(71, 38, 6, 8, -11),
    glyph(1, 55, 13, 12, 0),
    glyph(13, 55, 12, 12, 0),
    glyph(24, 55, 12, 12, 0),
    glyph(35, 55, 12, 12, 0),
    glyph(37, 79, 11, 12, 0),
    glyph(56, 55, 11, 12, 0),
    glyph(66, 55, 12, 12, 0),
    glyph(77, 55, 12, 12, 0),
    glyph(88, 55, 6, 12, 0),
    glyph(93, 55, 11, 12, 0),
    glyph(103, 55, 11, 12, 0),
    glyph(113, 55, 11, 12, 0),
    glyph(2, 67, 13, 12, 0),
    glyph(49, 79, 12, 12, 0),
    glyph(25, 67, 12, 12, 0),
    glyph(36, 67, 12, 12, 0),
    glyph(64, 79, 12, 13, 1),
    glyph(58, 67, 12, 12, 0),
    glyph(69, 67, 12, 12, 0),
    glyph(80, 67, 12, 12, 0),
    glyph(91, 67, 12, 12, 0),
    glyph(102, 67, 12, 12, 0),
    glyph(113, 67, 16, 12, 0),
    glyph(2, 79, 12, 12, 0),
    glyph(13, 79, 12, 12, 0),
    glyph(24, 79, 11, 12, 0),
    glyph(81, 93, 8, 18, 1),
    glyph(75, 93, 8, 18, 1),
];

pub(super) const PASSWORD: [Glyph; 77] = [
    glyph(2, 2, 14, 12, 0),
    glyph(16, 14, 7, 12, 0),
    glyph(21, 2, 14, 12, 0),
    glyph(34, 2, 14, 12, 0),
    glyph(47, 2, 13, 12, 0),
    glyph(59, 2, 14, 12, 0),
    glyph(72, 2, 14, 12, 0),
    glyph(85, 2, 12, 12, 0),
    glyph(96, 2, 14, 12, 0),
    glyph(109, 2, 14, 12, 0),
    glyph(41, 40, 6, 10, 0),
    glyph(42, 44, 6, 6, 0),
    glyph(5, 95, 15, 15, 0),
    glyph(23, 95, 15, 15, 0),
    glyph(58, 95, 15, 15, 0),
    glyph(54, 38, 10, 12, 0),
    glyph(41, 95, 15, 15, 0),
    glyph(2, 14, 14, 12, 0),
    glyph(140, 2, 14, 12, 0),
    glyph(28, 14, 14, 12, 0),
    glyph(41, 14, 14, 12, 0),
    glyph(135, 40, 13, 12, 0),
    glyph(66, 14, 13, 12, 0),
    glyph(78, 14, 14, 12, 0),
    glyph(91, 14, 14, 12, 0),
    glyph(104, 14, 6, 12, 0),
    glyph(109, 14, 13, 12, 0),
    glyph(121, 14, 13, 12, 0),
    glyph(124, 55, 13, 12, 0),
    glyph(2, 26, 15, 12, 0),
    glyph(137, 55, 14, 12, 0),
    glyph(29, 26, 14, 12, 0),
    glyph(42, 26, 14, 12, 0),
    glyph(55, 26, 14, 13, 1),
    glyph(68, 26, 14, 12, 0),
    glyph(81, 26, 14, 12, 0),
    glyph(94, 26, 14, 12, 0),
    glyph(126, 2, 14, 12, 0),
    glyph(120, 26, 14, 12, 0),
    glyph(133, 26, 18, 12, 0),
    glyph(2, 38, 14, 12, 0),
    glyph(120, 40, 14, 12, 0),
    glyph(28, 38, 14, 12, 0),
    glyph(79, 38, 7, 16, 2),
    glyph(48, 38, 6, 12, 0),
    glyph(85, 38, 7, 16, 2),
    glyph(64, 44, 6, 8, 1),
    glyph(42, 49, 14, 6, 5),
    glyph(71, 38, 6, 8, -5),
    glyph(1, 55, 13, 12, 0),
    glyph(13, 55, 12, 12, 0),
    glyph(24, 55, 12, 12, 0),
    glyph(35, 55, 12, 12, 0),
    glyph(37, 79, 11, 12, 0),
    glyph(56, 55, 11, 12, 0),
    glyph(66, 55, 12, 12, 0),
    glyph(77, 55, 12, 12, 0),
    glyph(88, 55, 6, 12, 0),
    glyph(93, 55, 11, 12, 0),
    glyph(103, 55, 11, 12, 0),
    glyph(113, 55, 11, 12, 0),
    glyph(2, 67, 13, 12, 0),
    glyph(49, 79, 12, 12, 0),
    glyph(25, 67, 12, 12, 0),
    glyph(36, 67, 12, 12, 0),
    glyph(64, 79, 12, 13, 1),
    glyph(58, 67, 12, 12, 0),
    glyph(69, 67, 12, 12, 0),
    glyph(80, 67, 12, 12, 0),
    glyph(91, 67, 12, 12, 0),
    glyph(102, 67, 12, 12, 0),
    glyph(113, 67, 16, 12, 0),
    glyph(2, 79, 12, 12, 0),
    glyph(13, 79, 12, 12, 0),
    glyph(24, 79, 11, 12, 0),
    glyph(80, 92, 8, 20, 2),
    glyph(88, 92, 8, 20, 2),
];

pub(super) const SPEED_DIGITS: [Glyph; 10] = [
    glyph(136, 111, 9, 14, 0),
    glyph(148, 111, 9, 14, 0),
    glyph(160, 111, 9, 14, 0),
    glyph(172, 111, 9, 14, 0),
    glyph(184, 111, 9, 14, 0),
    glyph(196, 111, 9, 14, 0),
    glyph(208, 111, 9, 14, 0),
    glyph(220, 111, 9, 14, 0),
    glyph(232, 111, 9, 14, 0),
    glyph(244, 111, 9, 14, 0),
];

pub(super) const SIGN: [Glyph; 77] = [
    glyph(0, 0, 7, 9, 0),
    glyph(9, 0, 4, 9, 0),
    glyph(16, 0, 7, 9, 0),
    glyph(24, 0, 7, 9, 0),
    glyph(32, 0, 7, 9, 0),
    glyph(40, 0, 6, 9, 0),
    glyph(48, 0, 7, 9, 0),
    glyph(56, 0, 7, 9, 0),
    glyph(64, 0, 7, 9, 0),
    glyph(72, 0, 7, 9, 0),
    glyph(81, 2, 3, 7, 0),
    glyph(81, 6, 3, 3, 0),
    glyph(90, 38, 18, 18, 3),
    glyph(47, 45, 6, 3, -2),
    glyph(109, 38, 18, 18, 3),
    glyph(109, 0, 6, 9, 0),
    glyph(58, 42, 18, 9, 0),
    glyph(0, 14, 9, 9, 0),
    glyph(10, 14, 7, 9, 0),
    glyph(19, 14, 7, 9, 0),
    glyph(27, 14, 8, 9, 0),
    glyph(36, 14, 6, 9, 0),
    glyph(44, 14, 6, 9, 0),
    glyph(52, 14, 8, 9, 0),
    glyph(62, 14, 8, 9, 0),
    glyph(71, 14, 3, 9, 0),
    glyph(75, 14, 5, 9, 0),
    glyph(82, 14, 8, 9, 0),
    glyph(91, 14, 6, 9, 0),
    glyph(99, 14, 11, 9, 0),
    glyph(0, 28, 7, 9, 0),
    glyph(8, 28, 9, 9, 0),
    glyph(18, 28, 7, 9, 0),
    glyph(27, 28, 9, 11, 2),
    glyph(37, 28, 7, 9, 0),
    glyph(46, 28, 6, 9, 0),
    glyph(53, 28, 7, 9, 0),
    glyph(61, 28, 8, 9, 0),
    glyph(70, 28, 9, 9, 0),
    glyph(80, 28, 12, 9, 0),
    glyph(93, 28, 8, 9, 0),
    glyph(102, 28, 9, 9, 0),
    glyph(112, 28, 7, 9, 0),
    glyph(0, 41, 5, 11, 1),
    glyph(37, 41, 3, 10, 0),
    glyph(11, 41, 5, 11, 1),
    glyph(85, 6, 3, 5, 2),
    glyph(22, 50, 7, 2, 1),
    glyph(85, 6, 3, 5, -6),
    glyph(0, 58, 7, 7, 0),
    glyph(8, 56, 7, 9, 0),
    glyph(16, 58, 6, 7, 0),
    glyph(24, 56, 7, 9, 0),
    glyph(32, 58, 7, 7, 0),
    glyph(40, 56, 6, 9, 0),
    glyph(47, 58, 7, 9, 2),
    glyph(55, 56, 7, 9, 0),
    glyph(63, 56, 3, 9, 0),
    glyph(66, 56, 5, 11, 2),
    glyph(72, 56, 7, 9, 0),
    glyph(80, 56, 3, 9, 0),
    glyph(84, 58, 11, 7, 0),
    glyph(0, 72, 7, 7, 0),
    glyph(8, 72, 7, 7, 0),
    glyph(17, 72, 7, 9, 2),
    glyph(25, 72, 7, 9, 2),
    glyph(33, 72, 5, 7, 0),
    glyph(40, 72, 5, 7, 0),
    glyph(47, 71, 5, 8, 0),
    glyph(54, 72, 7, 7, 0),
    glyph(62, 72, 7, 7, 0),
    glyph(71, 72, 11, 7, 0),
    glyph(83, 72, 8, 7, 0),
    glyph(92, 72, 8, 9, 2),
    glyph(101, 72, 6, 7, 0),
    glyph(0, 0, 0, 0, 0),
    glyph(0, 0, 0, 0, 0),
];

pub(super) const CAR_NAMES: [&str; 13] = [
    "SWEET TOOTH",
    "YELLOW JACKET",
    "DARKSIDE",
    "OUTLAW",
    "THUMPER",
    "CRIMSON FURY",
    "PIT VIPER",
    "WARTHOG",
    "MR GRIMM",
    "SPECTRE",
    "HAMMERHEAD",
    "ROAD KILL",
    "MINION",
];

// Backspaces adjust the spacing of two HUD labels in the original font renderer.
pub(super) const CAR_SIGNS: [&str; 13] = {
    let mut signs = CAR_NAMES;
    signs[1] = "Y\x08ELL\x08\x08O\x08W \x08\x08\x08\x08J\x08A\x08CK\x08ET";
    signs[5] = "CRIMSON \x08\x08FURY";
    signs
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        disc::Disc,
        screen::{Face, Fonts},
    };

    #[test]
    #[ignore = "requires the supported TM1_CUE"]
    fn layout_matches_jp_font_records() {
        let disc = Disc::open(std::env::var_os("TM1_CUE").unwrap()).unwrap();
        let exe = disc.read("SIPS_600.07").unwrap();
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let offset = |address: u32| 2048 + (address - base) as usize;
        let fonts = Fonts::load(&disc).unwrap();
        for (address, glyphs) in [
            (0x801709b8, TEXT.as_slice()),
            (0x801707e8, PASSWORD.as_slice()),
            (0x80170b88, SPEED_DIGITS.as_slice()),
            (0x80170bc4, SIGN.as_slice()),
        ] {
            for (i, g) in glyphs.iter().enumerate() {
                let at = offset(address) + 6 * i;
                assert_eq!(
                    [g.u, g.v, g.w, g.h],
                    <[u8; 4]>::try_from(&exe[at..at + 4])
                        .unwrap()
                        .map(usize::from)
                );
                assert_eq!(
                    g.y,
                    i32::from(i16::from_le_bytes(exe[at + 4..at + 6].try_into().unwrap()))
                );
            }
        }
        for (address, labels) in [(0x801712e0, CAR_SIGNS), (0x80171768, CAR_NAMES)] {
            for (i, label) in labels.iter().enumerate() {
                let at = offset(address) + 4 * i;
                let ptr = u32::from_le_bytes(exe[at..at + 4].try_into().unwrap());
                let cstr = std::ffi::CStr::from_bytes_until_nul(&exe[offset(ptr)..]).unwrap();
                assert_eq!(label.as_bytes(), cstr.to_bytes());
            }
        }
        let space_width = exe[offset(0x80170b88) + 47 * 6 + 2];
        assert_eq!(
            fonts.width(Face::Speed, "  7"),
            2 * (i32::from(space_width) + 3) + 9 + 3
        );
    }
}
