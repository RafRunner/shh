# Shh 🤫

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A simple and lightweight Command Line Interface (CLI) for steganography, written in Rust.

## What is Steganography?

Steganography is the practice of concealing messages or information within other non-secret data or a physical object. Shh uses this technique to hide data within images without visible alterations.

## Features

- Hide text messages or entire files within images
- Extract hidden data from previously encoded images
- Encode the original file name in the target image
- No visible alteration to carrier images
- Lossless encoding using PNG format
- Simple and intuitive command-line interface

## How It Works

Shh encodes data by manipulating the least significant bits of the RGB channels in each pixel of an image. The process works as follows:

1. The payload (text or file) is read as a vector of bytes
1. The first 16 bytes of the image store the payload file name length in little-endian order
1. The original file name is encoded as UTF-8 bytes with lossy conversion
1. The first 64 bytes of the image store the payload length in little-endian order
1. The remaining payload is encoded bit by bit into the least significant bits of each pixel's RGB channels
1. The resulting image is saved in PNG format (lossless compression)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/RafRunner/shh
cd shh

# Build with Cargo
cargo build --release

# Optional: Move the binary to your PATH
cp target/release/shh ~/.local/bin/
```

## Usage

### Encoding Data

Hide a payload (text or file) within an image:

```bash
shh e <target_image> <payload> [output_filename]
```

Example:

```bash
# Hide a text message (will result in a file named output.txt when decoded)
shh e original.jpg "This is a secret message" hidden.png

# Hide a file
shh e original.jpg secret.zip hidden.png
```

The original file name of the payload will also be encoded in the target image.

### Decoding Data

Extract hidden data from an encoded image:

```bash
shh d <encoded_image> [output_filename]
```

Example:

```bash
# Extract to default filename (decoded.png)
shh d hidden.png

# Extract with custom filename (the extension will be recovered from the original file name)
shh d hidden.png extracted
```

The original file name will be used as the default output filename unless a custom filename is specified.

### Help

Display help information:

```bash
shh help
```

## Payload Size Calculation

To determine if your payload will fit into a specific image, use this formula:

$
\text{Maximum payload size (bytes)} = \frac{\text{width} \times \text{height} \times 3}{8}
$

For example, a Full HD image (1920 x 1080 pixels) can store approximately:

$
\text{Maximum payload size (bytes)} = \frac{1920 \times 1080 \times 3}{8} = 777,560 \, \text{bytes} \approx 759 \, \text{KB}
$

These are approximations, as the original file name and lengths have to be encoded, but serves as a good estimate.

## Limitations

- The output format is always PNG (to preserve the hidden data)
- Very large payloads require correspondingly large carrier images
- No encryption is being done by default. If someone notices the noise in the least significant bits of the image, it would be somewhat trivial to recover the date encoded

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
