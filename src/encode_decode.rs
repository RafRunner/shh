use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};

use crate::Payload;

pub fn encode_image(input_image: &DynamicImage, payload: Payload) -> Result<DynamicImage> {
    let payload_size = payload.size();

    if !payload_fits(payload_size, image_rgb_bytes_size(input_image)) {
        return Err(anyhow!(
            "The payload is too big to be coded in the input image. Choose a bigger image (in resolution) or compress the payload."
        ));
    }

    let payload_size_bytes: [u8; 8] = (payload_size as u64).to_le_bytes();
    let payload = payload.into_bytes();

    let (width, height) = input_image.dimensions();
    let image_bytes = input_image.to_rgb8().into_raw();
    let chunks = create_byte_chunks(&image_bytes).take(payload.len() + 8);

    let mut output: Vec<u8> = payload_size_bytes
        .iter()
        .chain(payload.iter())
        .zip(chunks)
        .flat_map(|(payload, chunk)| encode_byte_in_bytes(chunk, payload))
        .collect();

    output.reserve(image_bytes.len() - output.len());

    for byte in image_bytes.into_iter().skip(output.len()) {
        output.push(byte);
    }

    let image_buffer: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, output).unwrap();

    Ok(DynamicImage::ImageRgb8(image_buffer))
}

pub fn decode_image(image: &DynamicImage) -> Result<Vec<u8>> {
    check_minimum_image_size(image)?;
    let image_bytes = image.to_rgb8().into_raw();

    let mut chunks = create_byte_chunks(&image_bytes);

    let payload_size: u64 = u64::from_le_bytes(
        <[u8; 8]>::try_from(
            chunks
                .by_ref()
                .take(8)
                .map(decode_byte)
                .collect::<Vec<u8>>(),
        )
        .unwrap(),
    );

    let payload_size: usize = u64_to_usize(payload_size)?;

    if !payload_fits(payload_size, image_bytes.len()) {
        return Err(anyhow!(
            "This image probably wasn't encoded. The encoded length is bigger then expected"
        ));
    }

    let mut decoded: Vec<u8> = Vec::with_capacity(payload_size);

    for chunk in chunks.take(payload_size) {
        decoded.push(decode_byte(chunk));
    }

    Ok(decoded)
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
        .and_then(|it| it.checked_add(8))
        .map(|it| it <= image_rgb_size)
        .unwrap_or(false)
}

fn check_minimum_image_size(image: &DynamicImage) -> Result<()> {
    if image_rgb_bytes_size(image) < 64 {
        Err(anyhow!("Input image is too small."))
    } else {
        Ok(())
    }
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
        assert!(payload_fits(1, 16));
        assert!(payload_fits(100, 1000000));
        assert!(payload_fits(1000, 8009));
        assert!(!payload_fits(1000, 8007));
        assert!(!payload_fits(usize::MAX, usize::MAX));
        assert!(!payload_fits(usize::MAX - 10, usize::MAX));
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
