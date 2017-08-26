pub fn hamming_distance(l: &[u8], r: &[u8]) -> u32 {
    let mut num_bits: u32 = 0;
    for (&b0, &b1) in l.iter().zip(r) {
        let res: u8 = b0 ^ b1;
        num_bits += res.count_ones();
    }
    num_bits
}

pub fn hamming_distance_test() {
    let l = "this is a test";
    let r = "wokka wokka!!!";
    let expected: u32 = 37;
    let d = hamming_distance(l.as_bytes(), r.as_bytes());
    if d == expected {
        println!("Finished Hamming distance test");
    } else {
        println!("ERROR Hamming distance: expected {} got {}", expected, d);
    }
}
