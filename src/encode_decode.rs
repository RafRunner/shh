use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};

use crate::Payload;

pub fn encode_image(input_image: &DynamicImage, payload: Payload) -> Result<DynamicImage> {
    let payload_size = payload.size();

    if !payload_fits(payload_size, image_rgb_bytes_size(input_image)) {
        return Err(anyhow!(
            "The payload is too big to be encoded in the input image. Choose a bigger image (in resolution) or compress the payload."
        ));
    }

    let payload_bytes = payload.into_bytes()?;

    let (width, height) = input_image.dimensions();
    let image_bytes = input_image.to_rgb8().into_raw();
    let chunks = create_byte_chunks(&image_bytes).take(payload_size);

    // Encode the payload
    let mut output: Vec<u8> = payload_bytes
        .iter()
        .zip(chunks)
        .flat_map(|(payload, chunk)| encode_byte_in_bytes(chunk, payload))
        .collect();

    output.reserve(image_bytes.len() - output.len());

    // Fill the rest of the image with the original bytes
    for byte in image_bytes.into_iter().skip(output.len()) {
        output.push(byte);
    }

    let image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, output).unwrap();

    Ok(DynamicImage::ImageRgb8(image_buffer))
}

pub fn decode_image(image: &DynamicImage) -> Result<(String, Vec<u8>)> {
    let image_bytes = image.to_rgb8().into_raw();

    let mut chunks = create_byte_chunks(&image_bytes);

    let file_name_size: u16 = u16::from_le_bytes(
        <[u8; 2]>::try_from(decode_chunks(&mut chunks, 2))
            .map_err(|_| anyhow!("This image probably wasn't encoded. It's too small to contain the encoded file name"))?,
    );

    let file_name = String::from_utf8(decode_chunks(&mut chunks, file_name_size as usize))
        .map_err(|_| {
            anyhow!("This image probably wasn't encoded. The file name is not valid UTF-8")
        })?;

    let payload_size: u64 = u64::from_le_bytes(
        <[u8; 8]>::try_from(decode_chunks(&mut chunks, 8))
            .map_err(|_| anyhow!("This image probably wasn't encoded. It's too small to contain the encoded payload size"))?,
    );

    let payload_size: usize = u64_to_usize(payload_size)?;
    let payload = decode_chunks(&mut chunks, payload_size);

    if payload.len() < payload_size {
        return Err(anyhow!(
            "This image probably wasn't encoded. The encoded length is smaller then expected"
        ));
    }

    Ok((file_name, payload))
}

fn decode_chunks<'a, I>(chunks: &mut I, count: usize) -> Vec<u8>
where
    I: Iterator<Item = &'a [u8; 8]>,
{
    chunks
        .by_ref()
        .take(count)
        .map(decode_byte)
        .collect::<Vec<u8>>()
}

fn image_rgb_bytes_size(image: &DynamicImage) -> usize {
    let (width, height) = image.dimensions();
    // No realistic image should overflow this
    width as usize * height as usize * 3
}

fn u64_to_usize(value: u64) -> Result<usize> {
    if value <= usize::MAX as u64 {
        Ok(value as usize)
    } else {
        Err(anyhow!(
            "Payload size {} is too big for this platform",
            value
        ))
    }
}

