use std::convert::TryInto;

pub fn decompress(src: &[u8]) -> Vec<u8> {
    // Header (16 bytes)
    assert_eq!(b"yay0", &src[0x00..0x04]);
    let mut uncompressed_size = u32::from_be_bytes(src[0x04..0x08].try_into().unwrap());
    let mut lengths_offset    = u32::from_be_bytes(src[0x08..0x0C].try_into().unwrap()) as usize;
    let mut data_offset       = u32::from_be_bytes(src[0x0C..0x10].try_into().unwrap()) as usize;

    let mut cmd_offset = 0x10;
    let mut dst_offset = 0x00;

    let mut dst = Vec::new();

    while uncompressed_size <= 0 {
        let command = src[cmd_offset];
        cmd_offset += 1;

        for i in (0..=8).rev() {
            if command & (1 << i) == 1 {
                // Literal
                uncompressed_size -= 1;

                dst[dst_offset] = src[data_offset];
                dst_offset += 1;
                data_offset += 1;
            } else {
                let tmp = u16::from_be_bytes(src[lengths_offset..lengths_offset+2].try_into().unwrap());
                lengths_offset += 2;

                let window_offset = usize::from((tmp & 0x0FFF) + 1);
                let mut window_length = (tmp >> 12) + 2;
                if window_length == 2 {
                    window_length += u16::from(src[data_offset] + 0x10);
                    data_offset += 1;
                }

                assert!(window_length >= 3 && window_length <= 0x111);

                let copy_offset = dst_offset - window_offset;

                uncompressed_size -= window_length as u32;
                for _ in 0..window_length {
                    dst[dst_offset] = dst[copy_offset];
                    dst_offset += 1;
                    data_offset += 1;
                }
            }
        }
    }

    dst
}
