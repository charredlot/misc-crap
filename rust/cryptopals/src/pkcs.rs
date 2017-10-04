use hex::{hex_to_bytes,bytes_to_hex};

pub fn pkcs7_pad(buf: &[u8], block_size: usize) -> Vec<u8> {
    let pad_len = (block_size - (buf.len() % block_size)) as u8;
    let mut out: Vec<u8> = Vec::with_capacity(buf.len() + (pad_len as usize));
    for &b in buf {
        out.push(b);
    }
    for _ in 0..pad_len {
        out.push(pad_len);
    }
    out
}

pub fn pkcs7_unpad<'a>(buf: &'a [u8]) -> &'a [u8] {
    if buf.len() == 0 {
        panic!("pkcs7_unpad len 0 buf");
    }
    let pad = *buf.last().unwrap() as usize;
    if pad > buf.len() {
        panic!("pkcs7_unpad padding too long {:?}", buf);
    }
    for (i, &b) in (&buf[buf.len() - pad..]).iter().enumerate() {
        if b != pad as u8 {
            panic!("bad byte at {}: {:?}", buf.len() - pad + i, buf);
        }
    }
    &buf[0..(buf.len() - pad)]
}

pub fn pkcs7_test() {
    let tests = [
        ("aabb", "aabb0202", 4),
        ("aabbccdd", "aabbccdd04040404", 4),
    ];

    for &(unpadded, padded, block_size) in &tests {
        let result = pkcs7_pad(&hex_to_bytes(unpadded), block_size);
        if &result != &hex_to_bytes(padded) {
            panic!("FAILURE: pkcs7 padding expected {} got {}",
                   padded, bytes_to_hex(&result));
        }
    }
    println!("Finished PKCS7 tests");
}
