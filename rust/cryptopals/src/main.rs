extern crate cryptopals;

use cryptopals::aes::aes_test;
use cryptopals::base64::base64_test;
use cryptopals::bytes::hamming_distance_test;
use cryptopals::pkcs::pkcs7_test;
use cryptopals::url::url_test;
use cryptopals::xor::xor_test;

fn main() {
    base64_test();
    xor_test();
    hamming_distance_test();
    aes_test();
    pkcs7_test();
    url_test();
}
