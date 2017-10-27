use std::thread;
use std::time;
use std::time::Duration;

use hex::hex_to_bytes;
use mac::{sha1_cat_mac, sha1_cat_mac_digest, sha1_pad, sha1_pad_extend,
          sha1_cat_mac_verify, hmac_sha1, hmac_sha256};
use sha1::{Sha1, Digest, DIGEST_LENGTH};
use util::{rand_bytes_range, assert_slice_cmp};

fn sha1_fixate_test() {
    let msgs = [
        "beep boop",
        "meow",
    ];
    let mut padded = sha1_pad(msgs[0].as_bytes());
    let save_len = padded.len();
    padded.extend_from_slice(msgs[1].as_bytes());
    let expected = {
        let mut tmp = Sha1::new();
        tmp.update(&padded);
        tmp.digest()
    };

    let mut mod_mac = {
        let mut tmp = Sha1::new();
        tmp.update(msgs[0].as_bytes());
        Sha1::new_with_digest(&tmp.digest())
    };
    mod_mac.update(&sha1_pad(&padded)[save_len..]);
    assert!(expected.data.state == mod_mac.state.state,
            "sha1_fixate_test failed expected {:?} got {:?}",
            expected.data.state, mod_mac.state.state);
}

fn sha1_cat_mac_length_ext_test() {
    let orig = "comment1=cooking%20MCs;userdata=foo;comment2=%20like%20a%20pound%20of%20bacon";
    let suffix = ";admin=true";

    let key = rand_bytes_range(1, 64);
    println!("sha1_cat_mac length extension test with key {:?} {}", key,
             key.len());

    let mod_mac = {
        let orig_digest = sha1_cat_mac_digest(&key, orig.as_bytes());
        Sha1::new_with_digest(&orig_digest)
    };

    // brute force key lengths
    for key_len in 1..64 {
        // start with the key_len guess bytes + orig_len bytes to get padding
        let mut msg = vec!(0u8; key_len);
        msg.extend_from_slice(orig.as_bytes());
        sha1_pad_extend(&mut msg);
        let orig_padded_len_guess = msg.len();

        // add suffix and pad the entire msg that we think will be hashed
        msg.extend_from_slice(suffix.as_bytes());
        sha1_pad_extend(&mut msg);

        // mod_mac state is after hashing key + orig + sha1 padding
        let mut mac = mod_mac.clone();
        // sha1 update just the suffix and full sha1 padding
        mac.update(&msg[orig_padded_len_guess..]);

        // don't do digest() because it does the length padding again
        let hash = Digest{data: mac.state};

        // careful not to include our extra sha1 padding on the whole msg
        let admin_msg = &msg[key_len..(orig_padded_len_guess +
                                       suffix.as_bytes().len())];
        if sha1_cat_mac_verify(&key, admin_msg, &hash) {
            println!("found correct mac at key_len {}", key_len);
            return;
        }
    }
    panic!("could not find a matching key_len");
}

fn hmac_sha_test() {
    const TEST_VECTORS: [(&'static str, &'static str,
                          &'static str, &'static str); 2] = [
        ("",
         "",
         "fbdb1d1b18aa6c08324b7d64b71fb76370690e1d",
         "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad"),
        ("key",
         "The quick brown fox jumps over the lazy dog",
         "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9",
         "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"),
    ];

    for &(key, msg, sha1_str, sha256_str) in &TEST_VECTORS {
        assert_eq!(&hex_to_bytes(sha1_str),
                   &hmac_sha1(key.as_bytes(), msg.as_bytes()));
        assert_eq!(&hex_to_bytes(sha256_str),
                   &hmac_sha256(key.as_bytes(), msg.as_bytes()));
    }
}

fn insecure_cmp(calc_hmac: &[u8], hmac: &[u8]) -> Result<(), ()> {
    if calc_hmac.len() != hmac.len() {
        return Err(());
    }

    for i in 0..calc_hmac.len() {
        if calc_hmac[i] != hmac[i] {
            return Err(());
        }
        // XXX: a little aggressive, may need to tune esp when using threads
        thread::sleep(time::Duration::from_millis(5));
    }

    return Ok(());
}

fn hmac_sha1_get_best_byte(calc_hmac: Vec<u8>, mut hmac_guess: Vec<u8>,
                           index: usize,
                           start: usize, end: usize) -> (Duration, u8) {
    let mut longest = time::Duration::new(0, 0);
    let mut best_byte = 0u8;

    for j in start..end {
        let b = j as u8;
        hmac_guess[index] = b;

        // TODO: should use something that measures user time + sleep time..
        // maybe getrusage?
        let now = time::Instant::now();
        let _ = insecure_cmp(&calc_hmac, &mut hmac_guess);
        let elapsed = now.elapsed();
        if elapsed > longest {
            longest = elapsed;
            best_byte = b;
        }
    }

    (longest, best_byte)
}

fn hmac_sha1_timing_test() {
    const MSG: &'static str = "beep boop meow";
    let mut hmac_guess = vec!(0u8; DIGEST_LENGTH);

    let key = rand_bytes_range(8, 32);
    println!("hmac_sha1_timing_test with key {:?}", key);

    // too lazy to set up http in rust blah
    // also should recalc hmac every time but save time here
    let calc_hmac = hmac_sha1(&key, MSG.as_bytes());
    for i in 0..hmac_guess.len() {
        let num_threads = 3;
        let mut threads = Vec::with_capacity(num_threads);
        let interval = u8::max_value() as usize / num_threads;
        for j in 0..num_threads {
            let start = j * interval;
            let end = if j == num_threads - 1 {
                u8::max_value() as usize
            } else {
                (j + 1) * interval
            };
            let hmac_guess_copy = hmac_guess.to_vec();
            // bleh need to move copy since can't figure out
            // how long thread will borrow it for
            let calc_hmac_copy = calc_hmac.clone();
            threads.push(
                thread::spawn(
                    move || -> (Duration, u8) {
                        hmac_sha1_get_best_byte(calc_hmac_copy,
                                                hmac_guess_copy,
                                                i,
                                                start, end)
                    }
                )
            );
        }

        let mut longest = time::Duration::new(0, 0);
        let mut best_byte = 0u8;
        for t in threads {
            let (elapsed, b) = t.join().unwrap();
            if elapsed > longest {
                longest = elapsed;
                best_byte = b;
            }
        }
        hmac_guess[i] = best_byte;
    }

    assert_slice_cmp("hmac_sha1_timing_test", &calc_hmac, &hmac_guess);
}

pub fn mac_test(full_test: bool) {
    println!("sha1_cat_mac {:?}", &sha1_cat_mac("abcd".as_bytes(),
                                                "efgh".as_bytes()));
    let padded = sha1_pad("beep boop".as_bytes());
    println!("sha1_pad {:?} {}", padded, padded.len());

    sha1_fixate_test();
    sha1_cat_mac_length_ext_test();
    // XXX: skip md4 because too lazy to port rust versions and don't
    // want to import the whole thing
    hmac_sha_test();
    if full_test {
        hmac_sha1_timing_test();
    }
    println!("Finished MAC tests");
}