fn encode_byte_in_bytes(target: &[u8; 8], payload: &u8) -> [u8; 8] {
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

fn decode_byte(encoded: &[u8; 8]) -> u8 {
    let mask: u8 = 0b0000_0001;

    let mut decoded: u8 = 0;

    for (i, byte) in encoded.iter().enumerate().take(8) {
        decoded |= (mask & byte) << i;
    }

    decoded
}

fn create_byte_chunks(image_bytes: &[u8]) -> impl Iterator<Item = &[u8; 8]> {
    image_bytes
        .chunks_exact(8)
        .map(|chunk| chunk.try_into().unwrap())
}

fn payload_fits(payload_size: usize, image_rgb_size: usize) -> bool {
    payload_size
        .checked_mul(8)
        .map(|it| it <= image_rgb_size)
        .unwrap_or(false)
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
        let payload: u8 = 0b1110_1010;

        let encoded: [u8; 8] = [
            0b1010_0000,
            0b1001_0111,
            0b0100_0100,
            0b0010_1001,
            0b1000_1100,
            0b0010_1111,
            0b0100_1101,
            0b1000_1001,
        ];

        assert_eq!(encoded, encode_byte_in_bytes(&target, &payload));
        assert_eq!(decode_byte(&encoded), payload);
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
        let payload: u8 = 0b0110_1000;

        let encoded: [u8; 8] = [
            0b0110_0000,
            0b0111_0110,
            0b0000_0100,
            0b0010_1001,
            0b1000_0010,
            0b0010_1111,
            0b0100_1101,
            0b1110_1000,
        ];

        assert_eq!(encoded, encode_byte_in_bytes(&target, &payload));
        assert_eq!(decode_byte(&encoded), payload);
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
        let payload: u8 = 0b1100_1010;

        let encoded: [u8; 8] = [
            0b0010_0000,
            0b0001_0111,
            0b0000_0100,
            0b0010_1001,
            0b1000_0000,
            0b0010_1110,
            0b0100_1101,
            0b1000_1001,
        ];

        assert_eq!(encoded, encode_byte_in_bytes(&target, &payload));
        assert_eq!(decode_byte(&encoded), payload);
    }

    #[test]
    fn encode_byte_payload_all_zeros() {
        let target: [u8; 8] = [0b0101_0101; 8];
        let payload: u8 = 0b0000_0000;
        let encoded = encode_byte_in_bytes(&target, &payload);
        assert_eq!(encoded, [0b0101_0100; 8]);
        assert_eq!(decode_byte(&encoded), payload);
    }

    #[test]
    fn encode_byte_payload_all_ones() {
        let target: [u8; 8] = [0b0101_0100; 8];
        let payload: u8 = 0b1111_1111;
        let encoded = encode_byte_in_bytes(&target, &payload);
        assert_eq!(encoded, [0b0101_0101; 8]);
        assert_eq!(decode_byte(&encoded), payload);
    }

    #[test]
    fn encode_byte_payload_mixed() {
        let target: [u8; 8] = [0b0101_0100; 8];
        let payload: u8 = 0b1010_1010;
        let encoded = encode_byte_in_bytes(&target, &payload);
        assert_eq!(
            encoded,
            [
                0b0101_0100,
                0b0101_0101,
                0b0101_0100,
                0b0101_0101,
                0b0101_0100,
                0b0101_0101,
                0b0101_0100,
                0b0101_0101
            ]
        );
        assert_eq!(decode_byte(&encoded), payload);
    }

    #[test]
    fn test_payload_fits() {
        assert!(payload_fits(1, 72));
        assert!(payload_fits(100, 1000000));
        assert!(payload_fits(1000, 8065));
        assert!(!payload_fits(1000, 8063));
        assert!(!payload_fits(usize::MAX, usize::MAX));
        assert!(payload_fits(usize::MAX / 8 - 9, usize::MAX));
        assert!(!payload_fits(usize::MAX / 8 - 7, usize::MAX));
    }

    #[test]
    fn decode_byte_all_zeros() {
        let encoded: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(decode_byte(&encoded), 0);
    }

    #[test]
    fn decode_byte_all_ones() {
        let encoded: [u8; 8] = [11, 19, 101, 17, 25, 1, 13, 1];
        assert_eq!(decode_byte(&encoded), 0b1111_1111);
    }

    #[test]
    fn decode_byte_mixed() {
        let encoded: [u8; 8] = [8, 1, 12, 13, 78, 236, 116, 11];
        assert_eq!(decode_byte(&encoded), 0b1000_1010);
    }
}
