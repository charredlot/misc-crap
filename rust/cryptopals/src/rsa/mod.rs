pub mod test;

extern crate gmp;

use asn1::PKCS1V15_SHA1_DIGEST_PREFIX;
use self::gmp::mpz::Mpz;
use sha1;
use util::{randomish_prime, bytes_to_mpz, mpz_bytes, mpz_byte_len,
           mpz_bytes_zero_pad};

#[derive(Debug)]
pub struct PublicKey {
    // pub for debugging
    pub e: Mpz,
    pub n: Mpz,
}

#[derive(Debug)]
pub struct PrivateKey {
    // pub for debugging
    pub d: Mpz,
    pub n: Mpz,
}

pub fn pkcs1v15_sha1_der_encode(msg: &[u8]) -> Vec<u8> {
    let mut der: Vec<u8> = Vec::new();
    der.extend_from_slice(&PKCS1V15_SHA1_DIGEST_PREFIX);
    der.extend_from_slice(&sha1::digest(msg));
    der
}

impl PublicKey {
    pub fn new(e: &Mpz, n: &Mpz) -> PublicKey {
        PublicKey{e: e.clone(), n: n.clone()}
    }

    pub fn encrypt_to_mpz(&self, msg: &[u8]) -> Mpz {
        let m = bytes_to_mpz(msg);
        m.powm(&self.e, &self.n)
    }

    pub fn encrypt(&self, msg: &[u8]) -> Vec<u8> {
        mpz_bytes(&self.encrypt_to_mpz(msg))
    }

    /// Doesn't properly check padding
    pub fn pkcs1v15_sha1_bad_verify(&self,
                                    msg: &[u8],
                                    signature: &[u8]) -> bool {
        // just panic in bad places for debugging
        let bytes = {
            let msg_num = &bytes_to_mpz(signature).powm(&self.e, &self.n);
            mpz_bytes_zero_pad(&msg_num, mpz_byte_len(&self.n))
        };

        if &bytes[..3] != &[0u8, 1u8, 0xffu8] {
            return false;
        }

        let der_index: usize = {
            // NB: seems like a spurious unused assignment warning.
            // we could totally hit a case where index isn't set in the loop
            let mut index = bytes.len();
            for (i, &b) in bytes[3..].iter().enumerate() {
                // purposely not checking padding values
                if b == 0u8 {
                    index = i + 1;
                    break;
                }
            }
            index + 3
        };

        let prefix_len = PKCS1V15_SHA1_DIGEST_PREFIX.len();
        if bytes.len() < (der_index + prefix_len + sha1::DIGEST_LENGTH) {
            // NB: we should check here on the good version that
            // it exactly equals bytes.len()
            return false;
        }

        if &bytes[der_index..der_index + prefix_len] !=
           &PKCS1V15_SHA1_DIGEST_PREFIX as &[u8] {
            return false;
        }

        let hash_index = der_index + prefix_len;
        &sha1::digest(&msg) == &bytes[hash_index..
                                      hash_index + sha1::DIGEST_LENGTH]
    }
}

impl PrivateKey {
    pub fn new(d: &Mpz, n: &Mpz) -> PrivateKey {
        PrivateKey{d: d.clone(), n: n.clone()}
    }

    pub fn decrypt_mpz(&self, ciphernum: &Mpz) -> Mpz {
        ciphernum.powm(&self.d, &self.n)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let m = bytes_to_mpz(ciphertext);
        mpz_bytes(&self.decrypt_mpz(&m))
    }

    pub fn pkcs1v15_sha1_sign(&self, msg: &[u8]) -> Vec<u8> {
        let der = pkcs1v15_sha1_der_encode(msg);

        let mut res: Vec<u8> = Vec::new();

        // type 1 is 0xff padding
        res.push(0u8);
        res.push(1u8);

        let bytes = mpz_byte_len(&self.n);
        // spec requires 8 padding bytes + 3 bytes (part of the encoding)
        assert!(bytes >= der.len() + 11);
        for _ in 0..bytes - der.len() - 3 {
            res.push(0xffu8);
        }

        // misc separator per RFC
        res.push(0u8);
        res.extend_from_slice(&der);
        mpz_bytes(&bytes_to_mpz(&res).powm(&self.d, &self.n))
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
