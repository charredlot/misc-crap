extern crate rand;

use aes::{AESCipher, AES_BLOCK_SIZE};
use aes::cbc::AESCipherCBC;
use aes::ecb::AESCipherECB;
use pkcs7::pkcs7_pad;
use self::rand::Rng;
use util::rand_key;

fn random_bookend(buf: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut result: Vec<u8> = Vec::new();

    let before = rng.gen_range(5, 11);
    for _ in 0..before {
        result.push(rng.gen_range(0, 256 as usize) as u8);
    }

    for &b in buf {
        result.push(b);
    }

    let after = rng.gen_range(5, 11);
    for _ in 0..after {
        result.push(rng.gen_range(0, 256 as usize) as u8);
    }

    // XXX: version that takes vec?
    pkcs7_pad(&result, AES_BLOCK_SIZE)
}

fn aes_cbc_ecb_random_encrypt(plaintext: &[u8]) -> (&'static str, Vec<u8>) {
    let key = rand_key();
    let padded = random_bookend(plaintext);

    // XXX: couldn't get closure boxing to work
    let mut rng = rand::thread_rng();
    match rng.gen_range(0, 2) {
            0 => ("cbc",
                  {
                      let iv = vec![rng.gen_range(0, 256) as u8;
                                    AES_BLOCK_SIZE];
                      let cipher = AESCipherCBC::new(&key, &iv);
                      cipher.encrypt(&padded)
                  }),
            1 => ("ecb", {
                let cipher = AESCipherECB::new(&key);
                cipher.encrypt(&padded)
            }),
            _ => panic!("welp"),  // should be unreachable
    }
}

fn distinguish_oracle_aes_cbc_ecb(ciphertext: &[u8]) -> &'static str{
    if ciphertext[AES_BLOCK_SIZE..(AES_BLOCK_SIZE * 2)] ==
        ciphertext[(AES_BLOCK_SIZE * 2)..(AES_BLOCK_SIZE * 3)] {
        "ecb"
    } else {
        "cbc"
    }
}

pub fn distinguish_aes_cbc_ecb_test() {
    let runs = 8;
    // goal is to get 2 identical blocks
    // min padding on each side is 5 bytes, so need 11 bytes on both sides
    // need 32 additional bytes to form 2 identical blocks
    // using same bytes ensures offsets aren't a problem
    let input = [11u8; 54];
    for _ in 0..runs {
        let (mode, ciphertext) = aes_cbc_ecb_random_encrypt(&input);
        let guess_mode = distinguish_oracle_aes_cbc_ecb(&ciphertext);
        if mode != guess_mode {
            panic!("expected {} got {} for {:?}", mode, guess_mode, ciphertext);
        }
    }
    println!("Finished {} runs of distinguish AES CBC and ECB", runs);
}
