use std::cmp::max;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str;

use aes::AESCipher;
use aes::ctr::AESCipherCTR;
use base64::base64_decode;
use charfreq::EnglishCharScore;
use util::rand_key;
use xor::slice_xor;

const SET_3_CHALLENGE_19: &'static [&'static str] = &[
    "SSBoYXZlIG1ldCB0aGVtIGF0IGNsb3NlIG9mIGRheQ==",
    "Q29taW5nIHdpdGggdml2aWQgZmFjZXM=",
    "RnJvbSBjb3VudGVyIG9yIGRlc2sgYW1vbmcgZ3JleQ==",
    "RWlnaHRlZW50aC1jZW50dXJ5IGhvdXNlcy4=",
    "SSBoYXZlIHBhc3NlZCB3aXRoIGEgbm9kIG9mIHRoZSBoZWFk",
    "T3IgcG9saXRlIG1lYW5pbmdsZXNzIHdvcmRzLA==",
    "T3IgaGF2ZSBsaW5nZXJlZCBhd2hpbGUgYW5kIHNhaWQ=",
    "UG9saXRlIG1lYW5pbmdsZXNzIHdvcmRzLA==",
    "QW5kIHRob3VnaHQgYmVmb3JlIEkgaGFkIGRvbmU=",
    "T2YgYSBtb2NraW5nIHRhbGUgb3IgYSBnaWJl",
    "VG8gcGxlYXNlIGEgY29tcGFuaW9u",
    "QXJvdW5kIHRoZSBmaXJlIGF0IHRoZSBjbHViLA==",
    "QmVpbmcgY2VydGFpbiB0aGF0IHRoZXkgYW5kIEk=",
    "QnV0IGxpdmVkIHdoZXJlIG1vdGxleSBpcyB3b3JuOg==",
    "QWxsIGNoYW5nZWQsIGNoYW5nZWQgdXR0ZXJseTo=",
    "QSB0ZXJyaWJsZSBiZWF1dHkgaXMgYm9ybi4=",
    "VGhhdCB3b21hbidzIGRheXMgd2VyZSBzcGVudA==",
    "SW4gaWdub3JhbnQgZ29vZCB3aWxsLA==",
    "SGVyIG5pZ2h0cyBpbiBhcmd1bWVudA==",
    "VW50aWwgaGVyIHZvaWNlIGdyZXcgc2hyaWxsLg==",
    "V2hhdCB2b2ljZSBtb3JlIHN3ZWV0IHRoYW4gaGVycw==",
    "V2hlbiB5b3VuZyBhbmQgYmVhdXRpZnVsLA==",
    "U2hlIHJvZGUgdG8gaGFycmllcnM/",
    "VGhpcyBtYW4gaGFkIGtlcHQgYSBzY2hvb2w=",
    "QW5kIHJvZGUgb3VyIHdpbmdlZCBob3JzZS4=",
    "VGhpcyBvdGhlciBoaXMgaGVscGVyIGFuZCBmcmllbmQ=",
    "V2FzIGNvbWluZyBpbnRvIGhpcyBmb3JjZTs=",
    "SGUgbWlnaHQgaGF2ZSB3b24gZmFtZSBpbiB0aGUgZW5kLA==",
    "U28gc2Vuc2l0aXZlIGhpcyBuYXR1cmUgc2VlbWVkLA==",
    "U28gZGFyaW5nIGFuZCBzd2VldCBoaXMgdGhvdWdodC4=",
    "VGhpcyBvdGhlciBtYW4gSSBoYWQgZHJlYW1lZA==",
    "QSBkcnVua2VuLCB2YWluLWdsb3Jpb3VzIGxvdXQu",
    "SGUgaGFkIGRvbmUgbW9zdCBiaXR0ZXIgd3Jvbmc=",
    "VG8gc29tZSB3aG8gYXJlIG5lYXIgbXkgaGVhcnQs",
    "WWV0IEkgbnVtYmVyIGhpbSBpbiB0aGUgc29uZzs=",
    "SGUsIHRvbywgaGFzIHJlc2lnbmVkIGhpcyBwYXJ0",
    "SW4gdGhlIGNhc3VhbCBjb21lZHk7",
    "SGUsIHRvbywgaGFzIGJlZW4gY2hhbmdlZCBpbiBoaXMgdHVybiw=",
    "VHJhbnNmb3JtZWQgdXR0ZXJseTo=",
    "QSB0ZXJyaWJsZSBiZWF1dHkgaXMgYm9ybi4=",
];

