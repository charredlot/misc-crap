
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

    pub fn state_size() -> usize {
        N
    }

    pub fn clone_from_state(state: &[u32]) -> MT19937 {
        assert!(state.len() == N);
        let mut mt = MT19937 {
            state: [0u32; N],
            index: N,
        };
        mt.state.clone_from_slice(state);
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

        let y = self.state[self.index];
        self.index += 1;
        MT19937::temper(y)
    }

    pub fn temper(initial: u32) -> u32 {
        let mut y = initial;
        //println!("boop temper {:08x}", y);
        y ^= (y >> U) & D;
        //println!("boop temper {:08x}", y);
        y ^= (y << S) & B;
        //println!("boop temper {:08x}", y);
        y ^= (y << T) & C;
        //println!("boop temper {:08x}", y);
        y ^= y >> L;
        //println!("boop temper {:08x}", y);
        y
    }

    pub fn untemper(initial: u32) -> u32 {
        let mut y = initial;
        //println!("boop untemper {:08x}", y);

        // y ^= y >> 18
        // top 14 bits are xor'd into bottom 14 bits
        // that means top 18 bits are the same, so
        // we can just do the same thing to invert
        y ^= y >> L;
        //println!("boop untemper {:08x}", y);

        // y ^= (y << T) & C;
        // y ^= (y << 15) & 0xEFC60000
        // bottom 15 bits are unchanged
        // can get next 15 bits by doing transform again
        // now that we have orig bits 0..30, we can do transform again
        // to get the top 2 bits
        // trick is to only modify bits we care about
        //
        // new_y31 = orig_y31 ^ (orig_y16 & C31)
        // new_y16 = orig_y16 ^ (orig_y01 & C31)
        // orig_y01 = orig_y01
        // =>
        // orig_y31 = new_y31 ^ (orig_y16 & C31)
        // orig_y31 = new_y31 ^ ((new_y16 ^ (orig_y01 & C16)) & C31)
        // restore bits 15..30 by doing the transform and masking
        // then only 2 bits left, can do it again to get the last 2
        let mut mask: u32 = (1 << T) - 1;
        // bits 15..30
        mask = mask << T;
        y = ((y ^ ((y << T) & C)) & mask) | (y & !mask);
        // bits 30..32
        mask = mask << T;
        y = ((y ^ ((y << T) & C)) & mask) | (y & !mask);
        //println!("boop untemper {:08x}", y);

        // y ^= (y << S) & B;
        // y ^= (y << 7) & 0x9D2C5680
        mask = (1 << S) - 1;
        mask = mask << S;
        y = ((y ^ ((y << S) & B)) & mask) | (y & !mask);
        // bits 14..21
        mask = mask << S;
        y = ((y ^ ((y << S) & B)) & mask) | (y & !mask);
        // bits 21..28
        mask = mask << S;
        y = ((y ^ ((y << S) & B)) & mask) | (y & !mask);
        // bits 28..32
        mask = mask << S;
        y = ((y ^ ((y << S) & B)) & mask) | (y & !mask);
        //println!("boop untemper {:08x}", y);

        // y ^= (y >> U) & D;
        // y ^= (y >> 11)
        // (D is just 32-bit mask)
        // same thing as above in reverse
        mask = ((1 << U) - 1) << (32 - U);
        // bits 21..32 are unchanged
        // bits 10..21
        mask = mask >> U;
        y = ((y ^ (y >> U)) & mask) | (y & !mask);
        // bits 0..10
        mask = mask >> U;
        y = ((y ^ (y >> U)) & mask) | (y & !mask);
        //println!("boop untemper {:08x}", y);

        y
    }

    pub fn encrypt(seed: u16, plaintext: &[u8]) -> Vec<u8> {
        let mut ciphertext = Vec::with_capacity(plaintext.len());

        let mut mt = MT19937::new(seed as u32);
        for chunk in plaintext.chunks(4) {
            let key32 = mt.extract32();
            for (i, &b) in chunk.iter().enumerate() {
                let shift = (3 - i) * 8;
                ciphertext.push(((key32 >> shift) & 0xFF) as u8 ^ b);
            }
        }
        ciphertext
    }

    pub fn decrypt(seed: u16, ciphertext: &[u8]) -> Vec<u8> {
        // since it's an xor stream cipher, same logic to reverse
        MT19937::encrypt(seed, ciphertext)
    }
}
