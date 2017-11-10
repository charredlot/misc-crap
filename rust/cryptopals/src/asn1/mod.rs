// rust enums have to be type isize, not u8
pub enum TagType {
    Integer =       0x02,
    BitString =     0x03,
    OctetString =   0x04,
    Null =          0x05,
    OID =           0x06,
    Sequence =      0x10,
}

pub const SHA1_OID: [u8; 5] = [
    0x2bu8, 0x0eu8, 0x03u8, 0x02u8, 0x1au8,
];

pub const PKCS1V15_SHA1_DIGEST_PREFIX: [u8; 15] = [
    TagType::Sequence as u8, 33u8,
        TagType::Sequence as u8, 9u8,
            // sha1 oid
            TagType::OID as u8, 5u8,
                SHA1_OID[0], SHA1_OID[1], SHA1_OID[2],
                SHA1_OID[3], SHA1_OID[4],
            // sha1 parameters (none)
            TagType::Null as u8, 0u8,
        // sha1 is 20 bytes
        TagType::OctetString as u8, 20u8,
];
