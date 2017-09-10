mod detect;
mod constants;
use self::constants::{SBOX,INV_SBOX,GF256_MUL_2, GF256_MUL_3, GF256_MUL_9,
                      GF256_MUL_11, GF256_MUL_13, GF256_MUL_14};
use self::detect::distinguish_aes_cbc_ecb_test;
use base64::base64_decode_file;
use hex::{hex_to_bytes,bytes_to_hex};
use pkcs::pkcs7_unpad;
use xor::fixed_xor;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str;

struct AESBlock {
    values: [u8; 16],
}

impl AESBlock {
    fn from_slice(values: &[u8]) -> AESBlock {
        assert!(values.len() == 16);
        let mut block = AESBlock {values: [0u8; 16]};
        for (i, &b) in values.iter().enumerate() {
            block.values[i] = b
        }
        block
    }

    fn set(&mut self, row: usize, col: usize, val: u8) {
        self.values[col * 4 + row] = val;
    }

    fn get(&self, row: usize, col: usize) -> u8 {
        self.values[col * 4 + row]
    }

    fn add_round_key(&mut self, key: &[u8]) {
        for (b, k) in self.values.iter_mut().zip(key) {
            *b = *b ^ k;
        }
    }

    fn sub_bytes(&mut self) {
        for b in &mut self.values {
            *b = SBOX[*b as usize];
        }
    }

    fn inv_sub_bytes(&mut self) {
        for b in &mut self.values {
            *b = INV_SBOX[*b as usize];
        }
    }

    fn ror_row(&mut self, row: usize, shift: usize) {
        let mut tmp = [0u8; 4];

        // XXX: arg this sucks, find better pattern
        tmp[0] = self.get(row, (0 + shift) % 4);
        tmp[1] = self.get(row, (1 + shift) % 4);
        tmp[2] = self.get(row, (2 + shift) % 4);
        tmp[3] = self.get(row, (3 + shift) % 4);
        for (i, &b) in tmp.iter().enumerate() {
            self.set(row, i, b);
        }
    }

    fn shift_rows(&mut self) {
        self.ror_row(1, 1);

        let mut tmp = [0u8; 4];
        tmp[0] = self.get(2, 2);
        tmp[1] = self.get(2, 3);
        tmp[2] = self.get(2, 0);
        tmp[3] = self.get(2, 1);
        for i in 0..4 {
            self.set(2, i, tmp[i]);
        }

        self.ror_row(3, 3);
    }

    fn inv_shift_rows(&mut self) {
        self.ror_row(1, 3);

        let mut tmp = [0u8; 4];
        tmp[0] = self.get(2, 2);
        tmp[1] = self.get(2, 3);
        tmp[2] = self.get(2, 0);
        tmp[3] = self.get(2, 1);
        for i in 0..4 {
            self.set(2, i, tmp[i]);
        }

        self.ror_row(3, 1);
    }

    fn mix_column(column: &mut [u8]) {
        let tmp = [
            GF256_MUL_2[column[0] as usize] ^
            GF256_MUL_3[column[1] as usize] ^
            column[2] ^
            column[3],

            column[0] ^
            GF256_MUL_2[column[1] as usize] ^
            GF256_MUL_3[column[2] as usize] ^
            column[3],

            column[0] ^
            column[1] ^
            GF256_MUL_2[column[2] as usize] ^
            GF256_MUL_3[column[3] as usize],

            GF256_MUL_3[column[0] as usize] ^
            column[1] ^
            column[2] ^
            GF256_MUL_2[column[3] as usize],
        ];

        for (b, &nb) in column.iter_mut().zip(&tmp) {
            *b = nb
        }
    }

