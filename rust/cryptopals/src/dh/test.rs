extern crate gmp;

use self::gmp::mpz::Mpz;
use dh::{ff_dhe_public, ff_dhe_shared};

fn ff_dhe_test() {
    let test_vectors = [
        // hex values
        ("e", "7", "5", "25", "15"),
        // https://stackoverflow.com/questions/6032675/diffie-hellman-test-vectors
        (
            "42c6ee70beb7465928a1efe692d2281b8f7b53d6",
            "54081a8fef2127a1f22ed90440b1b09c331d0614",
            "a51883e9ac0539859df3d25c716437008bb4bd8ec4786eb4bc643299daef5e3e5af5863a6ac40a597b83a27583f6a658d408825105b16d31b6ed088fc623f648fd6d95e9cefcb0745763cddf564c87bcf4ba7928e74fd6a3080481f588d535e4c026b58a21e1e5ec412ff241b436043e29173f1dc6cb943c09742de989547288",
            "da3a8085d372437805de95b88b675122f575df976610c6a844de99f1df82a06848bf7a42f18895c97402e81118e01a00d0855d51922f434c022350861d58ddf60d65bc6941fc6064b147071a4c30426d82fc90d888f94990267c64beef8c304a4b2b26fb93724d6a9472fa16bc50c5b9b8b59afb62cfe9ea3ba042c73a6ade35",
            "8d8f4175e16e15a42eb9099b11528af88741cc206a088971d3064bb291eda608d1600bff829624db258fd15e95d96d3e74c6be3232afe5c855b9c59681ce13b7aea9ff2b16707e4c02f0e82bf6dadf2149ac62630f6c62dea0e505e3279404da5ffd5a088e8474ae0c8726b8189cb3d2f04baffe700be849df9f91567fc2ebb8"
        ),
    ];

    for &(priv_a_str, priv_b_str, g_str, p_str, shared_str) in &test_vectors {
        let priv_a = Mpz::from_str_radix(priv_a_str, 16).unwrap();
        let priv_b = Mpz::from_str_radix(priv_b_str, 16).unwrap();
        let g = Mpz::from_str_radix(g_str, 16).unwrap();
        let p = Mpz::from_str_radix(p_str, 16).unwrap();

        let pub_a = ff_dhe_public(&priv_a, &g, &p);
        let pub_b = ff_dhe_public(&priv_b, &g, &p);

        let key_a = ff_dhe_shared(&priv_a, &pub_b, &p);
        let key_b = ff_dhe_shared(&priv_b, &pub_a, &p);
        let expected = Mpz::from_str_radix(shared_str, 16).unwrap();
        assert!((key_a == key_b) && (key_a == expected),
                "ff_dhe_test failed for {:?} {:?}
                 a: {:?}
                 b: {:?}
                 expected: {:?}",
                priv_a, priv_b, key_a, key_b, expected);
    }
}

pub fn dh_test() {
    let n = Mpz::new();
    println!("rust-gmp big zero {:?}", n);

    ff_dhe_test();
}
