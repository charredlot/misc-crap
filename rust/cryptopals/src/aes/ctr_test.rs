use std::str;

use aes::AESCipher;
use aes::ctr::AESCipherCTR;
use base64::base64_decode;

pub fn decrypt_aes_ctr_test() {
    let encoded = "L77na/nrFsKvynd6HzOoG7GHTLXsTVu9qvY/2syLXzhPweyyMTJULu/6/kXX0KSvoOLSFQ==";
    let expected = "Yo, VIP Let's kick it Ice, Ice, baby Ice, Ice, baby ";

    let ciphertext = base64_decode(encoded);
    let cipher = AESCipherCTR::new("YELLOW SUBMARINE".as_bytes(), 0);

    let decrypted = cipher.decrypt(&ciphertext);
    match str::from_utf8(&decrypted) {
        Ok(_) => {},
        Err(_) => {
            panic!("ERROR: AES CTR expected {}\ngot {:?}",
                   expected, decrypted);
        }
    }

    println!("Finished AES CTR tests");
}