fn nonce_reuse_test(ciphertexts: &Vec<Vec<u8>>) {
    // key ^ plaintextA = ciphertextA
    // => key ^ ciphertextA = plaintextA
    // so guess values for key that gives us best character
    // distribution in English across all the ciphertexts

    let max_key_len = (&ciphertexts).iter().map(|x| x.len()).fold(0, max);

    let mut key_guess: Vec<u8> = Vec::new();
    for i in 0..max_key_len {
        let mut score: EnglishCharScore<u8> = EnglishCharScore::new();
        for j in 0..256 {
            let guess = j as u8;

            // oop should've reused byte xor cipher code
            // but this is more readable in hindsight
            for ciphertext in ciphertexts {
                if i >= ciphertext.len() {
                    continue;
                }

                score.add_byte(ciphertext[i] ^ guess);
            }

            score.update_best(guess);
        }
        let (_, best_byte) = score.get_best();
        key_guess.push(best_byte.unwrap());
    }

    for ciphertext in ciphertexts {
        let plaintext = slice_xor(&key_guess, &ciphertext);
        let plaintext_str = str::from_utf8(&plaintext).unwrap();
        println!("\"{}\"", plaintext_str);
        // XXX: this has some fuckups at the end of long lines because
        // there aren't enough values to guess. live with it for now
    }

    println!("Ending nonce reuse AES CTR test");
}

fn nonce_reuse_test_19() {
    let key = rand_key();
    let cipher = AESCipherCTR::new(&key, 0);
    println!("Starting nonce reuse AES CTR test 19 with key {:?}", &key);

    let mut ciphertexts = Vec::new();
    for &plaintext in SET_3_CHALLENGE_19 {
        let ciphertext = cipher.encrypt(&base64_decode(plaintext));
        ciphertexts.push(ciphertext);
    }
    nonce_reuse_test(&ciphertexts);
}

fn nonce_reuse_test_20() {
    let key = rand_key();
    let cipher = AESCipherCTR::new(&key, 0);
    println!("Starting nonce reuse AES CTR test 20 with key {:?}", &key);

    let mut ciphertexts = Vec::new();

    let f = File::open("data/3.20.txt").unwrap();
    let buffered = BufReader::new(&f);
    for rline in buffered.lines() {
        let line = rline.unwrap();
        let ciphertext = cipher.encrypt(&base64_decode(&line));
        ciphertexts.push(ciphertext);
    }

    nonce_reuse_test(&ciphertexts);
}

pub fn decrypt_aes_ctr_test() {
    let encoded = "L77na/nrFsKvynd6HzOoG7GHTLXsTVu9qvY/2syLXzhPweyyMTJULu/6/kXX0KSvoOLSFQ==";
    let expected = "Yo, VIP Let's kick it Ice, Ice, baby Ice, Ice, baby ";

    let ciphertext = base64_decode(encoded);
    let cipher = AESCipherCTR::new("YELLOW SUBMARINE".as_bytes(), 0);

    let decrypted = cipher.decrypt(&ciphertext);
    match str::from_utf8(&decrypted) {
        Ok(_) => {},
        Err(_) => {
            panic!("ERROR: AES CTR expected {}\ngot {:?}",
                   expected, decrypted);
        }
    }

    nonce_reuse_test_19();
    nonce_reuse_test_20();
    println!("Finished AES CTR tests");
}
