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
