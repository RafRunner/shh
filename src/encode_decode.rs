use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};

use crate::Payload;

pub fn encode_byte_in_bytes(target: &[u8; 8], payload: &u8) -> [u8; 8] {
    let mut mask: u8 = 0b0000_0001;
    let mut result: [u8; 8] = [0; 8];

    for i in 0..8 {
        let current_bit = payload & mask;

        let encoded = if current_bit != 0 {
            target[i] | 0b0000_0001
        } else {
            target[i] & 0b1111_1110
        };

        result[i] = encoded;
        mask <<= 1;
    }

    result
}

pub fn u64_to_u8_array(value: &u64) -> [u8; 8] {
    let mut mask: u64 = 0x00000000000000FF;
    let mut result: [u8; 8] = [0; 8];

    for i in 0..8 {
        result[i] = ((value & mask) >> 8 * i) as u8;
        mask <<= 8;
    }

    result
}

pub fn encode_image(input_image: &DynamicImage, payload: Payload) -> Result<DynamicImage> {
    let payload_size = payload.size() as u64;
    let payload_size_bytes = u64_to_u8_array(&payload_size);
    let payload = payload.into_bytes();

    let (width, height) = input_image.dimensions();
    let image_bytes = input_image.to_rgb8().into_raw();
    let chunks = image_bytes
        .chunks_exact(8)
        .map(|chunk| chunk.try_into().expect("Impossible"))
        .take(payload.len() + 8);

    let mut output: Vec<u8> = payload_size_bytes
        .iter()
        .chain(payload.iter())
        .zip(chunks)
        .flat_map(|(payload, chunk)| encode_byte_in_bytes(chunk, payload))
        .collect();

    for i in output.len()..image_bytes.len() {
        output.push(image_bytes[i]);
    }

    let image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, output).unwrap();

    Ok(DynamicImage::ImageRgb8(image_buffer))
}

pub fn decode_image(image: &DynamicImage) -> Result<Vec<u8>> {
    let image_bytes = image.to_rgb8().into_raw();
    let chunks: Vec<[u8; 8]> = image_bytes
        .chunks_exact(8)
        .map(|chunk| chunk.try_into().expect("Impossible"))
        .collect();

    let mut decoded: Vec<u8> = Vec::new();
    let mut payload_size: usize = 0;

    for i in 0..8 {
        for j in 0..8 {
            payload_size |= (((chunks[i][j] & 0b0000_0001) << j) as usize) << i * 8;
        }
    }

    if payload_size + 8 > chunks.len() {
        return Err(anyhow!(
            "This image probably wasn't encoded. The encoded length is bigger then expected"
        ));
    }

    for i in 8..(8 + payload_size) {
        let encoded = chunks[i];
        let mut decoded_byte = 0 as u8;

        for j in 0..8 {
            decoded_byte |= (encoded[j] & 0b0000_0001) << j;
        }

        decoded.push(decoded_byte);
    }

    Ok(decoded)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_byte_all_zeros() {
        let target: [u8; 8] = [
            0b1010_0000,
            0b1001_0110,
            0b0100_0100,
            0b0010_1000,
            0b1000_1100,
            0b0010_1110,
            0b0100_1100,
            0b1000_1000,
        ];
        let byte: u8 = 0b1110_1010;

        let expected: [u8; 8] = [
            0b1010_0000,
            0b1001_0111,
            0b0100_0100,
            0b0010_1001,
            0b1000_1100,
            0b0010_1111,
            0b0100_1101,
            0b1000_1001,
        ];

        assert_eq!(expected, encode_byte_in_bytes(&target, &byte));
    }

    #[test]
    fn encode_byte_all_ones() {
        let target: [u8; 8] = [
            0b0110_0001,
            0b0111_0111,
            0b0000_0101,
            0b0010_1001,
            0b1000_0011,
            0b0010_1111,
            0b0100_1101,
            0b1110_1001,
        ];
        let byte: u8 = 0b0110_1000;

        let expected: [u8; 8] = [
            0b0110_0000,
            0b0111_0110,
            0b0000_0100,
            0b0010_1001,
            0b1000_0010,
            0b0010_1111,
            0b0100_1101,
            0b1110_1000,
        ];

        assert_eq!(expected, encode_byte_in_bytes(&target, &byte));
    }

    #[test]
    fn encode_byte_random() {
        let target: [u8; 8] = [
            0b0010_0000,
            0b0001_0111,
            0b0000_0101,
            0b0010_1001,
            0b1000_0000,
            0b0010_1111,
            0b0100_1101,
            0b1000_1000,
        ];
        let byte: u8 = 0b1100_1010;

        let expected: [u8; 8] = [
            0b0010_0000,
            0b0001_0111,
            0b0000_0100,
            0b0010_1001,
            0b1000_0000,
            0b0010_1110,
            0b0100_1101,
            0b1000_1001,
        ];

        assert_eq!(expected, encode_byte_in_bytes(&target, &byte));
    }

    #[test]
    fn u64_to_u8_test_all_zeros() {
        let value: u64 = 0x0000000000000000;
        let result = u64_to_u8_array(&value);
        assert_eq!(result, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn u64_to_u8_test_all_ones() {
        let value: u64 = 0xFFFFFFFFFFFFFFFF;
        let result = u64_to_u8_array(&value);
        assert_eq!(result, [255, 255, 255, 255, 255, 255, 255, 255]);
    }

    #[test]
    fn u64_to_u8_test_alternating_bits() {
        let value: u64 = 0xAA55AA55AA55AA55;
        let result = u64_to_u8_array(&value);
        assert_eq!(result, [85, 170, 85, 170, 85, 170, 85, 170]);
    }

    #[test]
    fn u64_to_u8_test_random_value() {
        let value: u64 = 0x123456789ABCDEF0;
        let result = u64_to_u8_array(&value);
        assert_eq!(result, [240, 222, 188, 154, 120, 86, 52, 18]);
    }
}
