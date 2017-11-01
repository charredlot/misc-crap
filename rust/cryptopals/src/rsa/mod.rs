pub mod test;

extern crate gmp;

use self::gmp::mpz::Mpz;
use util::{randomish_prime, bytes_to_mpz, mpz_bytes};

pub struct PublicKey {
    // pub for debugging
    pub e: Mpz,
    pub n: Mpz,
}

pub struct PrivateKey {
    // pub for debugging
    pub d: Mpz,
    pub n: Mpz,
}

impl PublicKey {
    pub fn new(e: &Mpz, n: &Mpz) -> PublicKey {
        PublicKey{e: e.clone(), n: n.clone()}
    }

    pub fn encrypt(&self, msg: &[u8]) -> Vec<u8> {
        let m = bytes_to_mpz(msg);
        mpz_bytes(&m.powm(&self.e, &self.n))
    }
}

impl PrivateKey {
    pub fn new(d: &Mpz, n: &Mpz) -> PrivateKey {
        PrivateKey{d: d.clone(), n: n.clone()}
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let m = bytes_to_mpz(ciphertext);
        mpz_bytes(&m.powm(&self.d, &self.n))
    }
}

pub fn new_keypair(bit_len: usize) -> (PublicKey, PrivateKey) {
    loop {
        // roughly half the bit len should end with bit_len n
        let p = randomish_prime(bit_len / 2);
        let q = randomish_prime(bit_len / 2);
        let n = &p * &q;

        let totient = ((&p - Mpz::one()) * (&q - Mpz::one())).modulus(&n);
        // just hardcode e to be 3 for now
        let e = Mpz::one() + Mpz::one() + Mpz::one();

        //  e.g. if p = 11 and q = 7, totient == 60, so it won't be coprime
        //  with 3
        match e.invert(&totient) {
            Some(d) => {
                return (PublicKey::new(&e, &n), PrivateKey::new(&d, &n));
            },
            None => continue,
        };
    }
}
