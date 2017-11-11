extern crate gmp;

use self::gmp::mpz::Mpz;
use util::{rand_bytes, bytes_to_mpz, mpz_bytes, mpz_print_padded};

use rsa::{new_keypair, PublicKey, PrivateKey, pkcs1v15_sha1_der_encode};

fn rsa_keypair_test(bit_len: usize) {
    let (pub_key, priv_key) = new_keypair(bit_len);

    // NB: RSA can only encrypt things smaller than n
    let plaintext = rand_bytes((bit_len / 8) / 2);

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

fn unpadded_msg_test() {
    let plaintext = rand_bytes(32);
    println!("rsa unpadded_msg_test plaintext {:?}", &plaintext);

    let (pub_key, priv_key) = new_keypair(1024);
    let ciphernum = pub_key.encrypt_to_mpz(&plaintext);

    let s = Mpz::one() + Mpz::one();
    let cprime = {
        (s.powm(&pub_key.e, &pub_key.n) * &ciphernum).modulus(&pub_key.n)
    };

    // should be a server doing this here but lazymode
    let pprime = priv_key.decrypt_mpz(&cprime);

    // c' = (s ^ e) * c mod n
    // p' = c'^d = (s^e * c)^d = s^ed * c^d mod n = s * c ^d mod n
    // c ^ d mod n is just the plain text, so divide by s to get it
    let plainnum = {
        &(&pprime * s.invert(&pub_key.n).unwrap()).modulus(&pub_key.n)
    };
    let recovered = mpz_bytes(&plainnum);
    assert_eq!(recovered, plaintext, "rsa unpadded_msg_test failed");
}

fn bsearch_cube_root(floor_start: &Mpz, ceil_start: &Mpz,
                     target: &Mpz) -> Option<Mpz> {
    let two = Mpz::one() + Mpz::one();
    let mut floor = floor_start.clone();
    let mut ceil = ceil_start.clone();
    while floor < ceil {
        let guess = &floor + ((&ceil - &floor) / &two);
        let root = guess.root(3);
        let result = root.pow(3);
        if result == guess {
            return Some(root);
        }

        if &result < target {
            floor = guess;
        } else {
            ceil = guess;
        }

        if &floor + Mpz::one() == ceil {
            // breaks the tie when guess isn't moving any more
            break;
        }
    }

    return None;
}

fn pkcs1v15_cube_root(der: &Mpz, der_len: usize) -> Option<Mpz> {
    // pkcs1v15 starts with 00 01 ff ... ff 00 ASN.1 hash
    // TODO: min padding is 8 bytes but can't seem to find a cube root
    // with >= 8 bytes
    let mut prefix = vec!(0x01u8);
    loop {
        prefix.push(0xffu8);

        // top byte is zero
        // there's a zero byte after padding
        // 35 is the sha1 len
        let used_len = 1 + prefix.len() + 1 + der_len;
        if used_len * 8 >= 1024 {
            break;
        }

        // shift prefix to 1 byte right of 1024-bit
        let mut num = bytes_to_mpz(&prefix);
        num <<= 1024 - ((1 + prefix.len()) * 8);

        // put der 1 byte to the right of the prefix
        num += der << (1024 - (used_len * 8));

        // technically we can play with the lowest set bit, but any value that
        // is greater than the lowest byte will mess up our sha1 hash. we can
        // only allow garbage on the bits below it
        let ceil = &num + (&Mpz::one() << (1024 - (used_len * 8)));
        match bsearch_cube_root(&num, &ceil, &num) {
            Some(n) => return Some(n),
            None => {},
        }
    }

    panic!("no cube root found");
}

fn pkcs1v15_e3_no_pad_check_test() {
    const BIT_LEN: usize = 1024;
    let (pub_key, priv_key) = new_keypair(BIT_LEN);
    let plaintext = "hi mom";
    let der = &pkcs1v15_sha1_der_encode(plaintext.as_bytes());
    let der_num = bytes_to_mpz(&der);

    let forged_signature = match pkcs1v15_cube_root(&der_num, der.len()) {
        Some(n) => {
            println!("pkcs1v15_e3_no_pad_check_test forged:");
            mpz_print_padded(&n.pow(3), 1024 / 8);
            mpz_bytes(&n)
        },
        None => panic!("pkcs1v15_e3_no_pad_check_test couldn't find root"),
    };

    let signature = priv_key.pkcs1v15_sha1_sign(plaintext.as_bytes());
    assert!(pub_key.pkcs1v15_sha1_bad_verify(plaintext.as_bytes(),
                                             &signature));

    assert!(pub_key.pkcs1v15_sha1_bad_verify(plaintext.as_bytes(),
                                             &forged_signature),
            "pkcs1v15_e3_no_pad_check_test forgery failed {:?}",
            &forged_signature);
}

fn pkcs1v15_test() {
    let (pub_key, priv_key) = new_keypair(512);
    println!("pkcs1v15_test {:?} {:?}", &pub_key, &priv_key);
    let msg = "beep boop meow";
    let signature = priv_key.pkcs1v15_sha1_sign(msg.as_bytes());
    assert!(pub_key.pkcs1v15_sha1_bad_verify(msg.as_bytes(), &signature));
}

pub fn rsa_test() {
    rsa_keypair_test(32);
    rsa_keypair_test(512);
    rsa_keypair_test(2048);
    rsa_e3_broadcast_test();
    unpadded_msg_test();
    pkcs1v15_test();
    pkcs1v15_e3_no_pad_check_test();
}
