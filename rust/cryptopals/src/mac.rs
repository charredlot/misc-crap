extern crate sha2;

use self::sha2::{Sha256, Digest as SHA2Digest};

// oops used different sha1 and sha2 unfortunately
use sha1::{Sha1, Digest as SHA1Digest};
use xor::slice_xor;

pub fn sha1_cat_mac_digest(key: &[u8], message: &[u8]) -> SHA1Digest {
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

pub fn sha1_cat_mac_verify(key: &[u8], message: &[u8],
                           hmac: &SHA1Digest) -> bool {
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

fn sha1_bytes(input: &[u8]) -> Vec<u8> {
    let mut hash = Sha1::new();
    hash.update(input);
    hash.digest().bytes().to_vec()
}

fn sha256_bytes(input: &[u8]) -> Vec<u8> {
    Sha256::digest(&input).to_vec()
}

pub fn hmac(key: &[u8], msg: &[u8], block_size: usize,
            hash: fn (&[u8]) -> Vec<u8>) -> Vec<u8> {
    let opad = vec!(0x5cu8; block_size);
    let ipad = vec!(0x36u8; block_size);
    let mut key_mem = vec!(0u8; block_size);

    if key.len() > block_size {
        // hash key if key too big
        for (dst, src) in key_mem.iter_mut().zip(&hash(&key)) {
            *dst = *src;
        }
    } else {
        // already zero padded if key is too small
        for (dst, src) in key_mem.iter_mut().zip(key) {
            *dst = *src;
        }
    }

    let mut input = slice_xor(&opad, &key_mem);
    input.extend_from_slice({
        let mut ipad_input = slice_xor(&ipad, &key_mem);
        ipad_input.extend_from_slice(msg);
        &hash(&ipad_input)
    });
    hash(&input)
}

pub fn hmac_sha1(key: &[u8], msg: &[u8]) -> Vec<u8> {
    hmac(key, msg, 64, sha1_bytes)
}

pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    hmac(key, msg, 64, sha256_bytes)
}
