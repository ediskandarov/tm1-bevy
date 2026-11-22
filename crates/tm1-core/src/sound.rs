//! Voice selection in the JP SOUND.c mixer.

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