    fn inv_mix_column(column: &mut [u8]) {
        let tmp = [
            GF256_MUL_14[column[0] as usize] ^
            GF256_MUL_11[column[1] as usize] ^
            GF256_MUL_13[column[2] as usize] ^
            GF256_MUL_9[column[3] as usize],

            GF256_MUL_9[column[0] as usize] ^
            GF256_MUL_14[column[1] as usize] ^
            GF256_MUL_11[column[2] as usize] ^
            GF256_MUL_13[column[3] as usize],

            GF256_MUL_13[column[0] as usize] ^
            GF256_MUL_9[column[1] as usize] ^
            GF256_MUL_14[column[2] as usize] ^
            GF256_MUL_11[column[3] as usize],

            GF256_MUL_11[column[0] as usize] ^
            GF256_MUL_13[column[1] as usize] ^
            GF256_MUL_9[column[2] as usize] ^
            GF256_MUL_14[column[3] as usize],
        ];

        for (b, &nb) in column.iter_mut().zip(&tmp) {
            *b = nb
        }
    }

    fn mix_columns(&mut self) {
        AESBlock::mix_column(&mut self.values[0..4]);
        AESBlock::mix_column(&mut self.values[4..8]);
        AESBlock::mix_column(&mut self.values[8..12]);
        AESBlock::mix_column(&mut self.values[12..16]);
    }

    fn inv_mix_columns(&mut self) {
        AESBlock::inv_mix_column(&mut self.values[0..4]);
        AESBlock::inv_mix_column(&mut self.values[4..8]);
        AESBlock::inv_mix_column(&mut self.values[8..12]);
        AESBlock::inv_mix_column(&mut self.values[12..16]);
    }
}

pub struct AESCipher {
    key_schedule: Vec<Vec<u8>>,
    rounds: usize,
}

impl AESCipher {
    fn new(key: &[u8]) -> AESCipher {
        let key_size = key.len();

        let rounds: usize = match key_size {
            // from AES spec
            16 => 10,  // 128
            24 => 12,  // 192
            32 => 14,  // 256
            // XXX: return error instead of panicking
            _ => panic!("bad key len {}", key_size)
        };

        // XXX: could pack encrypt/decrypt in a closure like go does?
        AESCipher {
            key_schedule: AESCipher::expand_key(key),
            rounds: rounds,
        }
    }

    fn expand_key(key: &[u8]) -> Vec<Vec<u8>> {
        // from the wiki for Rijndael key schedule
        let (n, b) = match key.len() {
            16 => (16, 176),
            24 => (24, 208),
            32 => (32, 240),
            // XXX: return error instead of panicking
            _ => panic!("bad key len {}", key.len())
        };

        let mut expanded: Vec<u8> = Vec::with_capacity(b);

        // put in original key
        for &b in key {
            expanded.push(b);
        }


        let mut rcon_i = 1;
        let mut t = [0u8; 4];
        // XXX: could rust optimize better with local length variable?
        while expanded.len() < b {
            for j in 0..4 {
                t[j] = expanded[expanded.len() + j - 4];
            }

            if expanded.len() % n == 0 {
                rijndael_core(&mut t, rcon_i);
                rcon_i += 1;
            }

            if n == 32 && (expanded.len() % 32 == 16) {
                // 256-bit keys have an extra sbox step for some reason
                for j in 0..4 {
                    t[j] = SBOX[t[j] as usize];
                }
            }

            for j in 0..4 {
                let prev = expanded[expanded.len() - n];
                expanded.push(t[j] ^ prev);
            }
        }

        let mut keys: Vec<Vec<u8>> = Vec::new();
        for chunk in expanded.chunks(16) {
            keys.push(chunk.to_vec());
        }

        keys
    }

    fn encrypt_block(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut state = AESBlock::from_slice(plaintext);

        // initial round
        state.add_round_key(&self.key_schedule[0]);

        for i in 1..self.rounds {
            state.sub_bytes();
            state.shift_rows();
            state.mix_columns();
            state.add_round_key(&self.key_schedule[i]);
        }

        state.sub_bytes();
        state.shift_rows();
        state.add_round_key(&self.key_schedule[self.rounds]);

        state.values.to_vec()
    }

