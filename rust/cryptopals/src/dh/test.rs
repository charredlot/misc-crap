extern crate gmp;

use self::gmp::mpz::Mpz;

pub fn dh_test() {
    let n = Mpz::new();
    println!("rust-gmp big zero {:?}", n);
}
