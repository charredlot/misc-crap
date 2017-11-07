pub mod test;

extern crate gmp;

use asn1::{TagType, SHA1_OID};
use self::gmp::mpz::Mpz;
use sha1;
use util::{randomish_prime, bytes_to_mpz, mpz_bytes};

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

    der.push(TagType::Sequence as u8);
    // length placeholder, fill later
    der.push(0u8);

    // sequence of SHA1_OID and a null for parameters
    der.push(TagType::Sequence as u8);
    der.push((2 + SHA1_OID.len() + 2) as u8);

    der.push(TagType::OID as u8);
    der.push(SHA1_OID.len() as u8);
    der.extend_from_slice(&SHA1_OID);
    der.push(TagType::Null as u8);
    der.push(0u8);

    let sha1_hash = sha1::digest(msg);
    der.push(TagType::OctetString as u8);
    der.push(sha1_hash.len() as u8);
    der.extend_from_slice(&sha1_hash);

    // XXX: for bigger hashes might be a prob?
    assert!(der.len() < 128);
    // 1 byte for tag type and 1 byte for len
    der[1] = (der.len() - 2) as u8;
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
            // TODO: this is probably broken, but gotta zero pad at front
            // because leading zeroes get killed by the bignum library
            let decrypted = {
                mpz_bytes(&bytes_to_mpz(signature).powm(&self.e, &self.n))
            };
            assert_eq!(decrypted.len(),
                       ((self.n.bit_length() + 8) / 8) - 1);

            let mut tmp = Vec::new();
            tmp.push(0u8);
            tmp.extend_from_slice(&decrypted);
            tmp
        };

        if &bytes[..3] != &[0u8, 1u8, 0xffu8] {
            return false;
        }

        let mut der_index: usize = {
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
        let mut len: usize = 0;

        // TODO: blechhhhh should do this cleaner
        if der_index >= bytes.len() {
            return false;
        }

        if bytes[der_index] != TagType::Sequence as u8 {
            return false;
        }
        der_index += 1;

        // just check this len, but ignore it otherwise
        len = bytes[der_index] as usize;
        der_index += 1;
        if der_index + len > bytes.len() {
            return false;
        }

        if bytes[der_index] != TagType::Sequence as u8 {
            return false;
        }
        der_index += 1;

        // TODO: this sequence is hash algorithm and params, which
        // we should verify, but we'll just skip over it for now
        // (+1 is for the length byte itself)
        der_index += bytes[der_index] as usize + 1;
        if der_index >= bytes.len() {
            return false;
        }

        if bytes[der_index] != TagType::OctetString as u8 {
            return false;
        }
        der_index += 1;

        len = bytes[der_index] as usize;
        der_index += 1;
        if der_index + len > bytes.len() {
            // NB: we should check here on the good version that
            // it exactly equals bytes.len()
            return false;
        }


        assert_eq!(&sha1::digest(&msg), &bytes[der_index..]);
        &sha1::digest(&msg) == &bytes[der_index..der_index + len]
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

        let bytes = (self.n.bit_length() + 8) / 8;
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
