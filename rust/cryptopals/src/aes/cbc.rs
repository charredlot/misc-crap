use aes::{AESCipher, expand_key, encrypt_block, decrypt_block, AES_BLOCK_SIZE};
use xor::fixed_xor;

pub struct AESCipherCBC {
    key_schedule: Vec<Vec<u8>>,
    iv: Vec<u8>,
}

impl AESCipherCBC {
    pub fn new(key: &[u8], iv: &[u8]) -> AESCipherCBC {
        AESCipherCBC {
            key_schedule: expand_key(key),
            iv: iv.to_vec(),
        }
    }

    pub fn encrypt_iv(&self, init_iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
        assert!(init_iv.len() == AES_BLOCK_SIZE,
                "iv is not the right length {:?}", init_iv);
        let mut iv = vec![0u8; AES_BLOCK_SIZE];                                 
        for (dst, src) in iv.iter_mut().zip(init_iv) {
            *dst = *src;
        }
        let mut result: Vec<u8> = Vec::new();
        for block in plaintext.chunks(AES_BLOCK_SIZE) {
            let mixed = fixed_xor(block, &iv);
            let encrypted = encrypt_block(&self.key_schedule, &mixed);
            result.extend_from_slice(&encrypted);
            for (dst, src) in iv.iter_mut().zip(&encrypted) {
                *dst = *src;
            }
        }                                                                       
        result
    }

    pub fn decrypt_iv(&self, init_iv: &[u8], ciphertext: &[u8]) -> Vec<u8> {
        assert!(init_iv.len() == AES_BLOCK_SIZE,
                "iv is not the right length {:?}", init_iv);
        let mut iv = vec![0; AES_BLOCK_SIZE];
        for (dst, src) in iv.iter_mut().zip(init_iv) {
            *dst = *src;
        }
        let mut result: Vec<u8> = Vec::new();
        for block in ciphertext.chunks(AES_BLOCK_SIZE) {
            let decrypted_block = decrypt_block(&self.key_schedule, block);
            result.append(&mut fixed_xor(&decrypted_block, &iv));
            for (dst, src) in iv.iter_mut().zip(block) {
                *dst = *src;
            }
        }
        result
    }
}

impl AESCipher for AESCipherCBC {
    fn set_iv(&mut self, iv: &[u8]) {
        self.iv = iv.to_vec();
    }

    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        self.encrypt_iv(&self.iv, plaintext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        self.decrypt_iv(&self.iv, ciphertext)
    }
}
