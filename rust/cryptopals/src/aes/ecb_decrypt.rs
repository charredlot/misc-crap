extern crate rand;

use std::collections::HashMap;
use std::str;

use aes::{AESCipher, AES_BLOCK_SIZE};
use base64::base64_decode;
use util::{rand_key, rand_bytes, EncryptOracle};

const ORACLE_SUFFIX: &'static str = "Um9sbGluJyBpbiBteSA1LjAKV2l0aCBteSByYWctdG9wIGRvd24gc28gbXkgaGFpciBjYW4gYmxvdwpUaGUgZ2lybGllcyBvbiBzdGFuZGJ5IHdhdmluZyBqdXN0IHRvIHNheSBoaQpEaWQgeW91IHN0b3A/IE5vLCBJIGp1c3QgZHJvdmUgYnkK";

const ORACLE_SUFFIX_STR: &'static str = "Rollin' in my 5.0\nWith my rag-top down so my hair can blow\nThe girlies on standby waving just to say hi\nDid you stop? No, I just drove by\n";

fn get_encrypt_aes_ecb_suffix_oracle(key: &[u8]) -> Box<EncryptOracle> {
    let cipher = AESCipher::new(key);

    Box::new(move |plaintext: &[u8]| -> Vec<u8> {
        let mut suffixed = plaintext.to_vec();
        suffixed.extend_from_slice(&base64_decode(ORACLE_SUFFIX));
        cipher.ecb_pad_and_encrypt(&suffixed)
    })
}

fn gen_encrypt_aes_ecb_sandwich_oracle(key: &[u8],
                                       prefix_len: usize) ->
                                            Box<EncryptOracle> {
    let v = rand_bytes(prefix_len);

    let cipher = AESCipher::new(key);
    Box::new(move |plaintext: &[u8]| -> Vec<u8> {
        let mut sandwich = v.to_vec();
        sandwich.extend_from_slice(plaintext);
        sandwich.extend_from_slice(&base64_decode(ORACLE_SUFFIX));
        cipher.ecb_pad_and_encrypt(&sandwich)
    })
}

fn confirm_aes_ecb(encrypt_oracle: &EncryptOracle) {
    let two_blocks = [0u8; 32];
    let ciphertext = encrypt_oracle(&two_blocks);
    if ciphertext[..AES_BLOCK_SIZE] !=
        ciphertext[AES_BLOCK_SIZE..(AES_BLOCK_SIZE * 2)] {
        panic!("expected aes ecb, got ciphertext {:?}", &ciphertext);
    }
}

// TODO: make encrypt_oracle take in a closure instead of cipher
fn get_aes_ecb_hidden_len(encrypt_oracle: &EncryptOracle) -> usize {
    let mut v: Vec<u8> = Vec::new();

    let mut result: usize = 0;
    let start_len = encrypt_oracle(&v).len();
    for i in 1..(AES_BLOCK_SIZE + 1) {
        v.push(0u8);
        let n = encrypt_oracle(&v).len();
        let diff = n - start_len;
        if diff > 0 {
            // if hidden_len % 16 == 0, adding 16 will give a 16 byte diff
            // if hidden_len % 16 == 15, adding 1 will give a 16 byte diff
            // if hidden_len % 16 == 14, adding 2 will give a 16 byte diff
            // etc.
            result = n - AES_BLOCK_SIZE - i;
            break;
        }
    }
    result
}

fn decrypt_aes_ecb_suffix(encrypt_oracle: &EncryptOracle,
                          pad_len: usize,
                          skip_len: usize,
                          suffix_len: usize) -> Vec<u8> {
    let mut block: Vec<u8> = Vec::new();
    let mut block_map: HashMap<Vec<u8>, u8>  = HashMap::new();
    let mut result: Vec<u8> = Vec::new();
    let mut block_index: usize = 0;

    for _ in 0..(pad_len + AES_BLOCK_SIZE) {
        block.push(0u8);
    }

    for i in 0..suffix_len {
        block_map.clear();
        for j in 0..256 {
            let byte = j as u8;
            block[pad_len + 15] = byte;

            let ciphertext = encrypt_oracle(&block);
            // for some reason "j as u8" returns an Option
            let block_end = skip_len + AES_BLOCK_SIZE;
            let block_key = (&ciphertext[skip_len..block_end]).to_vec();
            block_map.insert(block_key, byte);
        }
        let end = pad_len + 15 - (i % AES_BLOCK_SIZE);
        let ciphertext = encrypt_oracle(&block[0..end]);

        let cblock = skip_len + (block_index * AES_BLOCK_SIZE);
        match block_map.get(&ciphertext[cblock..(cblock + AES_BLOCK_SIZE)]) {
            Some(byte) => {
                block[pad_len + AES_BLOCK_SIZE - 1] = *byte;
                result.push(*byte);
            },
            None => panic!("block not found in map at byte {}", i),
        }
        // block is a sliding window, rotate left 1 every time
        for j in pad_len..pad_len + AES_BLOCK_SIZE - 1 {
            block[j] = block[j + 1];
        }
        if i % AES_BLOCK_SIZE == AES_BLOCK_SIZE - 1 {
            // figured out a block, go to the next ciphertext block
            block_index = block_index + 1;
        }
    }

    result
}

