extern crate cryptopals;

use cryptopals::base64::base64_encode;
use cryptopals::debug::bytes_to_hex;

struct base64Test {
    bytes: &'static [u8],
    encoded: &'static str,
}

const BASE64_TESTS: &[base64Test] = &[
    base64Test {
        bytes: &[178u8],
        encoded: "sg==",
    },
];

fn test_base64() {
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
}

fn main() {
    test_base64();
}
