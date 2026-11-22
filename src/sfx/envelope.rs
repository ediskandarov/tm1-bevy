//! SPU key-on volume envelope, clocked at the output sample rate.
#[derive(Clone, Copy)]
enum Phase {
    Attack,
    Decay,
    Sustain,
}

pub(super) struct Envelope {
    registers: [u16; 2],
    phase: Phase,
    level: i32,
    counter: u32,
}

impl Envelope {
    pub(super) fn new(registers: [u16; 2]) -> Self {
        Self {
            registers,
            phase: Phase::Attack,
            level: 0,
            counter: 0,
        }
    }

    pub(super) fn apply(&mut self, sample: i32) -> i32 {
        let output = (sample * self.level) >> 15;
        let [a, s] = self.registers;
        let (rate, decreasing, exponential) = match self.phase {
            Phase::Attack => ((a >> 8) & 127, false, a & 0x8000 != 0),
            Phase::Decay => (((a >> 4) & 15) * 4, true, true),
            Phase::Sustain => ((s >> 6) & 127, s & 0x4000 != 0, s & 0x8000 != 0),
        };
        if rate == 127 {
            return output;
        }
        let shift = rate / 4;
        let mut increment = (0x8000u32 >> shift.saturating_sub(11)).max(1);
        let magnitude = 7 - i32::from(rate & 3);
        let mut step =
            (if decreasing { !magnitude } else { magnitude }) << 11u16.saturating_sub(shift);
        if exponential && decreasing {
            step = (step * self.level) >> 15;
        } else if exponential && self.level >= 0x6000 {
            if rate < 44 {
                step >>= if rate < 40 { 2 } else { 1 };
            }
            if rate >= 40 {
                increment >>= if rate < 44 { 1 } else { 2 };
            }
        }
        self.counter += increment;
        if self.counter >= 0x8000 {
            self.counter = 0;
            self.level = (self.level + step).clamp(0, 32767);
            match self.phase {
                Phase::Attack if self.level == 32767 => self.phase = Phase::Decay,
                Phase::Decay if self.level < ((i32::from(a & 15) + 1) * 2048).min(32767) => {
                    self.phase = Phase::Sustain
                }
                _ => {}
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_disc_envelope_reaches_stationary_sustain() {
        let mut env = Envelope::new([0x80ff, 0x1fc0]);
        let levels: Vec<_> = (0..100).map(|_| -env.apply(-32768)).collect();
        assert_eq!(&levels[..4], &[0, 14336, 28672, 32256]);
        assert!(levels[4..20].iter().all(|&v| v == 32767));
        assert!(levels[20..].iter().all(|&v| v == 32759));
    }
}
