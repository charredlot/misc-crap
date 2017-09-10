extern crate rand;

use aes::AESCipher;
use pkcs::pkcs7_pad;
use self::rand::Rng;

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
    pkcs7_pad(&result, 16)
}

fn aes_cbc_ecb_random_encrypt(plaintext: &[u8]) -> (&'static str, Vec<u8>) {
    let mut rng = rand::thread_rng();
    let key = [rng.gen_range(0, 256) as u8; 16];
    let padded = random_bookend(plaintext);
    let cipher = AESCipher::new(&key);

    // XXX: couldn't get closure boxing to work
    match rng.gen_range(0, 2) {
            0 => ("cbc",
                  {
                      let iv: [u8; 16] = [rng.gen_range(0, 256) as u8; 16];
                      cipher.cbc_encrypt(&padded, &iv)
                  }),
            1 => ("ecb", cipher.ecb_encrypt(&padded)),
            _ => panic!("welp"),  // should be unreachable
    }
}

fn distinguish_oracle_aes_cbc_ecb(ciphertext: &[u8]) -> &'static str{
    if ciphertext[16..32] == ciphertext [32..48] {
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
