extern crate gmp;
extern crate rand;

use std::time::{SystemTime, UNIX_EPOCH};

use self::gmp::mpz::Mpz;
use self::rand::Rng;
use hex::{hex_to_bytes, bytes_to_hex};

pub type EncryptOracle = Fn (&[u8]) -> Vec<u8>;
pub type DecryptOracle = Fn (&[u8]) -> Vec<u8>;

pub fn rand_bytes_range(begin: usize, end: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    rand_bytes(rng.gen_range(begin, end))
}

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

pub fn assert_slice_cmp(label: &'static str, expected: &[u8], got: &[u8]) {
    assert!(expected == got,
            concat!("\n{}\n",
                    "expected {:?}\n",
                    "     got {:?}\n"),
            label, expected, got);
}

pub fn mpz_bytes(mpz: &Mpz) -> Vec<u8> {
    hex_to_bytes(&mpz.to_str_radix(16))
}

pub fn bytes_to_mpz(bytes: &[u8]) -> Mpz {
    Mpz::from_str_radix(&bytes_to_hex(bytes), 16).unwrap()
}

// random-ish num less than the max
pub fn randomish_mpz_lt(max: &Mpz) -> Mpz {
    // TODO: ehhhhh this is a kludge, just make sure it's less than max
    let len = (max.bit_length() / 8) - 1;
    bytes_to_mpz(&rand_bytes(len))
}

pub fn randomish_prime(bit_len: usize) -> Mpz {
    // TODO: eh round to byte len maybe fix later
    let len = bit_len / 8;
    let num = bytes_to_mpz(&rand_bytes(len));
    num.nextprime()
}

pub fn mpz_byte_len(mpz: &Mpz) -> usize {
    let bit_len = mpz.bit_length();
    if bit_len % 8 == 0 {
        bit_len / 8
    } else {
        (bit_len + 8) / 8
    }
}

pub fn mpz_bytes_zero_pad(mpz: &Mpz, len: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    let actual_len = mpz_byte_len(mpz);
    if len > actual_len {
        for _ in 0..len - actual_len {
            bytes.push(0u8);
        }
    }
    bytes.extend(mpz_bytes(mpz));
    bytes
}

pub fn mpz_print_padded(mpz: &Mpz, byte_len: usize) {
    let bytes = mpz_bytes_zero_pad(mpz, byte_len);
    for chunk in bytes.chunks(16) {
        println!("{}", bytes_to_hex(chunk));
    }
}
