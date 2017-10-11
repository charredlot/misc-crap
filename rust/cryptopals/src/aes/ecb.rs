use aes::{AESCipher, expand_key, encrypt_block, decrypt_block, AES_BLOCK_SIZE};

pub struct AESCipherECB {
    key_schedule: Vec<Vec<u8>>,
}

impl AESCipherECB {
    pub fn new(key: &[u8]) -> AESCipherECB {
        AESCipherECB {
            key_schedule: expand_key(key),
        }
    }
}

impl AESCipher for AESCipherECB {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for chunk in plaintext.chunks(AES_BLOCK_SIZE) {
            let mut block: Vec<u8> = encrypt_block(&self.key_schedule, chunk);
            result.append(&mut block)
        }

        result
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for chunk in ciphertext.chunks(AES_BLOCK_SIZE) {
            let mut block = decrypt_block(&self.key_schedule, chunk);
            result.append(&mut block)
        }

        result
    }
}
