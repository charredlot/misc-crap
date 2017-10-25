extern crate gmp;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use self::gmp::mpz::Mpz;
use aes::AESCipher;
use aes::cbc::AESCipherCBC;
use dh::{ff_dhe_public, ff_dhe_shared, ff_dhe_shared_aes_key,
         ff_dhe_aes_key_adjust};

type DHEPeer = fn (Sender<SimMsg>, Receiver<SimMsg>);
type DHEMitm = fn (Sender<SimMsg>, Receiver<SimMsg>,
                   Sender<SimMsg>, Receiver<SimMsg>,
                   mitm_type: MITMType);

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MITMType{
    Passthrough,
    ReplacePubs,
    ReplaceGWith1,
    ReplaceGWithP,
    ReplaceGWithPMinus1,
}

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

fn dhe_sim_send(tx: &Sender<SimMsg>, key: &[u8], plaintext: &[u8]) {
    let (cipher, iv) = AESCipherCBC::new_rand_iv(key);
    let ciphertext = cipher.pad_and_encrypt(plaintext);
    tx.send(SimMsg::Encrypted(ciphertext)).unwrap();
    tx.send(SimMsg::Plain(iv.clone())).unwrap();
}

fn dhe_sim_recv(rx: &Receiver<SimMsg>, key: &[u8]) -> Vec<u8> {
    let ciphertext = rx.recv().unwrap().expect_encrypted();
    let iv = rx.recv().unwrap().expect_plain();

    let cipher = AESCipherCBC::new(&key, &iv);
    cipher.decrypt_and_unpad(&ciphertext)
}

fn dhe_sim_a(tx: Sender<SimMsg>, rx: Receiver<SimMsg>) {
    // XXX: should probably randomize these or something
    let priv_a = Mpz::from_str_radix("53", 16).unwrap();
    let g = Mpz::from_str_radix("2", 16).unwrap();
    let p = Mpz::from_str_radix("71", 16).unwrap();

    // 1. send DHE params including public
    let pub_a = ff_dhe_public(&priv_a, &g, &p);
    tx.send(SimMsg::Exchange(g.clone())).unwrap();
    tx.send(SimMsg::Exchange(p.clone())).unwrap();
    tx.send(SimMsg::Exchange(pub_a.clone())).unwrap();

    // 2. get B's public and generate shared key
    let pub_b = rx.recv().unwrap().expect_exchange();

    let key = ff_dhe_shared_aes_key(&priv_a, &pub_b, &p);

    // 3. encrypt and send message
    dhe_sim_send(&tx, &key, DHE_SIM_MSG.as_bytes());

    // 4. receive echo'd message
    let plaintext = dhe_sim_recv(&rx, &key);
    assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
}

fn dhe_sim_b(tx: Sender<SimMsg>, rx: Receiver<SimMsg>) {
    // 1. recv DHE params
    let g = rx.recv().unwrap().expect_exchange();
    let p = rx.recv().unwrap().expect_exchange();
    let pub_a = rx.recv().unwrap().expect_exchange();

    // 2. generate private and public param and send public
    // XXX: should make a rand byte thing
    let priv_b = Mpz::from_str_radix("32", 16).unwrap();
    assert!(priv_b < p);

    let pub_b = ff_dhe_public(&priv_b, &g, &p);
    tx.send(SimMsg::Exchange(pub_b)).unwrap();

    // 3. derive shared key
    let key = ff_dhe_shared_aes_key(&priv_b, &pub_a, &p);

    // 4. receive and decrypt message
    let plaintext = dhe_sim_recv(&rx, &key);
    assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());

    // 5. encrypt and send back message
    dhe_sim_send(&tx, &key, &plaintext);
}

