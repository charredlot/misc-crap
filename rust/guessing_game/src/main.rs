extern crate rand;

use std::io;
use std::cmp::Ordering;
use std::num::ParseIntError;
use rand::Rng;

fn get_secret(start: u32, end: u32) -> u32 {
    let secret_number = rand::thread_rng().gen_range(start, end);
    println!("Picked a number between {} and {}", start, end);
    secret_number
}

fn get_input(guess: &mut String) -> Result<u32, ParseIntError> {
    println!("Enter number");
    io::stdin().read_line(guess).expect("failed to read line");
    guess.trim().parse()
}

fn main() {
    let secret_number = get_secret(1, 100);
    let mut guess_str = String::new();

    loop {
        guess_str.clear();
        let guess: u32 = match get_input(&mut guess_str) {
                Ok(num) => num,
                Err(_) => continue,
        };
        println!("Guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("Matched: {}!", secret_number);
                break;
            }
        }
    }
}
