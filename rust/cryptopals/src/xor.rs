use std::cmp;
use hex::{bytes_to_hex, hex_to_bytes};

pub fn fixed_xor(buf: &[u8], key: &[u8]) -> Vec<u8> {
    let l = cmp::min(buf.len(), key.len());
    let mut vec: Vec<u8> = Vec::with_capacity(l);
    for (&b, &k) in buf.iter().zip(key) {
        vec.push(b ^ k);
    }
    vec
}

pub fn fixed_xor_test() {
    let buf = hex_to_bytes("1c0111001f010100061a024b53535009181c");
    let key = hex_to_bytes("686974207468652062756c6c277320657965");
    let answer = "746865206b696420646f6e277420706c6179";

    let result = fixed_xor(buf.as_slice(), key.as_slice());

    let s = bytes_to_hex(result.as_slice());
    if s == answer {
        println!("fixed_xor_test finished");
    } else {
        println!("fixed_xor error:");
        println!("  expected {}", answer);
        println!("  got      {}", s);
    }
}
