pub mod test;

extern crate gmp;

use self::gmp::mpz::Mpz;

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
