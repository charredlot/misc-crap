const BASE64_VAL_CHAR: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '+', '/'
];

pub fn base64_encode(buf : &[u8]) -> String {
    let mut s = String::new();
    for chunk in buf.chunks(3) {
        let mut triple = [0u8; 3];

        for i in 0..chunk.len() {
            triple[i] = chunk[i];
        }

        let v = [
            (triple[0] >> 2) & 0x3fu8,
            ((triple[0] & 0x3u8) << 4) + ((triple[1] >> 4) & 0xfu8),
            ((triple[1] & 0xfu8) << 2) + ((triple[2] >> 6) & 0x3u8),
            triple[2] & 0x3fu8,
        ];

        s.push(BASE64_VAL_CHAR[v[0] as usize]);
        s.push(BASE64_VAL_CHAR[v[1] as usize]);
        if chunk.len() == 1 {
            s.push('=');
            s.push('=');
        } else if chunk.len() == 2 {
            s.push(BASE64_VAL_CHAR[v[2] as usize]);
            s.push('=');
        } else {
            s.push(BASE64_VAL_CHAR[v[2] as usize]);
            s.push(BASE64_VAL_CHAR[v[3] as usize]);
        }
    }
    s
}
