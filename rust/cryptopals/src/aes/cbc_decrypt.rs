extern crate rand;

use std::str;

use aes::{AESCipherOld, AES_BLOCK_SIZE};
use pkcs7::{pkcs7_pad, pkcs7_unpad, pkcs7_maybe_unpad_copy};
use util::{rand_key, rand_bytes, EncryptOracle, DecryptOracle};

// XXX: should be okay to be constant for this?
const SSV_IV: [u8; 16] = [7u8; 16];
const SSV_PREFIX: &'static str = "comment1=cooking%20MCs;userdata=";
const SSV_SUFFIX: &'static str = ";comment2=%20like%20a%20pound%20of%20bacon";

const ADMIN_STR: &'static str = ";admin=true;";

// ssv: semicolon-separated values
fn get_encrypt_aes_cbc_ssv_oracle(key: &[u8]) -> Box<EncryptOracle> {
    let cipher = AESCipherOld::new(key);
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
    let cipher = AESCipherOld::new(key);
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

fn aes_cbc_decrypt_padding_oracle(cipher: &AESCipherOld,
                                  iv: &[u8],
                                  ciphertext: &[u8]) ->
                                    Result<Vec<u8>, String> {
    let padded = cipher.cbc_decrypt(iv, ciphertext);
    match pkcs7_maybe_unpad_copy(&padded, AES_BLOCK_SIZE) {
        Ok(_) => Ok(padded),
        Err(e) => Err(e),
    }
}

fn disambiguate_padding(cipher: &AESCipherOld,
                        prev_block: &[u8],
                        block: &[u8],
                        i: usize) -> bool {
    // we want the one that gives us padding of [1]
    // however, if the block actually has [3, 3, x], we
    // might accidentally succeed with [3, 3, 3]
    // disambiguate by flipping high bits on all
    // the earlier bytes...e.g. if the actual data is
    // padded with 15 bytes like [15, 15, 15 ...],
    // then flipping the 2nd byte would cause an error
    let mut false_positive = false;
    let mut c1 = prev_block.to_vec();
    for p_i in 0..i {
        c1[p_i] = 0xffu8;
        match aes_cbc_decrypt_padding_oracle(cipher, &c1, block) {
            Ok(_) => {},
            Err(_) => {
                false_positive = true;
                break;
            },
        };
    }
    false_positive
}

fn aes_cbc_padding_oracle_guess_block(cipher: &AESCipherOld,
                                      prev_block: &[u8],
                                      block: &[u8]) -> Vec<u8> {
    // assume we have ciphertext blocks c1, c2 and plaintext p1, p2
    // p2 = decrypt(c2) ^ c1
    //
    // for the last byte, the only valid byte if we screw up decryption is 1.
    // so we can brute force until padding is valid.
    // XXX: corner case if original padding is 1, what are the other corners?
    // once we do so, we can assume:
    // 1 = decrypt(c2)[15] ^ brute_forced_byte
    //
    // then we solve with the 2 constraints to get p2 back out
    // let dc2 = decrypt(c2)
    //
    // xor both sides of orig equality by 1:
    //  p2[15] ^ 1 = dc2[15] ^ c1[15] ^ 1
    // replace 1 with our brute forced equality :
    //  p2[15] ^ 1 = dc2[15] ^ c1[15] ^ (dc2[15] ^ brute_forced_byte)
    // xor is transitive:
    //  p2[15] ^ 1 = dc2[15] ^ dc2[15] ^ c1[15] ^ brute_forced_byte
    // for all a, a ^ a == 0:
    //  p2[15] ^ 1 = c1[15] ^ brute_forced_byte
    // xor both sides by 1, done:
    //  p2[15] = c1[15] ^ brute_forced_byte ^ 1
    //
    // once we know p2[15], we can recover decrypt(c2)[15] and choose b
    // s.t. decrypt(c2)[15] ^ b = anything
    // so repeat with padding [2, 2], [3, 3, 3], [4, 4, 4, 4] etc.

    // plain_block is p2
    let mut plain_block = vec![0u8; AES_BLOCK_SIZE];
    // decrypt_block is decrypt(c2)
    let mut decrypt_block = vec![0u8; AES_BLOCK_SIZE];

    for i in (0..AES_BLOCK_SIZE).rev() {
        let mut c1 = prev_block.to_vec();

        // if we're looking at the n - 3 byte, we need to set n - 2 and
        // n - 1 to 0x3 for proper padding
        let pad = (AES_BLOCK_SIZE - i) as u8;
        for j in (i + 1)..AES_BLOCK_SIZE {
            // pad = decrypt(c2) ^ c1
            c1[j] = pad ^ decrypt_block[j];
        }

        let mut found = false;
        let c1_byte = c1[i];

        // in case nothing matches, set defaults
        plain_block[i] = pad;
        decrypt_block[i] = pad ^ c1_byte;
        for j in 0..256 {
            let b = j as u8;
            if b == c1_byte {
                continue;
            }

            c1[i] = b;
            match aes_cbc_decrypt_padding_oracle(cipher, &c1, block) {
                Ok(_) => {
                    let mut p2_byte = c1_byte ^ b ^ pad;
                    if found {
                        assert!(i == AES_BLOCK_SIZE - 1,
                                concat!("cbc padding ambiguity {}:",
                                        "prev {} curr {}\n{:?}\n{:?}\n{:?}"),
                                i, plain_block[i], p2_byte,
                                c1, plain_block, decrypt_block);

                        let false_positive = disambiguate_padding(cipher,
                                                                  &c1,
                                                                  block,
                                                                  i);
                        if false_positive {
                            p2_byte = plain_block[i];
                        }
                    }
                    plain_block[i] = p2_byte;
                    decrypt_block[i] = p2_byte ^ c1_byte;
                    found = true;
                },
                Err(_) => {},
            }
        }
    }

    plain_block
}

fn decrypt_aes_cbc_padding_test(plaintext: &[u8]) {
    // TODO: redo AESCipherOld as a trait :/
    let key = rand_key();
    let cipher = AESCipherOld::new(&key);
    let iv = rand_bytes(AES_BLOCK_SIZE);

    let mut ciphertext = cipher.cbc_pad_and_encrypt(&iv, plaintext);

    println!("aes cbc padding oracle decrypt {:?}", plaintext);
    println!("key: {:?}", &key);
    println!("initial ciphertext:");
    for chunk in (&ciphertext).chunks(AES_BLOCK_SIZE) {
        println!("{:?}", chunk);
    }

    let mut decrypted = aes_cbc_padding_oracle_guess_block(&cipher, &iv,
                                            &ciphertext[..AES_BLOCK_SIZE]);
    let mut chunks = Vec::new();
    for chunk in (&ciphertext).chunks(AES_BLOCK_SIZE) {
        chunks.push(chunk);
    }
    for w in chunks.windows(2) {
        decrypted.extend(
            aes_cbc_padding_oracle_guess_block(&cipher, w[0], w[1]));
    }

    // XXX: make slice compare and panic assert macro
    assert!(pkcs7_pad(&plaintext, AES_BLOCK_SIZE) == decrypted,
            "\nplaintext: {:?}\ndecrypted: {:?}", plaintext, decrypted);
}

pub fn decrypt_aes_cbc_test() {
    decrypt_aes_cbc_ssv_bitflip_test();
    let plaintexts = [
        "1234567890123\x03\x036", // test disambiguating [1] vs [3, 3, 3]
        "\x16\x16\x16\x16\x16\x16\x16\x16\x16\x16\x16\x16\x16\x16\x16",
        "MDAwMDAwTm93IHRoYXQgdGhlIHBhcnR5IGlzIGp1bXBpbmc=",
        "MDAwMDAxV2l0aCB0aGUgYmFzcyBraWNrZWQgaW4gYW5kIHRoZSBWZWdhJ3MgYXJlIHB1bXBpbic=",
        "MDAwMDAyUXVpY2sgdG8gdGhlIHBvaW50LCB0byB0aGUgcG9pbnQsIG5vIGZha2luZw==",
        "MDAwMDAzQ29va2luZyBNQydzIGxpa2UgYSBwb3VuZCBvZiBiYWNvbg==",
        "MDAwMDA0QnVybmluZyAnZW0sIGlmIHlvdSBhaW4ndCBxdWljayBhbmQgbmltYmxl",
        "MDAwMDA1SSBnbyBjcmF6eSB3aGVuIEkgaGVhciBhIGN5bWJhbA==",
        "MDAwMDA2QW5kIGEgaGlnaCBoYXQgd2l0aCBhIHNvdXBlZCB1cCB0ZW1wbw==",
        "MDAwMDA3SSdtIG9uIGEgcm9sbCwgaXQncyB0aW1lIHRvIGdvIHNvbG8=",
        "MDAwMDA4b2xsaW4nIGluIG15IGZpdmUgcG9pbnQgb2g=",
        "MDAwMDA5aXRoIG15IHJhZy10b3AgZG93biBzbyBteSBoYWlyIGNhbiBibG93",
    ];
    for plaintext in &plaintexts {
        decrypt_aes_cbc_padding_test(&plaintext.as_bytes());
    }
    println!("Finished AES CBC decrypt tests");
}
