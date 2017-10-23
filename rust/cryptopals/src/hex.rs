const NIBBLE_CHAR: [char; 16] = [
    '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
    'a', 'b', 'c', 'd', 'e', 'f',
];

pub fn bytes_to_hex(buf: &[u8]) -> String {
    let mut s = String::new();
    for &b in buf {
        s.push(NIBBLE_CHAR[((b >> 4) & 0xfu8) as usize]);
        s.push(NIBBLE_CHAR[(b & 0xfu8) as usize]);
    }
    s
}

fn byte_to_nibble(c: u8) -> u8 {
    match c as char {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' | 'A' => 10,
        'b' | 'B' => 11,
        'c' | 'C' => 12,
        'd' | 'D' => 13,
        'e' | 'E' => 14,
        'f' | 'F' => 15,
        _ => 255,
    }
}

pub fn hex_to_bytes(s: &str) -> Vec<u8> {
    // XXX: return reasonable errors
    let mut vec = Vec::with_capacity((s.len() + 1) / 2);
    let mut slice = s.as_bytes();

    if s.len() % 2 != 0 {
        vec.push(byte_to_nibble(slice[0]));
        slice = &slice[1..]
    };

    for pair in slice.chunks(2) {
        // XXX: return errors
        vec.push((byte_to_nibble(pair[0]) << 4) + byte_to_nibble(pair[1]));
    }
    vec
}
