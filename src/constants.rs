// File layout
pub const QOI_MAGIC: [u8; 4] = *b"qoif";
pub const QOI_HEADER_SIZE: usize = 14;
pub const QOI_EOF: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

// 2-bit tags (stored in the top two bits of the chunk)
pub const QOI_OP_INDEX: u8 = 0x00; // 00xxxxxx
pub const QOI_OP_DIFF: u8 = 0x40; // 01xxxxxx
pub const QOI_OP_LUMA: u8 = 0x80; // 10xxxxxx
pub const QOI_OP_RUN: u8 = 0xc0; // 11xxxxxx
pub const QOI_MASK_2: u8 = 0xc0; // selects the 2-bit tag

// 8-bit tags
pub const QOI_OP_RGB: u8 = 0xfe;
pub const QOI_OP_RGBA: u8 = 0xff;

// Longest run a single QOI_OP_RUN chunk can hold (stored with a bias of -1)
pub const QOI_MAX_RUN: u8 = 62;

// Size of the previously-seen pixel table
pub const QOI_INDEX_SIZE: usize = 64;
