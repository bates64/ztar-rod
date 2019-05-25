use std::convert::TryInto;

pub static MAGIC: u32 = 0x59617930; // "yay0"

pub fn decompress(source: &[u8]) -> Vec<u8> {
    // Header (16 bytes)
    assert_eq!(MAGIC, u32::from_be_bytes(source[0x00..0x04].try_into().unwrap()));
    let decompressed_size = u32::from_be_bytes(source[0x04..0x08].try_into().unwrap()) as usize;
    let mut link_offset  = u32::from_be_bytes(source[0x08..0x0C].try_into().unwrap()) as usize;
    let mut source_offset = u32::from_be_bytes(source[0x0C..0x10].try_into().unwrap()) as usize;

    let mut current_command = 0u8;
    let mut command_offset  = 16;
    let mut remaining_bits  = 0;

    let mut decoded = vec![0u8; decompressed_size];
    let mut decoded_bytes = 0;

    while decoded_bytes < decompressed_size {
        if remaining_bits == 0 {
            current_command = source[command_offset];
            command_offset += 1;
            remaining_bits = 8;
        }

        if (current_command & 0x80) != 0 {
            decoded[decoded_bytes] = source[source_offset];
            source_offset += 1;
            decoded_bytes += 1;
        } else {
            let link = u16::from_be_bytes(source[link_offset..link_offset+2].try_into().unwrap());
            link_offset += 2;

            let dist = link & 0xFFF;
            let copy_src = decoded_bytes - usize::from(dist + 1);

            let mut length = link >> 12 & 0xF;
            if length == 0 {
                length = u16::from(source[source_offset]);
                length += 16;
                source_offset += 1;
            }
            length += 2;

            for i in 0..length {
                decoded[decoded_bytes] = decoded[copy_src + usize::from(i)];
                decoded_bytes += 1;
            }
        }

        current_command <<= 1;
        remaining_bits -= 1;
    }

    decoded
}
