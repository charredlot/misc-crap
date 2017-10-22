use mac::{sha1_cat_mac, sha1_cat_mac_digest, sha1_pad, sha1_pad_extend,
          sha1_cat_mac_verify};
use sha1::{Sha1, Digest};
use util::rand_bytes_range;

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

pub fn mac_test() {
    println!("sha1_cat_mac {:?}", &sha1_cat_mac("abcd".as_bytes(),
                                                "efgh".as_bytes()));
    let padded = sha1_pad("beep boop".as_bytes());
    println!("sha1_pad {:?} {}", padded, padded.len());

    sha1_fixate_test();
    sha1_cat_mac_length_ext_test();
    println!("Finished MAC tests");
}
