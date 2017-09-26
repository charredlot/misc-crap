extern crate rand;

use std::collections::HashMap;
use std::str;

use aes::AESCipher;
use base64::base64_decode;
use pkcs::pkcs7_pad;
use self::rand::Rng;

const ORACLE_SUFFIX: &'static str = "Um9sbGluJyBpbiBteSA1LjAKV2l0aCBteSByYWctdG9wIGRvd24gc28gbXkgaGFpciBjYW4gYmxvdwpUaGUgZ2lybGllcyBvbiBzdGFuZGJ5IHdhdmluZyBqdXN0IHRvIHNheSBoaQpEaWQgeW91IHN0b3A/IE5vLCBJIGp1c3QgZHJvdmUgYnkK";

const ORACLE_SUFFIX_STR: &'static str = "Rollin' in my 5.0\nWith my rag-top down so my hair can blow\nThe girlies on standby waving just to say hi\nDid you stop? No, I just drove by\n";

fn encrypt_aes_ecb_suffix_oracle(cipher: &AESCipher,
                                 plaintext: &[u8]) -> Vec<u8> {
    let mut suffixed = plaintext.to_vec();
    suffixed.extend_from_slice(&base64_decode(ORACLE_SUFFIX));
    cipher.ecb_encrypt(&pkcs7_pad(&suffixed, 16))
}

fn confirm_aes_ecb(cipher: &AESCipher) {
    let two_blocks = [0u8; 32];
    let ciphertext = encrypt_aes_ecb_suffix_oracle(cipher, &two_blocks);
    if ciphertext[0..16] != ciphertext[16..32] {
        panic!("expected aes ecb, got ciphertext {:?}", &ciphertext);
    }
}

fn get_aes_ecb_suffix_len(cipher: &AESCipher) -> usize {
    let mut v: Vec<u8> = Vec::new();

    let mut result: usize = 0;
    let start_len = encrypt_aes_ecb_suffix_oracle(cipher, &v).len();
    for i in 1..17 {
        v.push(0u8);
        let n = encrypt_aes_ecb_suffix_oracle(cipher, &v).len();
        let diff = n - start_len;
        if diff > 0 {
            // if suffix_len % 16 == 0, adding 16 will give a 16 byte diff
            // if suffix_len % 16 == 15, adding 1 will give a 16 byte diff
            // if suffix_len % 16 == 14, adding 2 will give a 16 byte diff
            // etc.
            result = n - 16 - i;
            break;
        }
    }
    result
}

fn decrypt_aes_ecb_suffix(cipher: &AESCipher) -> Vec<u8> {
    let n = get_aes_ecb_suffix_len(cipher);
    let mut block = [0u8; 16];
    let mut block_map: HashMap<Vec<u8>, u8>  = HashMap::new();
    let mut result: Vec<u8> = Vec::new();
    let mut block_index: usize = 0;

    for i in 0..n {
        block_map.clear();
        for j in 0..256 {
            let byte = j as u8;
            block[15] = byte;

            let ciphertext = encrypt_aes_ecb_suffix_oracle(cipher, &block);
            // for some reason "j as u8" returns an Option
            block_map.insert((&ciphertext[0..16]).to_vec(), byte);
        }
        let end = 15 - (i % 16);
        let ciphertext = encrypt_aes_ecb_suffix_oracle(cipher, &block[0..end]);

        let cblock = block_index * 16;
        match block_map.get(&ciphertext[cblock..cblock + 16]) {
            Some(byte) => {
                block[15] = *byte;
                result.push(*byte);
            },
            None => panic!("oh noes"),
        }
        // block is a sliding window, rotate left 1 every time
        for j in 0..15 {
            block[j] = block[j + 1];
        }
        if i % 16 == 15 {
            // figured out a block, go to the next ciphertext block
            block_index = block_index + 1;
        }
    }

    result
}

pub fn decrypt_aes_ecb_simple_test() {
    let mut rng = rand::thread_rng();
    let mut key = [0u8;16];
    for i in 0..16 {
        key[i] = rng.gen_range(0, 256 as usize) as u8;
    }
    println!("AES ECB simple decrypt test with key {:?}", &key);

    let cipher = AESCipher::new(&key);
    confirm_aes_ecb(&cipher);
    let decrypted_bytes = decrypt_aes_ecb_suffix(&cipher);
    let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
    if decrypted != ORACLE_SUFFIX_STR {
        panic!("decrypt aes ecb suffix failed, got {:?}", &decrypted_bytes);
    }
}