fn mitm_msg(tx: Sender<SimMsg>, rx: Receiver<SimMsg>, p: &Mpz,
            mitm_type: MITMType) {
    let ciphertext = rx.recv().unwrap().expect_encrypted();
    let iv = rx.recv().unwrap().expect_plain();

    match mitm_type {
        MITMType::ReplacePubs => {
            // if we swap p for B then we get (p ^ a) % p which is always 0
            let cipher = AESCipherCBC::new(&[0u8; 16], &iv);
            let plaintext = cipher.decrypt_and_unpad(&ciphertext);
            assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
        },
        MITMType::ReplaceGWith1 => {
            // if g == 1, A == 1 and B == 1 => key == 1
            let key = ff_dhe_aes_key_adjust(&Mpz::one());
            let cipher = AESCipherCBC::new(&key, &iv);
            let plaintext = cipher.decrypt_and_unpad(&ciphertext);
            assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
        },
        MITMType::ReplaceGWithP => {
            // if g == p, then p ^ anything === 0 mod p
            let cipher = AESCipherCBC::new(&[0u8; 16], &iv);
            let plaintext = cipher.decrypt_and_unpad(&ciphertext);
            assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
        },
        MITMType::ReplaceGWithPMinus1 => {
            // if g == p - 1, (p - 1) ^ x mod p is a binomial, e.g.
            // (p - 1)^2 = p^2 - 2p + 1
            // (p - 1)^3 = p^3 - 3p^2 + 3p - 1
            // (p - 1)^4 = p^4 - 4p^3 + 6p^2 - 4p + 1
            // so with a hand-wavy argument, when the exponent is even,
            // we can subtract p's away until there's 1 leftover.
            // when the exponent is odd, we'll have p - 1 leftover.
            // so from A and B we can know if a and b are even or odd.
            // if a and b have different parity => a * b is even
            // a and b are even => a * b is even
            // a and b are odd => a * b is odd
            // XXX: too lazy to do this, just brute force
            let keys = [
                ff_dhe_aes_key_adjust(&Mpz::one()),
                ff_dhe_aes_key_adjust(p),
            ];
            let mut matched = false;
            for key in &keys {
                let cipher = AESCipherCBC::new(key, &iv);
                let plaintext = cipher.decrypt_and_unpad(&ciphertext);
                if &plaintext as &[u8] == DHE_SIM_MSG.as_bytes() {
                    matched = true;
                    break;
                }
            }
            assert!(matched);
        },
        _ => {},
    };

    tx.send(SimMsg::Encrypted(ciphertext)).unwrap();
    tx.send(SimMsg::Plain(iv)).unwrap();
}

fn dhe_sim_mitm(a_tx: Sender<SimMsg>, a_rx: Receiver<SimMsg>,
                b_tx: Sender<SimMsg>, b_rx: Receiver<SimMsg>,
                mitm_type: MITMType) {
    let g = a_rx.recv().unwrap().expect_exchange();
    let p = a_rx.recv().unwrap().expect_exchange();
    let pub_a = a_rx.recv().unwrap().expect_exchange();

    // pass initial params to b
    b_tx.send(SimMsg::Exchange(g.clone())).unwrap();
    b_tx.send(SimMsg::Exchange(p.clone())).unwrap();
    if mitm_type == MITMType::ReplacePubs {
        b_tx.send(SimMsg::Exchange(p.clone())).unwrap();
    } else {
        b_tx.send(SimMsg::Exchange(pub_a.clone())).unwrap();
    }

    // pass pub_b to a
    let pub_b = b_rx.recv().unwrap().expect_exchange();
    if mitm_type == MITMType::ReplacePubs {
        a_tx.send(SimMsg::Exchange(p.clone())).unwrap();
    } else {
        a_tx.send(SimMsg::Exchange(pub_b.clone())).unwrap();
    }

    // from a to b
    mitm_msg(b_tx, a_rx, &p, mitm_type);
    // from b to a
    mitm_msg(a_tx, b_rx, &p, mitm_type);
}

fn dhe_negotiate_a(tx: Sender<SimMsg>, rx: Receiver<SimMsg>) {
    // XXX: should probably randomize these or something
    let send_g = Mpz::from_str_radix("2", 16).unwrap();
    let send_p = Mpz::from_str_radix("71", 16).unwrap();

    // send our params, but let b decide what the params will be
    // so that we can mitm lol
    tx.send(SimMsg::Exchange(send_g)).unwrap();
    tx.send(SimMsg::Exchange(send_p)).unwrap();
    let g = rx.recv().unwrap().expect_exchange();
    let p = rx.recv().unwrap().expect_exchange();

    let priv_a = Mpz::from_str_radix("53", 16).unwrap();
    assert!(priv_a < p);

    tx.send(SimMsg::Exchange(ff_dhe_public(&priv_a, &g, &p))).unwrap();
    let pub_b = rx.recv().unwrap().expect_exchange();
    let key = ff_dhe_shared_aes_key(&priv_a, &pub_b, &p);

    dhe_sim_send(&tx, &key, DHE_SIM_MSG.as_bytes());

    let plaintext = dhe_sim_recv(&rx, &key);
    assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());
}