    fn decrypt_block(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut state = AESBlock::from_slice(plaintext);

        // inverse final round
        state.add_round_key(&self.key_schedule[self.rounds]);
        state.inv_shift_rows();
        state.inv_sub_bytes();


        for i in (1..self.rounds).rev() {
            state.add_round_key(&self.key_schedule[i]);
            state.inv_mix_columns();
            state.inv_shift_rows();
            state.inv_sub_bytes();
        }

        // inverse initial round
        state.add_round_key(&self.key_schedule[0]);

        state.values.to_vec()
    }

    fn ecb_encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for chunk in plaintext.chunks(16) {
            let mut block = self.encrypt_block(chunk);
            result.append(&mut block)
        }

        result
    }

    fn ecb_decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for chunk in ciphertext.chunks(16) {
            let mut block = self.decrypt_block(chunk);
            result.append(&mut block)
        }

        result
    }

    fn cbc_encrypt(&self, plaintext: &[u8], init_iv: &[u8]) -> Vec<u8> {
        assert!(init_iv.len() == 16);
        // XXX: a way to do this without creating an extra vec?
        let mut iv = [0u8; 16];
        for (dst, src) in iv.iter_mut().zip(init_iv) {
            *dst = *src;
        }
        let mut result: Vec<u8> = Vec::new();
        for block in plaintext.chunks(16) {
            let mixed = fixed_xor(block, &iv);
            let encrypted = self.encrypt_block(&mixed);
            result.extend(encrypted.iter().cloned());
            for (dst, src) in iv.iter_mut().zip(&encrypted) {
                *dst = *src;
            }
        }
        result
    }

    fn cbc_decrypt(&self, ciphertext: &[u8], init_iv: &[u8]) -> Vec<u8> {
        assert!(init_iv.len() == 16);
        // XXX: a way to do this without creating an extra vec?
        let mut iv = [0u8; 16];
        for (dst, src) in iv.iter_mut().zip(init_iv) {
            *dst = *src;
        }
        let mut result: Vec<u8> = Vec::new();
        for block in ciphertext.chunks(16) {
            result.append(&mut fixed_xor(&self.decrypt_block(block), &iv));
            for (dst, src) in iv.iter_mut().zip(block) {
                *dst = *src;
            }
        }
        result
    }
}

