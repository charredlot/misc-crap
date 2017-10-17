use sha1::Sha1;

pub fn sha1_cat_mac(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut v = key.to_vec();
    v.extend_from_slice(message);

    let mut hash = Sha1::new();
    hash.reset();
    hash.update(&v);
    hash.digest().bytes().to_vec()
}
