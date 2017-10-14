
const A: u32 = 0x9908B0DFu32;
const B: u32 = 0x9D2C5680u32;
const C: u32 = 0xEFC60000u32;
const D: u32 = 0xFFFFFFFFu32;
const F: u32 = 1812433253;
const L: usize = 18;
const M: usize = 397;
const N: usize = 624;
const R: usize = 31;
const S: usize = 7;
const T: usize = 15;
const U: usize = 11;
const W: usize = 32;

const MASK_UPPER: u32 = (1u32 << R);
const MASK_LOWER: u32 = (1u32 << R) - 1;

pub struct MT19937 {
    state: [u32; N],
    index: usize,
}

impl MT19937 {
    pub fn new(seed: u32) -> MT19937 {
        let mut mt = MT19937 {
            state: [0u32; N],
            index: N,
        };

        mt.state[0] = seed;
        for i in 1..mt.state.len() {
            mt.state[i] = F.wrapping_mul(mt.state[i - 1] ^
                                         (mt.state[i - 1] >> (W - 2)));
            mt.state[i] = mt.state[i].wrapping_add(i as u32);
        }
        mt
    }

    fn twist(&mut self) {
        for i in 0..N {
            let x = (self.state[i] & MASK_UPPER) +
                    (self.state[(i + 1) % N] & MASK_LOWER);
            let mut x_a = x >> 1;
            if x % 2 != 0 {
                x_a ^= A;
            }

            self.state[i] = self.state[(i + M) % N] ^ x_a;
        }
        self.index = 0;
    }

    pub fn extract32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }

        let mut y = self.state[self.index];
        self.index += 1;
        y ^= (y >> U) & D;
        y ^= (y << S) & B;
        y ^= (y << T) & C;
        y ^= y >> L;
        y
    }
}
