extern crate cryptopals;

use cryptopals::base64::base64_test;
use cryptopals::bytes::hamming_distance_test;
use cryptopals::xor::xor_test;

fn main() {
    base64_test();
    xor_test();
    hamming_distance_test();
}