pub fn decrypt_aes_ecb_simple_test() {
    let key = rand_key();
    println!("AES ECB simple decrypt test with key {:?}", &key);

    let encrypt_oracle = get_encrypt_aes_ecb_suffix_oracle(&key);
    confirm_aes_ecb(&*encrypt_oracle);
    let suffix_len = get_aes_ecb_hidden_len(&*encrypt_oracle);
    let decrypted_bytes = {
        decrypt_aes_ecb_suffix(&*encrypt_oracle, 0, 0, suffix_len)
    };
    let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
    if decrypted != ORACLE_SUFFIX_STR {
        panic!("decrypt aes ecb suffix failed, got {:?}", &decrypted_bytes);
    }
}

fn get_aes_ecb_prefix_len(encrypt_oracle: &EncryptOracle) -> usize {
    const EXPECT_BLOCKS: usize = 3;
    let splitter = [7u8; EXPECT_BLOCKS * AES_BLOCK_SIZE];
    let mut probe = splitter.to_vec();
    let mut prefix_len: Option<usize> = None;

    // number of consecutive blocks == EXPECT_BLOCKS:
    // if prefix_len % 16 == 0, 0 additional bytes
    // if prefix_len % 16 == 15, 1 additional bytes
    // if prefix_len % 16 == 14, 2 additional bytes
    'outer: for i in 0..AES_BLOCK_SIZE {
        let test = encrypt_oracle(&probe);
        let mut consecutive = 1;
        let mut prev_block: Option<&[u8]> = None;
        for (j, block) in (&test).chunks(AES_BLOCK_SIZE).enumerate() {
            match prev_block {
                Some(prev) => {
                    if prev == block {
                        consecutive += 1;
                        if consecutive == EXPECT_BLOCKS {
                            // if matched 3 blocks on block 4,
                            // prefix plus extra bytes are in blocks
                            // 0 and 1. i is the extra bytes
                            let block_index = j - EXPECT_BLOCKS + 1;
                            let block_end = block_index * AES_BLOCK_SIZE;
                            prefix_len = Some(block_end - i);
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

fn decrypt_aes_ecb_sandwich(encrypt_oracle: &EncryptOracle) -> Vec<u8> {
    let hidden_len = get_aes_ecb_hidden_len(encrypt_oracle);
    let prefix_len = get_aes_ecb_prefix_len(encrypt_oracle);
    let suffix_len = hidden_len - prefix_len;
    let rem = prefix_len % AES_BLOCK_SIZE;
    decrypt_aes_ecb_suffix(encrypt_oracle,
                           if rem == 0 {
                               0
                           } else {
                               AES_BLOCK_SIZE - rem
                           },
                           if rem == 0 {
                               prefix_len
                           } else {
                               prefix_len + AES_BLOCK_SIZE - rem
                           },
                           suffix_len)
}

pub fn decrypt_aes_ecb_sandwich_test() {
    let key = rand_key();
    println!("AES ECB sandwich decrypt test with key {:?}", &key);

    for i in 1..33 {
        let encrypt_oracle = gen_encrypt_aes_ecb_sandwich_oracle(&key, i);
        let decrypted_bytes = decrypt_aes_ecb_sandwich(&*encrypt_oracle);
        let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
        if decrypted != ORACLE_SUFFIX_STR {
            panic!("decrypt aes ecb sandwich failed, got {:?}",
                   &decrypted_bytes);
        }
        println!("Finished decrypt sandwich oracle for prefix size {}", i);
    }
}