fn rijndael_core(t: &mut [u8; 4], rcon_i: usize) {
    const RCON: [u8; 256] = [
        0x8du8, 0x01u8, 0x02u8, 0x04u8, 0x08u8, 0x10u8, 0x20u8, 0x40u8,
        0x80u8, 0x1bu8, 0x36u8, 0x6cu8, 0xd8u8, 0xabu8, 0x4du8, 0x9au8,
        0x2fu8, 0x5eu8, 0xbcu8, 0x63u8, 0xc6u8, 0x97u8, 0x35u8, 0x6au8,
        0xd4u8, 0xb3u8, 0x7du8, 0xfau8, 0xefu8, 0xc5u8, 0x91u8, 0x39u8,
        0x72u8, 0xe4u8, 0xd3u8, 0xbdu8, 0x61u8, 0xc2u8, 0x9fu8, 0x25u8,
        0x4au8, 0x94u8, 0x33u8, 0x66u8, 0xccu8, 0x83u8, 0x1du8, 0x3au8,
        0x74u8, 0xe8u8, 0xcbu8, 0x8du8, 0x01u8, 0x02u8, 0x04u8, 0x08u8,
        0x10u8, 0x20u8, 0x40u8, 0x80u8, 0x1bu8, 0x36u8, 0x6cu8, 0xd8u8,
        0xabu8, 0x4du8, 0x9au8, 0x2fu8, 0x5eu8, 0xbcu8, 0x63u8, 0xc6u8,
        0x97u8, 0x35u8, 0x6au8, 0xd4u8, 0xb3u8, 0x7du8, 0xfau8, 0xefu8,
        0xc5u8, 0x91u8, 0x39u8, 0x72u8, 0xe4u8, 0xd3u8, 0xbdu8, 0x61u8,
        0xc2u8, 0x9fu8, 0x25u8, 0x4au8, 0x94u8, 0x33u8, 0x66u8, 0xccu8,
        0x83u8, 0x1du8, 0x3au8, 0x74u8, 0xe8u8, 0xcbu8, 0x8du8, 0x01u8,
        0x02u8, 0x04u8, 0x08u8, 0x10u8, 0x20u8, 0x40u8, 0x80u8, 0x1bu8,
        0x36u8, 0x6cu8, 0xd8u8, 0xabu8, 0x4du8, 0x9au8, 0x2fu8, 0x5eu8,
        0xbcu8, 0x63u8, 0xc6u8, 0x97u8, 0x35u8, 0x6au8, 0xd4u8, 0xb3u8,
        0x7du8, 0xfau8, 0xefu8, 0xc5u8, 0x91u8, 0x39u8, 0x72u8, 0xe4u8,
        0xd3u8, 0xbdu8, 0x61u8, 0xc2u8, 0x9fu8, 0x25u8, 0x4au8, 0x94u8,
        0x33u8, 0x66u8, 0xccu8, 0x83u8, 0x1du8, 0x3au8, 0x74u8, 0xe8u8,
        0xcbu8, 0x8du8, 0x01u8, 0x02u8, 0x04u8, 0x08u8, 0x10u8, 0x20u8,
        0x40u8, 0x80u8, 0x1bu8, 0x36u8, 0x6cu8, 0xd8u8, 0xabu8, 0x4du8,
        0x9au8, 0x2fu8, 0x5eu8, 0xbcu8, 0x63u8, 0xc6u8, 0x97u8, 0x35u8,
        0x6au8, 0xd4u8, 0xb3u8, 0x7du8, 0xfau8, 0xefu8, 0xc5u8, 0x91u8,
        0x39u8, 0x72u8, 0xe4u8, 0xd3u8, 0xbdu8, 0x61u8, 0xc2u8, 0x9fu8,
        0x25u8, 0x4au8, 0x94u8, 0x33u8, 0x66u8, 0xccu8, 0x83u8, 0x1du8,
        0x3au8, 0x74u8, 0xe8u8, 0xcbu8, 0x8du8, 0x01u8, 0x02u8, 0x04u8,
        0x08u8, 0x10u8, 0x20u8, 0x40u8, 0x80u8, 0x1bu8, 0x36u8, 0x6cu8,
        0xd8u8, 0xabu8, 0x4du8, 0x9au8, 0x2fu8, 0x5eu8, 0xbcu8, 0x63u8,
        0xc6u8, 0x97u8, 0x35u8, 0x6au8, 0xd4u8, 0xb3u8, 0x7du8, 0xfau8,
        0xefu8, 0xc5u8, 0x91u8, 0x39u8, 0x72u8, 0xe4u8, 0xd3u8, 0xbdu8,
        0x61u8, 0xc2u8, 0x9fu8, 0x25u8, 0x4au8, 0x94u8, 0x33u8, 0x66u8,
        0xccu8, 0x83u8, 0x1du8, 0x3au8, 0x74u8, 0xe8u8, 0xcbu8, 0x8du8,
    ];

    let tmp = t[0];
    t[0] = t[1];
    t[1] = t[2];
    t[2] = t[3];
    t[3] = tmp;

    for i in 0..4 {
        t[i] = SBOX[t[i] as usize];
    }
    t[0] = t[0] ^ RCON[rcon_i];
}

