use sha1::{Sha1, Digest};

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
