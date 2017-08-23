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

pub fn base64_test() {
    println!("Running {} base64 tests", BASE64_TESTS.len());
    for t in BASE64_TESTS {
        let s: String = base64_encode(t.bytes);
            // XXX: couldn't get match without a match guard
        if s == t.encoded {
            continue;
        } else {
            println!("base64 encoding {}", bytes_to_hex(t.bytes));
            println!("  expected {}", t.encoded);
            println!("  got {}", s);
            break;
        }
    }
    println!("Finished base64 tests");
}


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
