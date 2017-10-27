pub mod test;

extern crate sha2;

use self::sha2::{Sha256, Digest};

pub fn salted_hash(salt: &[u8], identity: &[u8], password: &[u8]) -> Vec<u8> {
    let mut input = salt.to_vec();
    input.extend_from_slice(identity);
    input.extend_from_slice(password);

    Sha256::digest(&input).to_vec()
}