fn dhe_negotiate_b(tx: Sender<SimMsg>, rx: Receiver<SimMsg>) {
    let g = rx.recv().unwrap().expect_exchange();
    let p = rx.recv().unwrap().expect_exchange();

    // not really a negotiation since b just echoes a but whatevs
    tx.send(SimMsg::Exchange(g.clone())).unwrap();
    tx.send(SimMsg::Exchange(p.clone())).unwrap();

    let priv_b = Mpz::from_str_radix("32", 16).unwrap();
    assert!(priv_b < p);

    let pub_a = rx.recv().unwrap().expect_exchange();
    tx.send(SimMsg::Exchange(ff_dhe_public(&priv_b, &g, &p))).unwrap();
    let key = ff_dhe_shared_aes_key(&priv_b, &pub_a, &p);

    let plaintext = dhe_sim_recv(&rx, &key);
    assert!(&plaintext as &[u8] == DHE_SIM_MSG.as_bytes());

    dhe_sim_send(&tx, &key, &plaintext);
}

fn dhe_negotiate_mitm(a_tx: Sender<SimMsg>, a_rx: Receiver<SimMsg>,
                      b_tx: Sender<SimMsg>, b_rx: Receiver<SimMsg>,
                      mitm_type: MITMType) {
    let rcv_g = a_rx.recv().unwrap().expect_exchange();
    let p = a_rx.recv().unwrap().expect_exchange();

    let g = match mitm_type {
        MITMType::ReplaceGWith1 => Mpz::one(),
        MITMType::ReplaceGWithPMinus1 => p.clone() - Mpz::one(),
        MITMType::ReplaceGWithP => p.clone(),
        _ => rcv_g.clone(),
    };

    b_tx.send(SimMsg::Exchange(g.clone())).unwrap();
    b_tx.send(SimMsg::Exchange(p.clone())).unwrap();

    // ignore whatever params b sends
    b_rx.recv().unwrap().expect_exchange();
    b_rx.recv().unwrap().expect_exchange();
    a_tx.send(SimMsg::Exchange(g.clone())).unwrap();
    a_tx.send(SimMsg::Exchange(p.clone())).unwrap();

    let pub_a = a_rx.recv().unwrap().expect_exchange();
    b_tx.send(SimMsg::Exchange(pub_a.clone())).unwrap();

    let pub_b = b_rx.recv().unwrap().expect_exchange();
    a_tx.send(SimMsg::Exchange(pub_b.clone())).unwrap();

    mitm_msg(b_tx, a_rx, &p, mitm_type);
    mitm_msg(a_tx, b_rx, &p, mitm_type);
}

const DHE_SIM_MSG: &'static str = "beep boop meow";
fn dhe_mitm_test(func_a: DHEPeer, func_b: DHEPeer, func_m: DHEMitm,
                 mitm_type: MITMType) {
    let (a_tx, ma_rx) = channel();
    let (ma_tx, a_rx) = channel();
    let (b_tx, mb_rx) = channel();
    let (mb_tx, b_rx) = channel();
    let thread_a = thread::spawn(move || {
            func_a(a_tx, a_rx);
        }
    );
    let thread_b = thread::spawn(move || {
            func_b(b_tx, b_rx);
        }
    );
    let thread_m = thread::spawn(move || {
            func_m(ma_tx, ma_rx, mb_tx, mb_rx, mitm_type);
        }
    );

    thread_a.join().unwrap();
    thread_b.join().unwrap();
    thread_m.join().unwrap();
}

pub fn dh_test() {
    let n = Mpz::new();
    println!("rust-gmp big zero {:?}", n);

    ff_dhe_test();

    // XXX: figure out closures into threads one day...maybe when
    // FnBox or Box<FnOnce> works?
    dhe_mitm_test(dhe_sim_a, dhe_sim_b, dhe_sim_mitm, MITMType::Passthrough);
    dhe_mitm_test(dhe_sim_a, dhe_sim_b, dhe_sim_mitm, MITMType::ReplacePubs);
    dhe_mitm_test(dhe_negotiate_a, dhe_negotiate_b, dhe_negotiate_mitm,
                  MITMType::Passthrough);
    dhe_mitm_test(dhe_negotiate_a, dhe_negotiate_b, dhe_negotiate_mitm,
                  MITMType::ReplaceGWith1);
    dhe_mitm_test(dhe_negotiate_a, dhe_negotiate_b, dhe_negotiate_mitm,
                  MITMType::ReplaceGWithP);
    dhe_mitm_test(dhe_negotiate_a, dhe_negotiate_b, dhe_negotiate_mitm,
                  MITMType::ReplaceGWithPMinus1);
}
