use std::cmp;
use std::str;
use base64::base64_decode_file;
use xor::{guess_byte_xor_cipher, repeating_key_xor};

pub fn hamming_distance(l: &[u8], r: &[u8]) -> u32 {
    let mut num_bits: u32 = 0;
    for (&b0, &b1) in l.iter().zip(r) {
        let res: u8 = b0 ^ b1;
        num_bits += res.count_ones();
    }
    num_bits
}

pub fn guess_repeating_key_xor_size(ciphertext: &[u8]) -> Vec<usize> {
    const MIN_KEY_LEN: usize = 2;
    const MAX_KEY_LEN: usize = 40;
    let mut scores: Vec<(usize, u32)> = Vec::new();

    for i in MIN_KEY_LEN..MAX_KEY_LEN {
        if i >= ciphertext.len() {
            // need at least 1 block and remainder to be meaningful
            break;
        }

        let mut prev_chunk: Vec<u8> = Vec::with_capacity(i);
        // XXX: better to use option prob?
        let mut num_chunks: u32 = 0;
        let mut total: u32 = 0;

        for chunk in ciphertext.chunks(i) {
            if num_chunks > 0 {
                let len = cmp::min(i, chunk.len()) as u32;
                total += hamming_distance(chunk, &prev_chunk) * 1000 / len;
            }

            prev_chunk.truncate(0);
            for &b in chunk.iter() {
                prev_chunk.push(b);
            }
            num_chunks += 1;
        }

        scores.push((i, total / num_chunks));
    }
    scores.sort_by(|l, r| l.1.cmp(&r.1));

    let mut result: Vec<usize> = Vec::new();
    for (key_size, _) in scores {
        result.push(key_size)
    }
    result
}

fn blockwise_transpose(buf: &[u8], key_len: usize) -> Vec<Vec<u8>> {
    // XXX: check key_len against buf maybe
    let mut result: Vec<Vec<u8>> = Vec::new();

    for _ in 0..key_len {
        result.push(Vec::new() as Vec<u8>);
    }
    for chunk in buf.chunks(key_len) {
        for (i, &b) in chunk.iter().enumerate() {
            result[i].push(b);
        }
    }
    result
}

fn guess_repeating_key_xor(buf: &[u8]) -> Vec<u8> {
    let key_sizes = guess_repeating_key_xor_size(&buf);

    let mut key: Vec<u8> = Vec::new();

    for &key_size in &key_sizes[0..1] {
        let transposed = blockwise_transpose(&buf, key_size);

        key.truncate(0);
        for block in &transposed {
            let (key_byte, _) = guess_byte_xor_cipher(&block);
            key.push(key_byte);
        }
        // XXX: just try the first one for now
        break;
    }
    key
}

pub fn hamming_distance_test() {
    let l = "this is a test";
    let r = "wokka wokka!!!";
    let expected: u32 = 37;
    let d = hamming_distance(l.as_bytes(), r.as_bytes());
    if d == expected {
        println!("Finished Hamming distance test");
    } else {
        println!("ERROR Hamming distance: expected {} got {}", expected, d);
    }

    let buf = base64_decode_file("data/1.6.txt");
    let key = guess_repeating_key_xor(&buf);
    let decrypted_bytes = repeating_key_xor(&buf, &key);
    let decrypted = str::from_utf8(&decrypted_bytes).unwrap();
    println!("Finished repeating key xor test:\n{}", decrypted);
}
