use aes::{AESCipher, expand_key, encrypt_block, AES_BLOCK_SIZE};
use xor::slice_xor_inplace;

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
