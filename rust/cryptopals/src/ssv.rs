use aes::AESCipher;

pub const SSV_PREFIX: &'static str = "comment1=cooking%20MCs;userdata=";
pub const SSV_SUFFIX: &'static str =
    ";comment2=%20like%20a%20pound%20of%20bacon";

const ADMIN_STR: &'static str = ";admin=true;";

pub fn ssv_aes_encrypt(cipher: &AESCipher, plaintext: &[u8]) -> Vec<u8> {
    let mut result = SSV_PREFIX.as_bytes().to_vec();
    // XXX: should strip/escape semicolons and equal signs but too lazy
    result.extend_from_slice(plaintext);
    result.extend_from_slice(SSV_SUFFIX.as_bytes());
    cipher.pad_and_encrypt(&result)
}

pub fn ssv_aes_decrypt(cipher: &AESCipher, ciphertext: &[u8]) -> Vec<u8> {
    cipher.decrypt_and_unpad(ciphertext)
}

/// Checks that output is ASCII otherwise returns error with plaintext
pub fn ssv_aes_decrypt_and_check(cipher: &AESCipher,
                                 ciphertext: &[u8]) -> Result<Vec<u8>,
                                                              Vec<u8>> {
    let result = ssv_aes_decrypt(cipher, ciphertext);
    let mut not_ascii = false;
    for b in &result {
        if *b > 127u8 {
            not_ascii = true;
            break;
        }
    }
    if not_ascii {
        Err(result)
    } else {
        Ok(result)
    }
}

// ssv: semicolon-separated values
// XXX: should take a string but we may cause invalid utf8
pub fn has_admin(buf: &[u8]) -> bool {
    let match_bytes = ADMIN_STR.as_bytes();
    let match_len = match_bytes.len();
    if match_len > buf.len() {
        return false;
    }

    for window in buf.windows(match_len) {
        if window == match_bytes {
            return true;
        }
    }
    false
}