fn expand_key_test() {
    let tests = [
        ([0u8; 16],
         [
            0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
            0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8,
            0x62u8, 0x63u8, 0x63u8, 0x63u8, 0x62u8, 0x63u8, 0x63u8, 0x63u8,
            0x62u8, 0x63u8, 0x63u8, 0x63u8, 0x62u8, 0x63u8, 0x63u8, 0x63u8,
            0x9bu8, 0x98u8, 0x98u8, 0xc9u8, 0xf9u8, 0xfbu8, 0xfbu8, 0xaau8,
            0x9bu8, 0x98u8, 0x98u8, 0xc9u8, 0xf9u8, 0xfbu8, 0xfbu8, 0xaau8,
            0x90u8, 0x97u8, 0x34u8, 0x50u8, 0x69u8, 0x6cu8, 0xcfu8, 0xfau8,
            0xf2u8, 0xf4u8, 0x57u8, 0x33u8, 0x0bu8, 0x0fu8, 0xacu8, 0x99u8,
            0xeeu8, 0x06u8, 0xdau8, 0x7bu8, 0x87u8, 0x6au8, 0x15u8, 0x81u8,
            0x75u8, 0x9eu8, 0x42u8, 0xb2u8, 0x7eu8, 0x91u8, 0xeeu8, 0x2bu8,
            0x7fu8, 0x2eu8, 0x2bu8, 0x88u8, 0xf8u8, 0x44u8, 0x3eu8, 0x09u8,
            0x8du8, 0xdau8, 0x7cu8, 0xbbu8, 0xf3u8, 0x4bu8, 0x92u8, 0x90u8,
            0xecu8, 0x61u8, 0x4bu8, 0x85u8, 0x14u8, 0x25u8, 0x75u8, 0x8cu8,
            0x99u8, 0xffu8, 0x09u8, 0x37u8, 0x6au8, 0xb4u8, 0x9bu8, 0xa7u8,
            0x21u8, 0x75u8, 0x17u8, 0x87u8, 0x35u8, 0x50u8, 0x62u8, 0x0bu8,
            0xacu8, 0xafu8, 0x6bu8, 0x3cu8, 0xc6u8, 0x1bu8, 0xf0u8, 0x9bu8,
            0x0eu8, 0xf9u8, 0x03u8, 0x33u8, 0x3bu8, 0xa9u8, 0x61u8, 0x38u8,
            0x97u8, 0x06u8, 0x0au8, 0x04u8, 0x51u8, 0x1du8, 0xfau8, 0x9fu8,
            0xb1u8, 0xd4u8, 0xd8u8, 0xe2u8, 0x8au8, 0x7du8, 0xb9u8, 0xdau8,
            0x1du8, 0x7bu8, 0xb3u8, 0xdeu8, 0x4cu8, 0x66u8, 0x49u8, 0x41u8,
            0xb4u8, 0xefu8, 0x5bu8, 0xcbu8, 0x3eu8, 0x92u8, 0xe2u8, 0x11u8,
            0x23u8, 0xe9u8, 0x51u8, 0xcfu8, 0x6fu8, 0x8fu8, 0x18u8, 0x8eu8,
        ]),
    ];

    for &(key, expected) in &tests {
        let expanded = AESCipher::expand_key(&key);
        for ((i, chunk), block) in expected.chunks(16).enumerate().zip(expanded) {
            if block != chunk {
                panic!("expand_key_test expected {:?} got {:?} at chunk {}",
                       chunk, block, i);
            }
        }
    }
}

fn mix_columns_test() {
    // from wiki
    let tests = [
        ([0xdbu8, 0x13u8, 0x53u8, 0x45u8],
         [0x8eu8, 0x4du8, 0xa1u8, 0xbcu8]),
        ([0xf2u8, 0x0au8, 0x22u8, 0x5cu8],
         [0x9fu8, 0xdcu8, 0x58u8, 0x9du8]),
        ([0x01u8, 0x01u8, 0x01u8, 0x01u8],
         [0x01u8, 0x01u8, 0x01u8, 0x01u8]),
        ([0xc6u8, 0xc6u8, 0xc6u8, 0xc6u8],
         [0xc6u8, 0xc6u8, 0xc6u8, 0xc6u8]),
        ([0xd4u8, 0xd4u8, 0xd4u8, 0xd5u8],
         [0xd5u8, 0xd5u8, 0xd7u8, 0xd6u8]),
        ([0x2du8, 0x26u8, 0x31u8, 0x4cu8],
         [0x4du8, 0x7eu8, 0xbdu8, 0xf8u8]),
    ];
    for &(mut column, expected) in tests.iter() {
        AESBlock::mix_column(&mut column);
        if column != expected {
            panic!("FAILED: mix_columns expected {:?} got {:?}",
                   expected, column);
        }
    }
}

