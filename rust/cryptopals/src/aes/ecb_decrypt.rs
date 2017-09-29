extern crate rand;

use std::collections::HashMap;
use std::str;

use aes::AESCipher;
use base64::base64_decode;
use pkcs::pkcs7_pad;
use self::rand::Rng;

const ORACLE_SUFFIX: &'static str = "Um9sbGluJyBpbiBteSA1LjAKV2l0aCBteSByYWctdG9wIGRvd24gc28gbXkgaGFpciBjYW4gYmxvdwpUaGUgZ2lybGllcyBvbiBzdGFuZGJ5IHdhdmluZyBqdXN0IHRvIHNheSBoaQpEaWQgeW91IHN0b3A/IE5vLCBJIGp1c3QgZHJvdmUgYnkK";

const ORACLE_SUFFIX_STR: &'static str = "Rollin' in my 5.0\nWith my rag-top down so my hair can blow\nThe girlies on standby waving just to say hi\nDid you stop? No, I just drove by\n";

type EncryptOracle = Fn (&AESCipher, &[u8]) -> Vec<u8>;

fn encrypt_aes_ecb_suffix_oracle(cipher: &AESCipher,
                                 plaintext: &[u8]) -> Vec<u8> {
    let mut suffixed = plaintext.to_vec();
    suffixed.extend_from_slice(&base64_decode(ORACLE_SUFFIX));
    cipher.ecb_encrypt(&pkcs7_pad(&suffixed, 16))
}

fn encrypt_aes_ecb_sandwich_oracle(cipher: &AESCipher,
                                   prefix: &[u8],
                                   plaintext: &[u8]) -> Vec<u8> {
    let mut sandwich = prefix.to_vec();
    sandwich.extend_from_slice(plaintext);
    sandwich.extend_from_slice(&base64_decode(ORACLE_SUFFIX));
    cipher.ecb_encrypt(&pkcs7_pad(&sandwich, 16))
}

fn gen_encrypt_aes_ecb_sandwich_oracle(cipher: &AESCipher,
                                       prefix_len: usize) ->
                                            Box<EncryptOracle> {
    let mut rng = rand::thread_rng();
    let mut v: Vec<u8> = Vec::new();
    for i in 0..prefix_len {
        v.push(rng.gen_range(0, 256 as usize) as u8);
    }
    Box::new(move |cipher: &AESCipher, plaintext: &[u8]| -> Vec<u8> {
        encrypt_aes_ecb_sandwich_oracle(cipher, &v, plaintext)
    })
}

fn confirm_aes_ecb(cipher: &AESCipher) {
    let two_blocks = [0u8; 32];
    let ciphertext = encrypt_aes_ecb_suffix_oracle(cipher, &two_blocks);
    if ciphertext[0..16] != ciphertext[16..32] {
        panic!("expected aes ecb, got ciphertext {:?}", &ciphertext);
    }
}

// TODO: make encrypt_oracle take in a closure instead of cipher
fn get_aes_ecb_hidden_len(cipher: &AESCipher,
                          encrypt_oracle: &EncryptOracle) -> usize {
    let mut v: Vec<u8> = Vec::new();

    let mut result: usize = 0;
    let start_len = encrypt_oracle(cipher, &v).len();
    for i in 1..17 {
        v.push(0u8);
        let n = encrypt_oracle(cipher, &v).len();
        let diff = n - start_len;
        if diff > 0 {
            // if hidden_len % 16 == 0, adding 16 will give a 16 byte diff
            // if hidden_len % 16 == 15, adding 1 will give a 16 byte diff
            // if hidden_len % 16 == 14, adding 2 will give a 16 byte diff
            // etc.
            result = n - 16 - i;
            break;
        }
    }
    result
}

fn decrypt_aes_ecb_suffix(cipher: &AESCipher,
                          encrypt_oracle: &EncryptOracle,
                          pad_len: usize,
                          skip_len: usize,
                          suffix_len: usize) -> Vec<u8> {
    let mut block: Vec<u8> = Vec::new();
    let mut block_map: HashMap<Vec<u8>, u8>  = HashMap::new();
    let mut result: Vec<u8> = Vec::new();
    let mut block_index: usize = 0;

    for i in 0..pad_len + 16 {
        block.push(0u8);
    }

    for i in 0..suffix_len {
        block_map.clear();
        for j in 0..256 {
            let byte = j as u8;
            block[pad_len + 15] = byte;

            let ciphertext = encrypt_oracle(cipher, &block);
            // for some reason "j as u8" returns an Option
            block_map.insert((&ciphertext[skip_len..skip_len + 16]).to_vec(),
                             byte);
        }
        let end = pad_len + 15 - (i % 16);
        let ciphertext = encrypt_oracle(cipher, &block[0..end]);

        let cblock = skip_len + (block_index * 16);
        match block_map.get(&ciphertext[cblock..cblock + 16]) {
            Some(byte) => {
                block[pad_len + 15] = *byte;
                result.push(*byte);
            },
            None => panic!("block not found in map at byte {}", i),
        }
        // block is a sliding window, rotate left 1 every time
        for j in pad_len..pad_len + 15 {
            block[j] = block[j + 1];
        }
        if i % 16 == 15 {
            // figured out a block, go to the next ciphertext block
            block_index = block_index + 1;
        }
    }

    result
}

