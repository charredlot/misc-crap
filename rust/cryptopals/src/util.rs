extern crate rand;

use self::rand::Rng;

pub fn rand_bytes(len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut ret = Vec::new();
    for _i in 0..len {
        ret.push(rng.gen_range(0, 256 as usize) as u8);
    }
    ret
}

pub fn rand_key() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8;16];
    for i in 0..16 {
        key[i] = rng.gen_range(0, 256 as usize) as u8;
    }
    key
}
