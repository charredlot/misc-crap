use std::collections::HashMap;
use std::str;

use aes::AESCipherOld;
use pkcs7::pkcs7_pad;

pub fn url_decode(params: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut word: Vec<char> = Vec::new();
    let mut key = None;

    fn insert(map: &mut HashMap<String, String>,
              key: Option<String>,
              word: Vec<char>) { 
        match key {
            Some(key_str) => {
                map.insert(key_str, word.into_iter().collect());
            },
            None => {
                if word.len() == 0 {
                    panic!("bad && or ends with &");
                }
                map.insert(word.into_iter().collect(), "".to_string());
            },
        };
    };

    for c in params.chars() {
        if c == '=' {
            if word.len() == 0 {
                panic!("bad &= or started with =");
            }

            key = Some(word.into_iter().collect());
            word = Vec::new();
        } else if c == '&' {
            insert(&mut map, key, word);
            key = None;
            word = Vec::new();
        } else {
            word.push(c);
        }
    }
    insert(&mut map, key, word);
    map
}

fn profile_for(email: &str) -> String {
    let mut s = String::new();
    s.push_str("email=");
    s.push_str(email);
    s.push_str("&uid=10&role=user");
    s
}

fn encrypt_profile_for(cipher: &AESCipherOld, email: &str) -> Vec<u8> {
    cipher.ecb_pad_and_encrypt(profile_for(email).as_bytes())
}

fn decrypt_profile_for(cipher: &AESCipherOld,
                       ciphertext: &[u8]) -> HashMap<String, String> {
    let buf = cipher.ecb_decrypt_and_unpad(ciphertext);
    let s = str::from_utf8(&buf).unwrap();
    url_decode(s)
}

fn trick_url_decode(cipher: &AESCipherOld) {
    // &uid=10&role= is 13 characters, so we need 3 more to form middle block
    // 012345678901234567890123456789
    // email=foo01@bar.com&uid=10&role=
    // block 0 is email=foo01@bar.
    // block 1 is com&uid=10&role=
    // block 2 is admin
    let email = "foo01@bar.com";
    let blocks_0_and_1 = encrypt_profile_for(cipher, email);
    
    // email= is 6 chars, so we need 10 bytes to form the first block
    // the email part is throwaway
    let mut block2_email = "123456789@".to_string();

    // pkcs7 pad admin to make it like the last block
    let last_block = pkcs7_pad("admin".as_bytes(), 16);
    block2_email.push_str(str::from_utf8(&last_block).unwrap());

    // second block should be pkc7 padded admin
    let block2 = &encrypt_profile_for(cipher, &block2_email)[16..32];

    let mut chosen = Vec::new();
    chosen.extend(blocks_0_and_1[0..32].iter());
    chosen.extend(block2.iter());

    let out = decrypt_profile_for(cipher, &chosen);
    match out.get("role") {
        Some(role) => {
            if role == "admin" {
                println!("PASSED: trick_url_decode");
            } else {
                panic!("FAILED: trick_url_decode got role {}", role);
            }

        },
        None => panic!("FAILED: trick_url_decode no role found!"),
    };
}

pub fn url_test() {
    let map = url_decode("boop=1&beep=bop&meow=cat");
    println!("url_decode: {:?}", map);

    let key = "YELLOW SUBMARINE".as_bytes();
    let cipher = AESCipherOld::new(key);
    let ciphertext = encrypt_profile_for(&cipher, "foo@bar.com");
    let out = decrypt_profile_for(&cipher, &ciphertext);
    println!("foo@bar.com encrypt and decrypt: {:?}", out);

    trick_url_decode(&cipher);
    println!("Finished url decode tests");
}
