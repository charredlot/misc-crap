use std::cmp;
use std::str;
use hex::{bytes_to_hex, hex_to_bytes};

pub fn fixed_xor(buf: &[u8], key: &[u8]) -> Vec<u8> {
    let l = cmp::min(buf.len(), key.len());
    let mut vec: Vec<u8> = Vec::with_capacity(l);
    for (&b, &k) in buf.iter().zip(key) {
        vec.push(b ^ k);
    }
    vec
}

const ENGLISH_BYTE_SCALE: u64 = 10000;
const ENGLISH_BYTE_FREQS: [u64; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1076, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 724, 132, 241, 385, 1072, 205, 180, 528, 651, 9, 61, 355, 233, 619, 685,
    162, 10, 537, 560, 811, 256, 98, 186, 15, 188, 6, 0, 0, 0, 0, 0,
    0, 724, 132, 241, 385, 1072, 205, 180, 528, 651, 9, 61, 355, 233, 619, 685,
    162, 10, 537, 560, 811, 256, 98, 186, 15, 188, 6, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn english_freq_score(freqs: &[u64; 256], total: u64) -> u64 {
    let mut score: u64 = 0;
    for (&freq, &expected_freq) in freqs.iter().zip(ENGLISH_BYTE_FREQS.iter()) {
        let normalized_freq = (freq * ENGLISH_BYTE_SCALE) / total;
        let diff = (expected_freq as i64) - (normalized_freq as i64);
        // chi squared doesn't behave well if expected values are 0
        // could do fisher's exact test or barnard's exact test but seems overkill
        // so just do simple diff addition
        score += diff.abs() as u64;
    }
    score
}

/// Returns decrypted, key, score
pub fn decrypt_byte_xor_cipher(buf: &[u8]) -> (Vec<u8>, u8, u64) {
    let mut low_score: u64 = u64::max_value();
    let mut low_key: u8 = 0;

    for i in 0..256 {
        let key = i as u8;
        let mut freq = [0u64; 256];

        for &b in buf {
            let candidate = b ^ key;
            freq[candidate as usize] += 1;
        }

        let score = english_freq_score(&freq, buf.len() as u64);
        if score < low_score {
            // XXX: same score?
            low_score = score;
            low_key = key;
        }
    }
    let mut vec = Vec::with_capacity(buf.len());
    for &b in buf {
        vec.push(b ^ low_key);
    }
    (vec, low_key, low_score)
}

fn fixed_xor_test() {
    let buf = hex_to_bytes("1c0111001f010100061a024b53535009181c");
    let key = hex_to_bytes("686974207468652062756c6c277320657965");
    let answer = "746865206b696420646f6e277420706c6179";

    let result = fixed_xor(buf.as_slice(), key.as_slice());

    let s = bytes_to_hex(result.as_slice());
    if s == answer {
        println!("Finished fixed_xor_test");
    } else {
        println!("ERROR in fixed_xor_test:");
        println!("  expected {}", answer);
        println!("  got      {}", s);
    }
}

fn byte_xor_cipher_test(ciphertext: &str, plaintext: &str) {
    let cipher_bytes = hex_to_bytes(ciphertext);
    let (decrypted_bytes, _, _) = decrypt_byte_xor_cipher(&cipher_bytes);

    // this might panic?
    let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
    if decrypted == plaintext {
        println!("Finished byte_xor_cipher_test for {}", ciphertext);
    } else {
        println!("ERROR in byte_xor_cipher_test:");
        println!("  expected {}", plaintext);
        println!("  got      {}", decrypted);
    }
}

pub fn xor_test() {
    fixed_xor_test();
    byte_xor_cipher_test("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736",
                         "Cooking MC's like a pound of bacon");
}
