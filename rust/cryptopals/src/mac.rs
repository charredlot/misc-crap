use sha1::{Sha1, Digest};
use xor::slice_xor;

pub fn sha1_cat_mac_digest(key: &[u8], message: &[u8]) -> Digest {
    let mut v = key.to_vec();
    v.extend_from_slice(message);

    let mut hash = Sha1::new();
    hash.reset();
    hash.update(&v);
    hash.digest()
}

pub fn sha1_cat_mac(key: &[u8], message: &[u8]) -> Vec<u8> {
    sha1_cat_mac_digest(key, message).bytes().to_vec()
}

pub fn sha1_cat_mac_verify(key: &[u8], message: &[u8], hmac: &Digest) -> bool {
    sha1_cat_mac_digest(key, message).data.state == hmac.data.state
}

pub fn sha1_pad(msg: &[u8]) -> Vec<u8> {
    let mut padded = msg.to_vec();
    sha1_pad_extend(&mut padded);
    padded
}

pub fn sha1_pad_extend(msg: &mut Vec<u8>) {
    let mut pad = [0u8; 64];
    let len = msg.len();
    let rem = len % 64;

    // add a 1 bit then 0 bits until we get len % 64 == 56
    pad[0] = 0x80;
    if rem < 56 {
        msg.extend_from_slice(&pad[..56 - rem]);
    } else {
        // gotta fill up this block then do 56 bytes in the next one
        msg.extend_from_slice(&pad[..56] as &[u8]);
        for _ in 0..64 - rem {
            msg.push(0u8);
        }
    }

    // last 8 bytes is the length of the msg in bits
    let bitlen = len * 8;
    let bitlen_as_block = [
        ((bitlen >> 56) & 0xff) as u8,
        ((bitlen >> 48) & 0xff) as u8,
        ((bitlen >> 40) & 0xff) as u8,
        ((bitlen >> 32) & 0xff) as u8,
        ((bitlen >> 24) & 0xff) as u8,
        ((bitlen >> 16) & 0xff) as u8,
        ((bitlen >>  8) & 0xff) as u8,
        ((bitlen >>  0) & 0xff) as u8,
    ];
    msg.extend_from_slice(&bitlen_as_block[..] as &[u8]);
}

pub fn hmac_sha1_digest(key: &[u8], msg: &[u8]) -> Digest {
    const SHA1_BLOCK_SIZE: usize = 64;
    const OPAD: [u8; SHA1_BLOCK_SIZE] = [0x5cu8; SHA1_BLOCK_SIZE];
    const IPAD: [u8; SHA1_BLOCK_SIZE] = [0x36u8; SHA1_BLOCK_SIZE];
    let mut key_mem = [0u8; SHA1_BLOCK_SIZE];

    if key.len() > SHA1_BLOCK_SIZE {
        // sha1 hash key if key too big
        let mut hash = Sha1::new();
        hash.update(&key);
        for (dst, src) in key_mem.iter_mut().zip(&hash.digest().bytes()) {
            *dst = *src;
        }
    } else {
        // already zero padded if key is too small
        for (dst, src) in key_mem.iter_mut().zip(key) {
            *dst = *src;
        }
    }

    let mut result_hash = Sha1::new();
    let mut input = slice_xor(&OPAD, &key_mem);
    input.extend_from_slice({
        let mut hash = Sha1::new();
        let mut ipad_input = slice_xor(&IPAD, &key_mem);
        ipad_input.extend_from_slice(msg);
        hash.update(&ipad_input);
        &hash.digest().bytes()
    });
    result_hash.update(&input);
    result_hash.digest()
}

pub fn hmac_sha1(key: &[u8], msg: &[u8]) -> Vec<u8> {
    hmac_sha1_digest(key, msg).bytes().to_vec()
}
