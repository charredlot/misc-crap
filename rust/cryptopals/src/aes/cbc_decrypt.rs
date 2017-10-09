extern crate rand;

use aes::{AESCipher, AES_BLOCK_SIZE};
use util::{rand_key, EncryptOracle, DecryptOracle};

// XXX: should be okay to be constant for this?
const SSV_IV: [u8; 16] = [7u8; 16];
const SSV_PREFIX: &'static str = "comment1=cooking%20MCs;userdata=";
const SSV_SUFFIX: &'static str = ";comment2=%20like%20a%20pound%20of%20bacon";

const ADMIN_STR: &'static str = ";admin=true;";

// ssv: semicolon-separated values
fn get_encrypt_aes_cbc_ssv_oracle(key: &[u8]) -> Box<EncryptOracle> {
    let cipher = AESCipher::new(key);
    Box::new(move |plaintext: &[u8]| -> Vec<u8> {
        let mut result = SSV_PREFIX.as_bytes().to_vec();
        // XXX: should strip/escape semicolons and equal signs but
        // too lazy
        result.extend_from_slice(plaintext);
        result.extend_from_slice(SSV_SUFFIX.as_bytes());
        cipher.cbc_pad_and_encrypt(&SSV_IV, &result)
    })
}

fn get_decrypt_aes_cbc_ssv(key: &[u8]) -> Box<DecryptOracle> {
    let cipher = AESCipher::new(key);
    Box::new(move |ciphertext: &[u8]| -> Vec<u8> {
        cipher.cbc_decrypt_and_unpad(&SSV_IV, ciphertext)
    })
}

// XXX: should take a string but we may cause invalid utf8
fn has_admin_true(buf: &[u8]) -> bool {
    let match_bytes = ADMIN_STR.as_bytes();
    let match_len = match_bytes.len();
    if match_len > buf.len() {
        return false;
    }

    for i in 0..(buf.len() - 1 - match_len) {
        if &buf[i..(i + match_len)] == match_bytes {
            return true;
        }
    }
    false
}

fn decrypt_aes_cbc_ssv_bitflip_test() {
    let key = rand_key();
    println!("AES CBC semicolon values decrypt test with key {:?}", &key);
    let key = rand_key();
    let encrypt_oracle = get_encrypt_aes_cbc_ssv_oracle(&key);
    let decrypt_oracle = get_decrypt_aes_cbc_ssv(&key);

    // need to flip the byte between admin and true to an '='
    // and the first byte to ';'
    // use '-' == 0x2d for '=' == 0x3d
    // use '+' == 0x2b for ';' == 0x3b
    // add a garbage block in front so we don't modify anything important
    let mut plaintext: Vec<u8> = vec!['A' as u8; AES_BLOCK_SIZE];
    plaintext.extend_from_slice("+admin-true".as_bytes());
    let mut ciphertext = encrypt_oracle(&plaintext);

    let mod_byte = |buf: &mut Vec<u8>, target_byte: usize, xor: u8| {
        let mod_block = (target_byte / AES_BLOCK_SIZE) - 1;
        let mod_byte = (mod_block * AES_BLOCK_SIZE) +
            (target_byte % AES_BLOCK_SIZE);
        buf[mod_byte] = buf[mod_byte] ^ xor;
    };

    // XXX: technically we should try not to generate invalid utf8?
    // it should be possible to munge our garbage block so it generates
    // valid utf8 after a bit flip, but it doesn't work in general since
    // they'll usually generate a random iv

    // prefix + garbage block for the ';'
    mod_byte(&mut ciphertext, SSV_PREFIX.len() + AES_BLOCK_SIZE, 0x10u8);

    // prefix + garbage block plus up to end of admin for the '='
    mod_byte(&mut ciphertext,
             SSV_PREFIX.len() + AES_BLOCK_SIZE + ";admin".len(),
             0x10u8);

    let decrypted_bytes = decrypt_oracle(&ciphertext);

    // debugging
    {
        let mut expected = SSV_PREFIX.as_bytes().to_vec();
        expected.extend_from_slice(&plaintext);
        expected.extend_from_slice(SSV_SUFFIX.as_bytes());
        for (expect_chunk, decrypt_chunk) in
             expected.chunks(AES_BLOCK_SIZE).zip(
              decrypted_bytes.chunks(AES_BLOCK_SIZE)) {
            println!("boop0 {:?}", expect_chunk);
            println!("boop1 {:?}", decrypt_chunk);
        }
    }
    assert!(has_admin_true(&decrypted_bytes));
    println!("Finished AES CBC bitflip test");
}

pub fn decrypt_aes_cbc_test() {
    decrypt_aes_cbc_ssv_bitflip_test();
    println!("Finished AES CBC decrypt tests");
}
