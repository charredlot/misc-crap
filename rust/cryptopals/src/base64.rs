use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::vec::Vec;
use hex::bytes_to_hex;

const BASE64_VAL_CHAR: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '+', '/'
];

struct Base64Test {
    bytes: &'static [u8],
    encoded: &'static str,
}

const BASE64_TESTS: &[Base64Test] = &[
    Base64Test {
        bytes: &[18],
        encoded: "Eg==",
    },
    Base64Test {
        bytes: &[175, 53],
        encoded: "rzU=",
    },
    Base64Test {
        bytes: &[251, 10, 224],
        encoded: "+wrg",
    },
    Base64Test {
        bytes: &[155, 218, 164, 88],
        encoded: "m9qkWA==",
    },
    Base64Test {
        bytes: &[73, 10, 149, 19, 64],
        encoded: "SQqVE0A=",
    },
    Base64Test {
        bytes: &[135, 139, 134, 95, 187, 71],
        encoded: "h4uGX7tH",
    },
    Base64Test {
        bytes: &[29, 119, 154, 13, 59, 255, 210],
        encoded: "HXeaDTv/0g==",
    },
];

pub fn base64_encode(buf : &[u8]) -> String {
    let mut s = String::new();
    for chunk in buf.chunks(3) {
        let mut triple = [0u8; 3];

        for i in 0..chunk.len() {
            triple[i] = chunk[i];
        }

        let v = [
            (triple[0] >> 2) & 0x3fu8,
            ((triple[0] & 0x3u8) << 4) + ((triple[1] >> 4) & 0xfu8),
            ((triple[1] & 0xfu8) << 2) + ((triple[2] >> 6) & 0x3u8),
            triple[2] & 0x3fu8,
        ];

        s.push(BASE64_VAL_CHAR[v[0] as usize]);
        s.push(BASE64_VAL_CHAR[v[1] as usize]);
        if chunk.len() == 1 {
            s.push('=');
            s.push('=');
        } else if chunk.len() == 2 {
            s.push(BASE64_VAL_CHAR[v[2] as usize]);
            s.push('=');
        } else {
            s.push(BASE64_VAL_CHAR[v[2] as usize]);
            s.push(BASE64_VAL_CHAR[v[3] as usize]);
        }
    }
    s
}

fn base64_char_to_byte(c: u8) -> u8 {
    const UPPER_A: u8 = 'A' as u8;
    const UPPER_Z: u8 = 'Z' as u8;
    const LOWER_A: u8 = 'a' as u8;
    const LOWER_Z: u8 = 'z' as u8;
    const DIGIT_0: u8 = '0' as u8;
    const DIGIT_9: u8 = '9' as u8;
    const PLUS: u8 = '+' as u8;
    const SLASH: u8 = '/' as u8;
    let res: u8 = match c {
        UPPER_A...UPPER_Z => c - UPPER_A,
        LOWER_A...LOWER_Z => (c - LOWER_A) + 26,
        DIGIT_0...DIGIT_9 => (c - DIGIT_0) + 52,
        PLUS => 62,
        SLASH => 63,
        // XXX: should error handle better
        _ => {
            panic!("got unexpected base64 byte {}", c as char);
        }
    };
    res
}

pub fn base64_decode(s: &str) -> Vec<u8> {
    assert!(s.len() % 4 == 0);
    const EQUALS: u8 = '=' as u8;

    let mut vec: Vec<u8> = Vec::new();
    for chunk in s.as_bytes().chunks(4) {
        let b0 = base64_char_to_byte(chunk[0]);
        let b1 = base64_char_to_byte(chunk[1]);
        vec.push((b0 << 2) | ((b1 >> 4) & 0b11));

        if chunk[2] != EQUALS {
            let b2 = base64_char_to_byte(chunk[2]);
            vec.push(((b1 & 0b1111) << 4) | ((b2 >> 2) & 0b1111));

            // XXX: error if 4th is EQUALS but 3rd is not
            if chunk[3] != EQUALS {
                let b3 = base64_char_to_byte(chunk[3]);
                vec.push(((b2 & 0b11) << 6) | b3);
            }
        }
    }
    vec
}

pub fn base64_decode_file(filename: &str) -> Vec<u8> {
    let f = match File::open(filename) {
        Ok(file) => file,
        Err(e) => { panic!("{}", e); },
    };

    let mut contents = String::new();
    let mut buffered = BufReader::new(&f);
    match buffered.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => { panic!("{}", e); },
    };

    base64_decode(contents.replace("\n", "").trim())
}

pub fn base64_test() {
    println!("Running {} base64 tests", BASE64_TESTS.len());
    for t in BASE64_TESTS {
        let s: String = base64_encode(t.bytes);
            // XXX: couldn't get match without a match guard
        if s != t.encoded {
            println!("ERROR base64 encoding {}", bytes_to_hex(t.bytes));
            println!("  expected {}", t.encoded);
            println!("  got {}", s);
            panic!("ERROR base64 encoding");
        }

        let decoded = base64_decode(t.encoded);
        if decoded != t.bytes {
            println!("ERROR base64 decoding {}", t.encoded);
            println!("  expected {:?}", t.bytes);
            println!("  got {:?}", decoded);
            panic!("ERROR base64 decoding");
        }
    }
    println!("Finished base64 tests");
}
