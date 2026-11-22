//! Test-only R3000 instruction subset. Unsupported instructions fail closed.
//! No BIOS, GPU, interrupts, cache timing, or game-runtime emulation.
pub struct Cpu {
    pub ram: Vec<u8>,
    pub r: [u32; 32],
    pub random_state: u32,
    hi: u32,
    lo: u32,
    lzcr: u32,
    gte: [u32; 32],
    pc: u32,
    next: u32,
    load: Option<(usize, u32)>,
}
impl Cpu {
    pub fn new(exe: &[u8]) -> Self {
        assert_eq!(&exe[..8], b"PS-X EXE");
        let base = u32::from_le_bytes(exe[24..28].try_into().unwrap());
        let size = u32::from_le_bytes(exe[28..32].try_into().unwrap()) as usize;
        let mut cpu = Self {
            ram: vec![0; 2 * 1024 * 1024 + 1024],
            r: [0; 32],
            random_state: 1,
            hi: 0,
            lo: 0,
            lzcr: 0,
            gte: [0; 32],
            pc: 0,
            next: 0,
            load: None,
        };
        cpu.put(base, &exe[2048..2048 + size]);
        cpu
    }
    fn offset(&self, addr: u32, size: usize) -> usize {
        let p = (addr & 0x1fff_ffff) as usize;
        if (0x1f800000..0x1f800400).contains(&p) && p + size <= 0x1f800400 {
            return 0x200000 + p - 0x1f800000;
        }
        assert!(p + size <= 0x200000, "unmapped address {addr:#x}");
        p
    }
    pub fn put(&mut self, addr: u32, bytes: &[u8]) {
        let p = self.offset(addr, bytes.len());
        self.ram[p..p + bytes.len()].copy_from_slice(bytes);
    }
    pub fn word(&self, addr: u32) -> u32 {
        assert_eq!(addr & 3, 0, "unaligned word");
        let p = self.offset(addr, 4);
        u32::from_le_bytes(self.ram[p..p + 4].try_into().unwrap())
    }
    fn half(&self, addr: u32) -> u16 {
        assert_eq!(addr & 1, 0, "unaligned halfword");
        let p = self.offset(addr, 2);
        u16::from_le_bytes(self.ram[p..p + 2].try_into().unwrap())
    }
    pub fn call(&mut self, address: u32, args: &[u32]) -> u32 {
        self.call_until(address, args, 0x8000_0080)
    }
    pub fn call_until(&mut self, address: u32, args: &[u32], stop: u32) -> u32 {
        self.call_observed(address, args, stop, |_| {})
    }
    pub fn call_observed(
        &mut self,
        address: u32,
        args: &[u32],
        stop: u32,
        mut before: impl FnMut(&Self),
    ) -> u32 {
        self.r = [0; 32];
        let register_count = args.len().min(4);
        self.r[4..4 + register_count].copy_from_slice(&args[..register_count]);
        self.r[28] = 0x8018_bf04; // JP global pointer set by the startup code.
        self.r[29] = 0x801f_f000;
        for (i, arg) in args.iter().enumerate().skip(4) {
            self.put(self.r[29] + 4 * i as u32, &arg.to_le_bytes());
        }
        self.r[31] = 0x8000_0080;
        self.pc = address;
        self.next = address + 4;
        self.load = None;
        for _ in 0..100_000 {
            if self.pc == stop {
                return self.r[2];
            }
            before(self);
            self.step();
        }
        panic!("instruction budget exhausted at {:#x}", self.pc);
    }
    pub fn pc(&self) -> u32 {
        self.pc
    }
    fn step(&mut self) {
        if self.pc == 0xa0 {
            assert_eq!(self.r[9], 0x2f, "unexpected BIOS call");
            self.random_state = self
                .random_state
                .wrapping_mul(1103515245)
                .wrapping_add(12345);
            self.r[2] = (self.random_state >> 16) & 32767;
            self.pc = self.r[31];
            self.next = self.pc + 4;
            return;
        }
        let pc = self.pc;
        let ins = self.word(pc);
        self.pc = self.next;
        self.next = self.next.wrapping_add(4);
        let old_load = self.load.take();
        let op = ins >> 26;
        let rs = ((ins >> 21) & 31) as usize;
        let rt = ((ins >> 16) & 31) as usize;
        let rd = ((ins >> 11) & 31) as usize;
        let shift = (ins >> 6) & 31;
        let a = self.r[rs];
        let b = self.r[rt];
        let imm = ins as u16 as i16 as i32 as u32;
        let addr = a.wrapping_add(imm);
        let mut write = None;
        let target = pc.wrapping_add(4).wrapping_add(imm << 2);
        match op {
            0 => match ins & 63 {
                0 => write = Some((rd, b << shift)),
                2 => write = Some((rd, b >> shift)),
                3 => write = Some((rd, ((b as i32) >> shift) as u32)),
                4 => write = Some((rd, b << (a & 31))),
                6 => write = Some((rd, b >> (a & 31))),
                7 => write = Some((rd, ((b as i32) >> (a & 31)) as u32)),
                8 => self.next = a,
                9 => {
                    self.next = a;
                    write = Some((rd, pc + 8));
                }
                16 => write = Some((rd, self.hi)),
                18 => write = Some((rd, self.lo)),
                24 => {
                    let product = i64::from(a as i32) * i64::from(b as i32);
                    self.lo = product as u32;
                    self.hi = (product >> 32) as u32;
                }
                25 => {
                    let product = u64::from(a) * u64::from(b);
                    self.lo = product as u32;
                    self.hi = (product >> 32) as u32;
                }
                26 => {
                    let (a, b) = (a as i32, b as i32);
                    (self.lo, self.hi) = if b == 0 {
                        (if a < 0 { 1 } else { u32::MAX }, a as u32)
                    } else {
                        (a.wrapping_div(b) as u32, a.wrapping_rem(b) as u32)
                    };
                }
                27 => {
                    self.lo = a.checked_div(b).unwrap_or(u32::MAX);
                    self.hi = a.checked_rem(b).unwrap_or(a);
                }
                32 => write = Some((rd, (a as i32).checked_add(b as i32).unwrap() as u32)),
                33 => write = Some((rd, a.wrapping_add(b))),
                34 => write = Some((rd, (a as i32).checked_sub(b as i32).unwrap() as u32)),
                35 => write = Some((rd, a.wrapping_sub(b))),
                36 => write = Some((rd, a & b)),
                37 => write = Some((rd, a | b)),
                38 => write = Some((rd, a ^ b)),
                39 => write = Some((rd, !(a | b))),
                42 => write = Some((rd, ((a as i32) < (b as i32)) as u32)),
                43 => write = Some((rd, (a < b) as u32)),
                _ => panic!("unsupported SPECIAL {ins:#x} at {pc:#x}"),
            },
            1 => {
                let taken = match rt {
                    0 => (a as i32) < 0,
                    1 => (a as i32) >= 0,
                    _ => panic!("unsupported REGIMM"),
                };
                if taken {
                    self.next = target;
                }
            }
            2 | 3 => {
                self.next = ((pc + 4) & 0xf000_0000) | ((ins & 0x03ff_ffff) << 2);
                if op == 3 {
                    write = Some((31, pc + 8));
                }
            }
            4 => {
                if a == b {
                    self.next = target;
                }
            }
            5 => {
                if a != b {
                    self.next = target;
                }
            }
            6 => {
                if (a as i32) <= 0 {
                    self.next = target;
                }
            }
            7 => {
                if (a as i32) > 0 {
                    self.next = target;
                }
            }
            8 => write = Some((rt, (a as i32).checked_add(imm as i32).unwrap() as u32)),
            9 => write = Some((rt, addr)),
            10 => write = Some((rt, ((a as i32) < (imm as i32)) as u32)),
            11 => write = Some((rt, (a < imm) as u32)),
            12 => write = Some((rt, a & (ins & 0xffff))),
            13 => write = Some((rt, a | (ins & 0xffff))),
            14 => write = Some((rt, a ^ (ins & 0xffff))),
            15 => write = Some((rt, ins << 16)),
            18 => match (rs, rd) {
                (4, 30) => {
                    self.lzcr = if (b as i32) < 0 {
                        (!b).leading_zeros()
                    } else {
                        b.leading_zeros()
                    }
                }
                (0, 31) => self.load = Some((rt, self.lzcr)),
                (4, 0..=29) => self.gte[rd] = b,
                (0, 0..=29) => self.load = Some((rt, self.gte[rd])),
                (16..=31, _) if matches!(ins & 63, 0x28 | 0x3d) => {
                    let shift = if ins & (1 << 19) != 0 { 12 } else { 0 };
                    for i in 1..=3 {
                        let ir = self.gte[8 + i] as i16 as i32;
                        let mul = if ins & 63 == 0x28 {
                            ir
                        } else {
                            self.gte[8] as i16 as i32
                        };
                        let mac = (ir * mul) >> shift;
                        self.gte[24 + i] = mac as u32;
                        self.gte[8 + i] =
                            mac.clamp(if ins & (1 << 10) != 0 { 0 } else { -32768 }, 32767) as u32;
                    }
                }
                _ => panic!("unsupported COP2 {ins:#x} at {pc:x}"),
            },
            58 => {
                let v = self.gte[rt];
                self.put(addr, &v.to_le_bytes());
            }
            50 => self.gte[rt] = self.word(addr),
            32 => self.load = Some((rt, self.ram[self.offset(addr, 1)] as i8 as i32 as u32)),
            33 => self.load = Some((rt, self.half(addr) as i16 as i32 as u32)),
            35 => self.load = Some((rt, self.word(addr))),
            36 => self.load = Some((rt, self.ram[self.offset(addr, 1)] as u32)),
            37 => self.load = Some((rt, self.half(addr) as u32)),
            40 => self.put(addr, &[b as u8]),
            41 => {
                assert_eq!(addr & 1, 0);
                self.put(addr, &(b as u16).to_le_bytes());
            }
            43 => {
                assert_eq!(addr & 3, 0);
                self.put(addr, &b.to_le_bytes());
            }
            _ => panic!("unsupported opcode {ins:#x} at {pc:#x}"),
        }
        if let Some((r, v)) = old_load {
            self.r[r] = v;
        }
        if let Some((r, v)) = write {
            self.r[r] = v;
        }
        self.r[0] = 0;
    }
}

#[test]
fn executes_branch_and_load_delay_slots() {
    let mut exe = vec![0; 2080];
    exe[..8].copy_from_slice(b"PS-X EXE");
    exe[24..28].copy_from_slice(&0x8001_0000u32.to_le_bytes());
    exe[28..32].copy_from_slice(&32u32.to_le_bytes());
    // v0=7; lw v0,0(a0); v1=v0; branch; v1+=1 (delay); skipped; return; v0+=v1.
    for (i, word) in [
        0x24020007u32,
        0x8c820000,
        0x00401821,
        0x10000002,
        0x24630001,
        0x24030063,
        0x03e00008,
        0x00431021,
    ]
    .iter()
    .enumerate()
    {
        exe[2048 + i * 4..2052 + i * 4].copy_from_slice(&word.to_le_bytes());
    }
    let mut cpu = Cpu::new(&exe);
    cpu.put(0x8002_0000, &20u32.to_le_bytes());
    assert_eq!(cpu.call(0x8001_0000, &[0x8002_0000]), 28);
}