fn detect_aes_ecb(buf: &[u8]) -> u64 {
    // XXX: length not multiple of block size
    let mut score: u64 = 0;
    let mut chunks: HashSet<&[u8]> = HashSet::new();
    for chunk in buf.chunks(16) {
        if chunks.contains(chunk) {
            score += 1;
        } else {
            chunks.insert(chunk);
        }
    }
    score
}

fn detect_aes_ecb_in_file(filename: &str) {
    let f = match File::open(filename) {
        Ok(file) => file,
        Err(e) => { panic!("{}", e); }
    };

    let mut best_score: u64 = 0;
    let mut best_i: usize = 0;

    let buffered = BufReader::new(&f);
    for (i, line) in buffered.lines().enumerate() {
        let l = match line {
            Ok(line_str) => line_str,
            Err(e) => { panic!("{}", e); }
        };

        let score = detect_aes_ecb(l.as_bytes());
        if score > best_score {
            best_score = score;
            best_i = i;
        }
    }

    println!("AES ECB 1.8.txt best_score {} on line {}", best_score, best_i);
}

fn decrypt_aes_cbc_base64_file(filename: &str, key: &[u8], iv: &[u8]) {
    let f = base64_decode_file(filename);
    let cipher = AESCipher::new(key);
    let decrypted_bytes = cipher.cbc_decrypt(&f, iv);
    let decrypted = str::from_utf8(&pkcs7_unpad(&decrypted_bytes)).unwrap();
    println!("AES CBC decrypt {}:\n{}", filename, decrypted);
}

pub fn aes_test() {
    expand_key_test();
    mix_columns_test();

    let encrypt_tests = [
        ("000102030405060708090a0b0c0d0e0f",
         "00112233445566778899aabbccddeeff",
         "69c4e0d86a7b0430d8cdb78070b4c55a",),
        ("59454c4c4f57205355424d4152494e45",
         "626f6f70626f6f70626f6f70626f6f70",
         "524086dcdd3fba9d571165a93e5bf91c",),
    ];

    for &(key, plaintext, expected_ciphertext) in &encrypt_tests {
        let cipher = AESCipher::new(&hex_to_bytes(key));
        let ciphertext = cipher.encrypt_block(&hex_to_bytes(plaintext));
        if ciphertext != hex_to_bytes(expected_ciphertext) {
            panic!("FAILED: encrypt expected {} got {}",
                   expected_ciphertext, bytes_to_hex(&ciphertext));
        }

        let decrypted = cipher.decrypt_block(&ciphertext);
        if decrypted != hex_to_bytes(plaintext) {
            panic!("FAILED: decrypt expected {} got {}",
                   plaintext, bytes_to_hex(&decrypted));
        }
    }

    let f = base64_decode_file("data/1.7.txt");
    let cipher = AESCipher::new("YELLOW SUBMARINE".as_bytes());
    let decrypted_bytes = cipher.ecb_decrypt(&f);
    let decrypted = str::from_utf8(pkcs7_unpad(&decrypted_bytes)).unwrap();
    println!("AES ECB decrypt 1.7.txt:\n{}----", decrypted);

    detect_aes_ecb_in_file("data/1.8.txt");

    decrypt_aes_cbc_base64_file("data/2.10.txt",
                                "YELLOW SUBMARINE".as_bytes(),
                                &[0u8; 16]);
    distinguish_aes_cbc_ecb_test();
    println!("Finished AES tests");
}
