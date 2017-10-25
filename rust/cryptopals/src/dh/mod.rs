pub mod test;

extern crate gmp;

use self::gmp::mpz::Mpz;
use aes::AES_BLOCK_SIZE;
use util::mpz_bytes;

// finite-field diffie-hellman private to public
pub fn ff_dhe_public(private: &Mpz, generator: &Mpz, prime: &Mpz) -> Mpz {
    // (generator ^ private) % prime
    generator.powm(private, prime)
}

// private is b, peer's public is A
// A ^ b = (g ^ a) ^ b = g ^ (a * b) = g ^ (b * a) = B ^ a
pub fn ff_dhe_shared(private: &Mpz, peer_public: &Mpz, prime: &Mpz) -> Mpz {
    peer_public.powm(private, prime)
}

// aes128
pub fn ff_dhe_aes_key_adjust(raw_key: &Mpz) -> Vec<u8> {
    let mut key = mpz_bytes(raw_key);
    if key.len() <= AES_BLOCK_SIZE {
        // probably bad but just zeropad
        for _ in key.len()..AES_BLOCK_SIZE {
            key.push(0u8);
        }
        key
    } else {
        (&key[..AES_BLOCK_SIZE]).to_vec()
    }
}

pub fn ff_dhe_shared_aes_key(private: &Mpz,
                             peer_public: &Mpz,
                             prime: &Mpz) -> Vec<u8> {
    ff_dhe_aes_key_adjust(&ff_dhe_shared(private, peer_public, prime))
}
