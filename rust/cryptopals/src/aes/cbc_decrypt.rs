extern crate rand;

use std::str;

use aes::AESCipher;
use util::{rand_key, EncryptOracle, DecryptOracle};

// XXX: should be okay to be constant for this?
const SSV_IV: [u8; 16] = [7u8; 16];
const SSV_PREFIX: &'static str = "comment1=cooking%20MCs;userdata=";
const SSV_SUFFIX: &'static str = ";comment2=%20like%20a%20pound%20of%20bacon";

// ssv: semicolon-separated values
fn get_encrypt_aes_cbc_ssv_oracle(key: &[u8]) -> Box<EncryptOracle> {
    let cipher = AESCipher::new(key);
    Box::new(move |plaintext: &[u8]| -> Vec<u8> {
        let mut result = SSV_PREFIX.as_bytes().to_vec();
        result.extend_from_slice(plaintext);
        result.extend_from_slice(SSV_SUFFIX.as_bytes());
        cipher.cbc_pad_and_encrypt(&result, &SSV_IV)
    })
}

fn get_decrypt_aes_cbc_ssv(key: &[u8]) -> Box<DecryptOracle> {
    let cipher = AESCipher::new(key);
    Box::new(move |ciphertext: &[u8]| -> Vec<u8> {
        cipher.cbc_decrypt_and_unpad(ciphertext, &SSV_IV)
    })
}

fn decrypt_aes_cbc_ssv_bitflip_test() {
    let key = rand_key();
    println!("AES CBC semicolon values decrypt test with key {:?}", &key);
    let key = rand_key();
    let encrypt_oracle = get_encrypt_aes_cbc_ssv_oracle(&key);
    let decrypt_oracle = get_decrypt_aes_cbc_ssv(&key);
    let ciphertext = encrypt_oracle("beepboopbop".as_bytes());
    let decrypted_bytes = decrypt_oracle(&ciphertext);
    println!("boop {}", str::from_utf8(&decrypted_bytes).unwrap());
}

pub fn decrypt_aes_cbc_test() {
    decrypt_aes_cbc_ssv_bitflip_test();
    println!("Finished AES CBC decrypt tests");
}
