extern crate gmp;

use self::gmp::mpz::Mpz;
use util::{rand_bytes, mpz_bytes};

use rsa::{new_keypair, PublicKey, PrivateKey};

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

fn rsa_e3_broadcast_test() {
    const BIT_LEN: usize = 1024;
    const NUM_KEYS: usize = 3;

    let mut keypairs: Vec<(PublicKey, PrivateKey)> = Vec::new();
    'find_keypair: loop {
        keypairs.clear();
        for i in 0..NUM_KEYS {
            let (pub_key, priv_key) = new_keypair(BIT_LEN);

            // chinese remainder theorem (CRT) requires that
            // all the modulus' are pairwise relatively prime
            for &(ref other_pub, _) in &keypairs[..i] {
                if pub_key.n.gcd(&other_pub.n) != Mpz::one() {
                    println!("rsa_e3_broadcast_test not rel prime, retrying");
                    continue 'find_keypair;
                }
            }
            keypairs.push((pub_key, priv_key));
        }
        break;
    }

    let plaintext = rand_bytes(32);
    println!("rsa_e3_broadcast_test plaintext {:?}", &plaintext);
    println!("rsa_e3_broadcast_test keypairs {:?}", &keypairs);

    let mut ciphertexts = Vec::new();
    for &(ref pub_key, _) in &keypairs {
        ciphertexts.push(pub_key.encrypt_to_mpz(&plaintext));
    }

    // from wiki, use bezout's identity and extended euclidean algo to
    // solve for n0 and n1, then induct with n0 * n1, n2
    // just implement here per cryptopals, figure out general case later
    let &(ref pub0, _) = &keypairs[0];
    let &(ref pub1, _) = &keypairs[1];
    let &(ref pub2, _) = &keypairs[2];

    let ms0 = &pub1.n * &pub2.n;
    let ms1 = &pub0.n * &pub2.n;
    let ms2 = &pub0.n * &pub1.n;

    let mut result = &ciphertexts[0] * &ms0 * (&ms0).invert(&pub0.n).unwrap();
    result += &ciphertexts[1] * &ms1 * (&ms1).invert(&pub1.n).unwrap();
    result += &ciphertexts[2] * &ms2 * (&ms2).invert(&pub2.n).unwrap();

    // e = 3 means the result should be < n0 * n1 * n2
    // TODO: says not to do the modulus? but breaks without it. revisit
    let n = &pub0.n * &pub1.n * &pub2.n;
    let cube_root = result.modulus(&n).root(3);
    let recovered = mpz_bytes(&cube_root);
    assert_eq!(recovered, plaintext, "rsa_e3_broadcast_test failed");
}

pub fn rsa_test() {
    rsa_keypair_test(32);
    rsa_keypair_test(512);
    rsa_keypair_test(2048);
    rsa_e3_broadcast_test();
}
