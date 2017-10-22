extern crate cryptopals;

use cryptopals::aes::aes_test;
use cryptopals::base64::base64_test;
use cryptopals::bytes::hamming_distance_test;
use cryptopals::dh::test::dh_test;
use cryptopals::mac_test::mac_test;
use cryptopals::mt19937_test::mt19937_test;
use cryptopals::pkcs7::pkcs7_test;
use cryptopals::url::url_test;
use cryptopals::xor::xor_test;

fn main() {
    dh_test();
    base64_test();
    xor_test();
    hamming_distance_test();
    mt19937_test();
    pkcs7_test();
    url_test();
    mac_test(false);
    aes_test(false);
}
