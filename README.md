# Shh

Shh is a simple Command Line Interface (CLI) for steganography, written in Rust.

This program allows you to hide a payload (either a text or a file) within any image, as long as there's enough room. The payload file is read as a vector of bytes. If the payload is a file, it is read directly from the disk as is, without any abstraction or interpretation. Subsequently, this payload is encoded, bit by bit, into the least significant bits of the RGB channels of the input image. The first 64 bytes of the image also store the length of the payload, following little-endian order.

The encoded image is saved to disk in .png format, chosen for its lossless yet compressed nature. To determine whether a payload will fit into a specific image, you should calculate whether `(number of pixels in the image * 3) / 8 + 64` is smaller than the size of the payload in bytes. This approach allows you to hide an image of similar resolution inside another, provided the payload image is compressed enough.

The program is also capable of decoding images that were encoded following these same rules. Make sure to provide an output file name with the correct original extension, as the extension is not stored or determined; this is an aspect that could be improved in the future.

## Usage:

### To encode a payload into an image:
    shh e <target image> <payload> <output file name, default is output.pnn>

### To decode a payload from an image:
    shh d <encoded image> <output file name, default is output.png>

### To see a simple help message:
    shh help
