//! Host integration probe for the unchanged upstream timer module.
//! This is not yet a gameplay core or a hardware timer emulator.
use std::sync::Mutex;
pub mod bullet;
pub mod drive;
pub mod math;
pub mod sound;

pub const TICKS_PER_FIELD: u32 = 70_618;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerSnapshot {
    pub frame: u32,
    pub elapsed_ticks: u32,
    pub fields: i32,
    pub update_rate: i32,
}
unsafe extern "C" {
    fn tm1_timer_reset();
    fn tm1_timer_step(delta: u32) -> TimerSnapshot;
}
static TIMER: Mutex<()> = Mutex::new(());

/// Each call starts a fresh replay. The mutex owns access to C globals.
pub fn replay_ticks(deltas: &[u32]) -> Vec<TimerSnapshot> {
    let _guard = TIMER.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        tm1_timer_reset();
    }
    deltas
        .iter()
        .map(|&delta| unsafe { tm1_timer_step(delta) })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_timer_rounding_and_clamp() {
        let snapshots = replay_ticks(&[0, 35_308, 35_309, 70_618, 211_854, 353_090, 423_708]);
        let pairs: Vec<_> = snapshots
            .iter()
            .map(|s| (s.fields, s.update_rate))
            .collect();
        assert_eq!(
            pairs,
            [
                (3, 20),
                (3, 20),
                (1, 60),
                (1, 60),
                (3, 20),
                (5, 12),
                (5, 12)
            ]
        );
        assert_eq!(snapshots.last().unwrap().frame, 7);
    }
    #[test]
    fn unsigned_clock_wrap_and_replay_reset() {
        let snapshots = replay_ticks(&[u32::MAX - 100, 211_854]);
        assert_eq!(snapshots[1].elapsed_ticks, 211_854);
        assert_eq!(snapshots[1].fields, 3);
        assert_eq!(replay_ticks(&[211_854])[0].frame, 1);
    }
}
