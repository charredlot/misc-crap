use std::cmp::min;

use aes::{AESCipher, expand_key, encrypt_block, AES_BLOCK_SIZE};
use xor::{slice_xor, slice_xor_inplace};

pub struct AESCipherCTR {
    // XXX: don't allow control of counter for now
    key_schedule: Vec<Vec<u8>>,
    pub nonce_le: u64,
}

impl AESCipherCTR {
    pub fn new(key: &[u8], nonce_le: u64) -> AESCipherCTR {
        AESCipherCTR {
            key_schedule: expand_key(key),
            nonce_le: nonce_le,
        }
    }

    fn ctr_mode(&self, text: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let mut counter = vec![0u8; AES_BLOCK_SIZE];

        u64_fill_slice_le(&mut counter[..8], self.nonce_le);

        for (i, chunk) in text.chunks(AES_BLOCK_SIZE).enumerate() {
            u64_fill_slice_le(&mut counter[8..], i as u64);
            let mut keystream = encrypt_block(&self.key_schedule, &counter);

            // xor into keystream since we're going to drop it
            slice_xor_inplace(&mut keystream, chunk);

            // might have partial chunk at end
            result.extend_from_slice(&keystream[..chunk.len()]);
        }

        result
    }

    pub fn edit(&self, ciphertext: &mut [u8], offset: usize,
                new_plaintext: &[u8]) {
        assert!(offset + new_plaintext.len() <= ciphertext.len());

        let mut counter = vec![0u8; AES_BLOCK_SIZE];
        u64_fill_slice_le(&mut counter[..8], self.nonce_le);

        let mut counter_val = (offset / AES_BLOCK_SIZE) as u64;
        let mut old_offset = offset;
        let mut new_offset = 0;

        while old_offset < ciphertext.len() &&
                new_offset < new_plaintext.len() {
            u64_fill_slice_le(&mut counter[8..], counter_val);
            let key_block = encrypt_block(&self.key_schedule, &counter);

            let block_start = counter_val as usize * AES_BLOCK_SIZE;
            let block_end = min(block_start + AES_BLOCK_SIZE,
                                ciphertext.len());
            let mut plain_block =
                slice_xor(&key_block, &ciphertext[block_start..block_end]);

            let rem = old_offset % AES_BLOCK_SIZE;
            let new_len = AES_BLOCK_SIZE - rem;
            let new_end = min(new_offset + new_len, new_plaintext.len());
            for (dst, src) in plain_block[rem..].iter_mut().zip(
                    &new_plaintext[new_offset..new_end]) {
                *dst = *src;
            }

            // yikes lol
            for ((dst, kb), pb) in ciphertext[block_start..block_end]
                                    .iter_mut()
                                    .zip(&key_block)
                                    .zip(&plain_block) {
                *dst = *kb ^ *pb;
            }

            old_offset = old_offset + new_len;
            new_offset += new_len;
            counter_val += 1;
        }
    }
}

fn u64_fill_slice_le(dst: &mut [u8], src: u64){
    // XXX: use iter of some sort?
    for i in 0..8 {
        dst[i] = ((src >> (i * 8)) as u8) & 0xffu8;
    }
}

impl AESCipher for AESCipherCTR {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        self.ctr_mode(plaintext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        self.ctr_mode(ciphertext)
    }
}
