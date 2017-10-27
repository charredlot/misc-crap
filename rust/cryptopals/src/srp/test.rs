use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

extern crate gmp;
extern crate sha2;

use self::gmp::mpz::Mpz;
use self::sha2::{Sha256, Digest};

use mac::hmac_sha256;
use srp;
use util::{rand_bytes, randomish_mpz_lt, bytes_to_mpz, mpz_bytes};

const TEST_IDENTITY: &'static str = "username";
const TEST_PASSWORD: &'static str = "beepboopmeow";

const SRP_N: &'static str = concat!("00c037c37588b4329887e61c2da332",
                                    "4b1ba4b81a63f9748fed2d8a410c2f",
                                    "c21b1232f0d3bfa024276cfd884481",
                                    "97aae486a63bfca7b8bf7754dfb327",
                                    "c7201f6fd17fd7fd74158bd31ce772",
                                    "c9f5f8ab584548a99a759b5a2c0532",
                                    "162b7b6218e8f142bce2c30d778468",
                                    "9a483e095e701618437913a8c39c3d",
                                    "d0d4ca3c500b885fe3");

#[derive(Debug)]
enum SRPMsg {
    Num(Mpz),
    Bytes(Vec<u8>),
}

impl SRPMsg {
    fn expect_num(self) -> Mpz {
        match self {
            SRPMsg::Num(n) => n,
            _ => panic!("SRPMsg expected num {:?}", self),
        }
    }

    fn expect_bytes(self) -> Vec<u8> {
        match self {
            SRPMsg::Bytes(v) => v,
            _ => panic!("SRPMsg expected bytes {:?}", self),
        }
    }
}

fn srp_client(tx: Sender<SRPMsg>, rx: Receiver<SRPMsg>,
              replace_a: Option<Mpz>) {
    let n = Mpz::from_str_radix(SRP_N, 16).unwrap();
    let g = Mpz::from_str_radix("2", 10).unwrap();
    // TODO: wiki say to sha256 n || g here?
    let k = Mpz::from_str_radix("3", 10).unwrap();

    tx.send(SRPMsg::Num(n.clone())).unwrap();
    tx.send(SRPMsg::Num(g.clone())).unwrap();
    tx.send(SRPMsg::Num(k.clone())).unwrap();

    let priv_a = randomish_mpz_lt(&n);
    println!("SRP client private a {:?}", &priv_a);

    let pub_a = match replace_a {
        Some(ref a) => a.clone(),
        None => g.powm(&priv_a, &n),
    };

    tx.send(SRPMsg::Bytes(TEST_IDENTITY.as_bytes().to_vec())).unwrap();
    tx.send(SRPMsg::Num(pub_a.clone())).unwrap();

    let salt = rx.recv().unwrap().expect_bytes();
    let pub_b = rx.recv().unwrap().expect_num();

    let ab = [mpz_bytes(&pub_a), mpz_bytes(&pub_b)].concat();
    let u = bytes_to_mpz(&Sha256::digest(&ab).to_vec());

    let x = bytes_to_mpz(&srp::salted_hash(&salt, TEST_IDENTITY.as_bytes(),
                                           TEST_PASSWORD.as_bytes()));

    // assume if we replace_a we're using N^i so that s is always 0
    let s = match replace_a {
        Some(_) => Mpz::zero(),
        None => {
           let tmp = pub_b - (k * g.powm(&x, &n));
            tmp.powm(&(priv_a + (u * x)), &n)
        }
    };
    let k = Sha256::digest(&mpz_bytes(&s)).to_vec();

    let hmac = hmac_sha256(&k, &salt);
    tx.send(SRPMsg::Bytes(hmac)).unwrap();
}

fn srp_server(tx: Sender<SRPMsg>, rx: Receiver<SRPMsg>) {
    let n = rx.recv().unwrap().expect_num();
    let g = rx.recv().unwrap().expect_num();
    let k = rx.recv().unwrap().expect_num();

    let identity = rx.recv().unwrap().expect_bytes();
    let pub_a = rx.recv().unwrap().expect_num();

    let salt = rand_bytes(16);
    println!("SRP server salt {:?}", salt);
    let x = bytes_to_mpz(&srp::salted_hash(&salt, &identity,
                                           TEST_PASSWORD.as_bytes()));
    let v = g.powm(&x, &n); 
    let priv_b = randomish_mpz_lt(&n);
    println!("SRP server private b {:?}", &priv_b);
    let pub_b = ((&k * &v) + g.powm(&priv_b, &n)).modulus(&n);

    tx.send(SRPMsg::Bytes(salt.clone())).unwrap();
    tx.send(SRPMsg::Num(pub_b.clone())).unwrap();

    let ab = [mpz_bytes(&pub_a), mpz_bytes(&pub_b)].concat();
    let u = bytes_to_mpz(&Sha256::digest(&ab).to_vec());

    let s = (pub_a * v.powm(&u, &n)).powm(&priv_b, &n);
    let k = Sha256::digest(&mpz_bytes(&s)).to_vec();

    let hmac = hmac_sha256(&k, &salt);
    let client_hmac = rx.recv().unwrap().expect_bytes();

    assert_eq!(hmac, client_hmac, "SRP hmac compare failed");
}

fn srp_exchange_test(client: fn (Sender<SRPMsg>, Receiver<SRPMsg>,
                                 Option<Mpz>),
                     server: fn (Sender<SRPMsg>, Receiver<SRPMsg>),
                     replace_a: Option<Mpz>) {
    let (c_tx, s_rx) = channel();
    let (s_tx, c_rx) = channel();

    let client_thread = thread::spawn(move || {
        client(c_tx, c_rx, replace_a)
    });
    let server_thread = thread::spawn(move || { server(s_tx, s_rx) });

    client_thread.join().unwrap();
    server_thread.join().unwrap();
}

pub fn srp_test() {
    println!("Starting SRP tests");
    srp_exchange_test(srp_client, srp_server, None);
    srp_exchange_test(srp_client, srp_server, Some(Mpz::zero()));

    let n = Mpz::from_str_radix(SRP_N, 16).unwrap();
    srp_exchange_test(srp_client, srp_server, Some(n.clone()));
    srp_exchange_test(srp_client, srp_server, Some(&n * &n));
    println!("Finished SRP tests");
}
