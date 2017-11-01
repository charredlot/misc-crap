use util::rand_bytes;

use rsa::new_keypair;

fn rsa_keypair_test(bit_len: usize) {
    let plaintext = rand_bytes((bit_len / 8) - 1);
    let (pub_key, priv_key) = new_keypair(bit_len);

    println!("RSA keypair test with bit_len {} plaintext {:?}",
             bit_len, plaintext);
    println!(concat!("  e = {}\n",
                     "  n = {}\n",
                     "  d = {}"),
                     pub_key.e, pub_key.n, priv_key.d);

    let ciphertext = pub_key.encrypt(&plaintext);
    let decrypted = priv_key.decrypt(&ciphertext);
    assert_eq!(&decrypted, &plaintext, "rsa_keypair_test failed");
}

pub fn rsa_test() {
    rsa_keypair_test(32);
    rsa_keypair_test(512);
    rsa_keypair_test(2048);
}
