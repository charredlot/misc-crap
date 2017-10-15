extern crate rand;

use std::time::{SystemTime, UNIX_EPOCH};

use self::rand::Rng;

pub type EncryptOracle = Fn (&[u8]) -> Vec<u8>;
pub type DecryptOracle = Fn (&[u8]) -> Vec<u8>;

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

pub fn rand_u64() -> u64 {
    let mut rng = rand::thread_rng();
    // TODO: maybe use u128 when it's available to get the max value + 1
    rng.gen_range(0, u64::max_value())
}

pub fn unix_timestamp_sec() -> i64{
    // this is painful :/
    let now = SystemTime::now();
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(_) => 0 as i64, // can't handle negative times for now :/
    }
}
