//! Voice selection in the JP SOUND.c mixer.

/// Seven-bit volume passed to SsUtSetVVol by soundProcessIds and soundPlaySound.
pub fn voice_volume(program: u8, tone: u8, channel: u8, percent: u8) -> u8 {
    let program = u32::from(channel) * u32::from(program) / 99;
    let tone = (program * u32::from(tone) / 99) as u8;
    let scaled = (u32::from(tone) * u32::from(percent) / 99) as u8;
    scaled.min(127)
}

/// Pitch register written by the JP SDK for a voice with sample note 60.
pub fn spu_pitch(note: u8, fine: u8) -> u16 {
    let step = usize::from(note.min(127)) * 16 + usize::from(fine.min(127)) / 8;
    let table = PITCH_TABLE
        .get_or_init(|| std::array::from_fn(|i| (4096.0 * 2.0_f64.powf(i as f64 / 192.0)) as u16));
    let octave = (step / 192) as i32 - 5;
    let mantissa = u32::from(table[step % 192]);
    if octave >= 0 {
        (mantissa << octave) as u16
    } else {
        (mantissa >> -octave) as u16
    }
}
static PITCH_TABLE: std::sync::OnceLock<[u16; 192]> = std::sync::OnceLock::new();

/// Sort the requested voices in tonesToPlay list order. Equal keys undergo
/// both original partitions, which can change who gets the last SPU channel.
pub fn sort_requested<T>(voices: &mut [T], range: impl Fn(&T) -> u32, priority: impl Fn(&T) -> u8) {
    fn partition<T, K: Ord>(items: &mut [T], key: &impl Fn(&T) -> K) {
        if items.len() < 2 {
            return;
        }
        items.swap(0, (items.len() - 1) / 2);
        let pivot = key(&items[0]);
        let mut boundary = 0;
        for i in 1..items.len() {
            if key(&items[i]) <= pivot {
                boundary += 1;
                items.swap(boundary, i);
            }
        }
        items.swap(0, boundary);
        let (left, right) = items.split_at_mut(boundary);
        partition(left, key);
        partition(&mut right[1..], key);
    }
    partition(voices, &range);
    partition(voices, &priority);
}
