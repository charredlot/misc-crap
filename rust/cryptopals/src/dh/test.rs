extern crate gmp;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use self::gmp::mpz::Mpz;
use aes::AESCipher;
use aes::cbc::AESCipherCBC;
use dh::{ff_dhe_public, ff_dhe_shared, ff_dhe_shared_aes_key};

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

#[derive(Debug)]
enum SimMsg {
    Exchange(Mpz),
    Encrypted(Vec<u8>),
    Plain(Vec<u8>),
}

impl SimMsg {
    fn expect_exchange(self) -> Mpz {
        match self {
            SimMsg::Exchange(n) => n,
            _ => panic!("SimMsg expected exchange got {:?}", self),
        }
    }

    fn expect_encrypted(self) -> Vec<u8> {
        match self {
            SimMsg::Encrypted(msg) => msg,
            _ => panic!("SimMsg expected encrypted got {:?}", self),
        }
    }

    // could macro this ?
    fn expect_plain(self) -> Vec<u8> {
        match self {
            SimMsg::Plain(msg) => msg,
            _ => panic!("SimMsg expected plain got {:?}", self),
        }
    }
}

fn dhe_sim_send(tx: &Sender<SimMsg>, key: &[u8], plaintext: &[u8]) {
    let (cipher, iv) = AESCipherCBC::new_rand_iv(key);
    let ciphertext = cipher.pad_and_encrypt(plaintext);
    tx.send(SimMsg::Encrypted(ciphertext)).unwrap();
    tx.send(SimMsg::Plain(iv.clone())).unwrap();
}

fn dhe_sim_recv(rx: &Receiver<SimMsg>, key: &[u8], mitm: bool) -> Vec<u8> {
    let ciphertext = rx.recv().unwrap().expect_encrypted();
    let iv = rx.recv().unwrap().expect_plain();

    if mitm {
        // if we swap p for B then we get (p ^ a) % p which is always 0
        let cipher = AESCipherCBC::new(&[0u8; 16], &iv);
        let plaintext = cipher.decrypt_and_unpad(&ciphertext);
        assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
    }

    let cipher = AESCipherCBC::new(&key, &iv);
    cipher.decrypt_and_unpad(&ciphertext)
}

fn dhe_sim_a(tx: Sender<SimMsg>, rx: Receiver<SimMsg>, mitm: bool) {
    // XXX: should probably randomize these or something
    let priv_a = Mpz::from_str_radix("53", 16).unwrap();
    let g = Mpz::from_str_radix("2", 16).unwrap();
    let p = Mpz::from_str_radix("71", 16).unwrap();

    // 1. send DHE params including public
    let pub_a = ff_dhe_public(&priv_a, &g, &p);
    tx.send(SimMsg::Exchange(g.clone())).unwrap();
    tx.send(SimMsg::Exchange(p.clone())).unwrap();
    if mitm {
        tx.send(SimMsg::Exchange(p.clone())).unwrap();
    } else {
        tx.send(SimMsg::Exchange(pub_a)).unwrap();
    }

    // 2. get B's public and generate shared key
    let pub_b = rx.recv().unwrap().expect_exchange();

    let key = ff_dhe_shared_aes_key(&priv_a, &pub_b, &p);

    // 3. encrypt and send message
    dhe_sim_send(&tx, &key, DHE_SIM_MSG.as_bytes());

    // 4. receive echo'd message
    let plaintext = dhe_sim_recv(&rx, &key, mitm);
    assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
}

fn dhe_sim_b(tx: Sender<SimMsg>, rx: Receiver<SimMsg>, mitm: bool) {
    // 1. recv DHE params
    let g = rx.recv().unwrap().expect_exchange();
    let p = rx.recv().unwrap().expect_exchange();
    let pub_a = rx.recv().unwrap().expect_exchange();

    // 2. generate private and public param and send public
    // XXX: should make a rand byte thing
    let priv_b = Mpz::from_str_radix("32", 16).unwrap();
    assert!(priv_b < p);

    let pub_b = ff_dhe_public(&priv_b, &g, &p);
    if mitm {
        tx.send(SimMsg::Exchange(p.clone())).unwrap();
    } else {
        tx.send(SimMsg::Exchange(pub_b)).unwrap();
    }

    // 3. derive shared key
    let key = ff_dhe_shared_aes_key(&priv_b, &pub_a, &p);

    // 4. receive and decrypt message
    let plaintext = dhe_sim_recv(&rx, &key, mitm);
    assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());

    // 5. encrypt and send back message
    dhe_sim_send(&tx, &key, &plaintext);
}

const DHE_SIM_MSG: &'static str = "beep boop meow";
fn dhe_mitm_test(mitm: bool) {
    let (a_tx, b_rx) = channel();
    let (b_tx, a_rx) = channel();
    let thread_a = thread::spawn(move || {
            dhe_sim_a(a_tx, a_rx, mitm);
        }
    );
    let thread_b = thread::spawn(move || {
            dhe_sim_b(b_tx, b_rx, mitm);
        }
    );

    thread_a.join().unwrap();
    thread_b.join().unwrap();
}

pub fn dh_test() {
    let n = Mpz::new();
    println!("rust-gmp big zero {:?}", n);

    ff_dhe_test();
    dhe_mitm_test(true);
    dhe_mitm_test(false);
}