fn rand_key() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut key = [0u8;16];
    for i in 0..16 {
        key[i] = rng.gen_range(0, 256 as usize) as u8;
    }
    key
}

pub fn decrypt_aes_ecb_simple_test() {
    let key = rand_key();
    println!("AES ECB simple decrypt test with key {:?}", &key);

    let cipher = AESCipher::new(&key);
    confirm_aes_ecb(&cipher);
    let suffix_len = get_aes_ecb_hidden_len(&cipher,
                                            &encrypt_aes_ecb_suffix_oracle);
    let decrypted_bytes = {
        decrypt_aes_ecb_suffix(&cipher, &encrypt_aes_ecb_suffix_oracle,
                               0, 0, suffix_len)
    };
    let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
    if decrypted != ORACLE_SUFFIX_STR {
        panic!("decrypt aes ecb suffix failed, got {:?}", &decrypted_bytes);
    }
}

fn get_aes_ecb_prefix_len(cipher: &AESCipher,
                          encrypt_oracle: &EncryptOracle) -> usize {
    let tot_len = encrypt_oracle(cipher, &[]).len();

    const expect_blocks: usize = 3;
    let splitter = [7u8; expect_blocks * 16];
    let mut probe = splitter.to_vec();
    let mut prefix_len: Option<usize> = None;

    // number of consecutive blocks == expect_blocks:
    // if prefix_len % 16 == 0, 0 additional bytes
    // if prefix_len % 16 == 15, 1 additional bytes
    // if prefix_len % 16 == 14, 2 additional bytes
    'outer: for i in 0..16 {
        let test = encrypt_oracle(cipher, &probe);
        let mut consecutive = 1;
        let mut prev_block: Option<&[u8]> = None;
        for (j, block) in (&test).chunks(16).enumerate() {
            match prev_block {
                Some(prev) => {
                    if prev == block {
                        consecutive += 1;
                        if consecutive == expect_blocks {
                            // if matched 3 blocks on block 4,
                            // prefix plus extra bytes are in blocks
                            // 0 and 1. i is the extra bytes
                            prefix_len = Some((j - expect_blocks + 1) * 16 -
                                              i);
                            break 'outer;
                        }
                    } else {
                        consecutive = 1;
                    }
                },
                None => {},
            };
            prev_block = Some(block);
        }
        probe.insert(0, 6u8);
    }

    match prefix_len {
        Some(plen) => {
            plen
        },
        None => panic!("didn't find get_aes_ecb_prefix_len"),
    }
}

fn decrypt_aes_ecb_sandwich(cipher: &AESCipher,
                            encrypt_oracle: &EncryptOracle) -> Vec<u8> {
    let hidden_len = get_aes_ecb_hidden_len(cipher, encrypt_oracle);
    let prefix_len = get_aes_ecb_prefix_len(cipher, encrypt_oracle);
    let suffix_len = hidden_len - prefix_len;
    let rem = prefix_len % 16;
    decrypt_aes_ecb_suffix(&cipher,
                           encrypt_oracle,
                           if rem == 0 {
                               0
                           } else {
                               16 - rem
                           },
                           if rem == 0 {
                               prefix_len
                           } else {
                               prefix_len + 16 - rem
                           },
                           suffix_len)
}

pub fn decrypt_aes_ecb_sandwich_test() {
    let key = rand_key();
    println!("AES ECB sandwich decrypt test with key {:?}", &key);

    let cipher = AESCipher::new(&key);

    for i in 1..33 {
        let encrypt_oracle = gen_encrypt_aes_ecb_sandwich_oracle(&cipher, i);
        let decrypted_bytes = decrypt_aes_ecb_sandwich(&cipher,
                                                       &*encrypt_oracle);
        let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
        if decrypted != ORACLE_SUFFIX_STR {
            panic!("decrypt aes ecb sandwich failed, got {:?}",
                   &decrypted_bytes);
        }
        println!("Finished decrypt sandwich oracle for prefix size {}", i);
    }
}
