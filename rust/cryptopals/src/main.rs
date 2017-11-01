extern crate cryptopals;

use cryptopals::aes::aes_test;
use cryptopals::base64::base64_test;
use cryptopals::bytes::hamming_distance_test;
use cryptopals::dh::test::dh_test;
use cryptopals::mac_test::mac_test;
use cryptopals::mt19937_test::mt19937_test;
use cryptopals::pkcs7::pkcs7_test;
use cryptopals::rsa::test::rsa_test;
use cryptopals::srp::test::srp_test;
use cryptopals::url::url_test;
use cryptopals::xor::xor_test;

fn main() {
    rsa_test();
    srp_test();
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
