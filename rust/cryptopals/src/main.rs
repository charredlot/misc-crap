extern crate cryptopals;

use cryptopals::base64::base64_test;
use cryptopals::xor::fixed_xor_test;

fn main() {
    base64_test();
    fixed_xor_test();
}
