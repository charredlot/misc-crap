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

