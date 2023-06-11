/*!
Encoding decimal significands and NaN payloads.

This module converts between text representations of integers and their encoding in the
IEEE754-2019 decimal interchange format. The encodings used along the way are:

- **ASCII digits**: like `b"555"`. These are the ASCII-coded digits that make up the number.
As bytes, they might look like `[0b0000_0101, 0b0000_0101, 0b0000_0101]`. These typically come
from the parser in the `text` module.
- **Binary coded digits**: like `0b0000_0101_0101_0101`. These are the lower 4 bits of the ASCII
digits that occupy 12 bits of a single `u16`. These are produced by `encode_ascii_digits_to_bcd`.
- **Densely packed decimal**: like `0b0000_0010_1101_0101`. These are the 12 binary coded bits
further squashed into 10 bits. These are produced by `encode_bcd_to_dpd`.

For more details on these formats, see their respective encoding functions.
*/

use crate::binary::BinaryBuf;
use core::iter;

/**
Encode the trailing digits into the decimal buffer.

This method only encodes up to `D::trailing_significand_digits` into the buffer. The last
digit that should be encoded into the combination field is returned as the `MostSignificantDigit`.
*/
pub fn encode_significand_trailing_digits<D: BinaryBuf, const N: usize>(
    decimal: &mut D,
    mut chunks: [&[u8]; N],
) -> MostSignificantDigit {
    let mut chunk_index = Some(N - 1);
    let max_digits = decimal.trailing_significand_digits();

    // The format for decimals always encodes some multiple of 3 significand digits plus 1 extra.
    // This method is only concerned with the multiple of 3 "trailing" significand digits.
    // That final "plus 1" digit will be returned and encoded later.
    debug_assert_eq!(0, max_digits % 3, "{}", max_digits);

    let decimal = decimal.bytes_mut();
    let mut digit_index = 0;
    let mut bit_index = 0;

    while digit_index < max_digits {
        match next_ascii_declet_rev(&mut chunks, &mut chunk_index) {
            Some(ascii) => {
                // Encode the 3 ASCII digits into BCD, which is a slightly compressed encoding.
                // The docs of this function describe the BCD format in more detail.
                let bcd = encode_ascii_declet_to_bcd(ascii);

                // Encode the 3 BCD digits into DPD, which is a more compressed encoding.
                // The docs of this function describe the DPD format in more detail.
                encode_bcd_declet_to_dpd(bcd, decimal, &mut bit_index);

                // If we've reached the maximum number of digits that belong to the
                // trailing significand then break the loop
                digit_index += 3;
            }
            None => break,
        }
    }

    // If there's a digit left then we filled the significand buffer
    // We'll return this last digit as the most significant digit
    if chunk_index.is_some() {
        // The most significant digit is always the first digit in the first chunk
        MostSignificantDigit::from_ascii(chunks[0][0])
    }
    // If we emptied the significand buffer then the most significant digit
    // will be zero. There are implicit leading zeroes that we don't encode
    // into the ASCII buffer
    else {
        MostSignificantDigit::zero()
    }
}

/**
Encode the trailing digits into the decimal buffer.

This method returns the final digit to encode into the combination field as the `MostSignificantDigit`.
*/
pub fn encode_significand_trailing_digits_repeat<D: BinaryBuf>(
    decimal: &mut D,
    digit: u8,
) -> MostSignificantDigit {
    // The process here is basically the same as above, except instead
    // of using a set of chunks, we fill the entire buffer with the same value
    let max_digits = decimal.trailing_significand_digits();

    debug_assert_eq!(0, max_digits % 3, "{}", max_digits);

    let decimal = decimal.bytes_mut();
    let mut digit_index = 0;
    let mut bit_index = 0;

    while digit_index < max_digits {
        let bcd = encode_ascii_declet_to_bcd([digit, digit, digit]);

        encode_bcd_declet_to_dpd(bcd, decimal, &mut bit_index);

        digit_index += 3;
    }

    MostSignificantDigit::from_ascii(digit)
}

/**
Decode and stream the trailing digits encoded into the decimal.
*/
pub fn decode_significand_trailing_declets<D: BinaryBuf>(
    decimal: &D,
) -> impl Iterator<Item = [u8; 3]> + '_ {
    let mut bit_index = decimal.trailing_significand_width_bits();

    let decimal = decimal.bytes();

    iter::from_fn(move || {
        // If there's another declet to read then yield it
        if bit_index > 0 {
            let bcd = decode_dpd_declet_to_bcd(decimal, &mut bit_index);

            Some(decode_bcd_declet_to_ascii(bcd))
        }
        // If there are no digits and no declets then we're finished
        else {
            None
        }
    })
}

/**
Get the next 3 digits from the back of the buffer to encode.

The digits will be yielded in _reverse_ order. Digits will be merged from each chunk as they're yielded.
If we've reached the end of all buffers then any remaining digits will be left `0`. So if the
buffer contains `[1, 2, 3, 4]` the results returned by this method will be `[4, 3, 2]`, then `[1, 0, 0]`.
*/
fn next_ascii_declet_rev(chunks: &mut [&[u8]], chunk_index: &mut Option<usize>) -> Option<[u8; 3]> {
    match *chunk_index {
        Some(c) => {
            let chunk = chunks[c];

            let (mut out, mut out_index) = match chunk.len() {
                // Grab the last byte from the chunk. We might be crossing a chunk boundary or have finished.
                1 => {
                    let out = [chunk[0], b'0', b'0'];

                    *chunk_index = c.checked_sub(1);

                    (out, 1)
                }
                // Grab the last 2 bytes from the chunk. We might be crossing a chunk boundary or have finished.
                2 => {
                    let out = [chunk[1], chunk[0], b'0'];

                    *chunk_index = c.checked_sub(1);

                    (out, 2)
                }
                // Fast path: Read the next 3 digits in a single pass, then return.
                // The next call will start from the next chunk or have finished.
                3 => {
                    let out = [chunk[2], chunk[1], chunk[0]];

                    *chunk_index = c.checked_sub(1);

                    return Some(out);
                }
                // Fast path: Read the next 3 digits in a single pass, then return
                _ => {
                    debug_assert_ne!(0, chunk.len(), "all chunks must have at least 1 digit");

                    let out = [
                        chunk[chunk.len() - 1],
                        chunk[chunk.len() - 2],
                        chunk[chunk.len() - 3],
                    ];

                    chunks[c] = &chunk[..chunk.len() - 3];

                    return Some(out);
                }
            };

            // If we get this far then we've hit the end of the current chunk.
            //
            // This might be the end of the digits, or there may be more chunks.

            while let Some(c) = *chunk_index {
                // Slow path: Read the next 3 digits, 1 digit at a time

                if out_index == 3 {
                    return Some(out);
                }

                let chunk = chunks[c];

                match chunk.len() {
                    // If the chunk is empty, move on to the next one
                    0 => *chunk_index = c.checked_sub(1),
                    // If this is the last byte in the chunk, fetch it and move on to the next one
                    1 => {
                        out[out_index] = chunk[0];
                        out_index += 1;

                        *chunk_index = c.checked_sub(1);
                    }
                    // If there's more than 1 byte left, fetch 1 and shift it out of the chunk
                    _ => {
                        out[out_index] = chunk[chunk.len() - 1];
                        out_index += 1;

                        chunks[c] = &chunk[..chunk.len() - 1];
                    }
                }
            }

            Some(out)
        }
        // If there are no chunks left then we're done
        None => None,
    }
}

/**
The most significant digit to encode into the combination field.

The value is encoded as a regular binary number, not an ASCII digit.
*/
#[derive(Clone, Copy)]
pub struct MostSignificantDigit(u8);

impl MostSignificantDigit {
    /**
    The most significant digit is zero.
    */
    pub(crate) fn zero() -> Self {
        MostSignificantDigit(0)
    }

    /**
    The most significant digit is potentially non-zero.
    */
    pub(crate) fn from_ascii(digit: u8) -> Self {
        MostSignificantDigit(encode_ascii_digit_to_bcd(digit))
    }

    /**
    The most significant digit is potentially non-zero.
    */
    pub(crate) fn from_bcd(digit: u8) -> Self {
        MostSignificantDigit(digit)
    }

    /**
    Get the most significant digit as BCD.
    */
    pub fn get_bcd(self) -> u8 {
        self.0
    }

    /**
    Get the most significant digit as ASCII.
    */
    pub fn get_ascii(self) -> u8 {
        decode_bcd_digit_to_ascii(self.0)
    }
}

/**
Encode a single ASCII digit into binary coded decimal (BCD).

BCD is a translation from ASCII that simply uses the lower 4 bits of each digit. In ASCII,
the lower 4 bits of digits are encoded the same way as their binary integer equivalents. For example,
the number `6` in ASCII is `0b0011_0110`, and as a binary integer is `0000_0110`. Since the higher
4 bits of each ASCII digit are always the same, we can ignore them. That lets us squash 3 digits
handily into a `u16` (it would technically fit 4, but we only need 3).
*/
fn encode_ascii_digit_to_bcd(ascii: u8) -> u8 {
    ascii - b'0'
}

/**
Decode a single binary coded decimal (BCD) into an ASCII digit.

There are some details on what BCD is in the encoding function.
*/
fn decode_bcd_digit_to_ascii(bcd: u8) -> u8 {
    bcd + b'0'
}

/**
Encode ASCII digits into binary coded decimal (BCD).
*/
fn encode_ascii_declet_to_bcd(ascii: [u8; 3]) -> u16 {
    let d0 = encode_ascii_digit_to_bcd(ascii[0]) as u16;
    let d1 = (encode_ascii_digit_to_bcd(ascii[1]) as u16) << 4;
    let d2 = (encode_ascii_digit_to_bcd(ascii[2]) as u16) << 8;

    d2 | d1 | d0
}

/**
Decode binary coded decimal (BCD) into ASCII digits.
*/
fn decode_bcd_declet_to_ascii(bcd: u16) -> [u8; 3] {
    const D2: u16 = 0b0000_1111_0000_0000u16;
    const D1: u16 = 0b0000_0000_1111_0000u16;
    const D0: u16 = 0b0000_0000_0000_1111u16;

    let d2 = decode_bcd_digit_to_ascii(((bcd & D2) >> 8) as u8);
    let d1 = decode_bcd_digit_to_ascii(((bcd & D1) >> 4) as u8);
    let d0 = decode_bcd_digit_to_ascii((bcd & D0) as u8);

    [d2, d1, d0]
}

/**
Compress binary coded decimal (BCD) into densely packed decimal (DPD).

DPD is an encoding that compresses 3 decimal digits (24 bits in ASCII, 12 bits in BCD) into 10 bits.
It's one of the encodings for decimal numbers supported by IEEE 754-2008 decimals, and the one used
by the format provided by this library.

The encoding works by squashing 12 bits into 10. Say we have the digits `123`. These are encoded
using BCD into a `u16`, with the least significant digit in the least significant position:

```text
m                  l
        ---1---2---3
00000000000100100011
```

DPD looks at the most significant bit of each digit to decide how to encode the set:

```text
m                  l
        ---1---2---3
00000000x001x010x011
```

It ends up squashing the digits into 10 bits, saving 2 bits for every 3 digits. It seems like
a lot of effort for a small saving, but those 2 bits make a big difference to the precision of
a decimal format!
*/
fn encode_bcd_declet_to_dpd(bcd: u16, decimal: &mut [u8], decimal_bit_index: &mut usize) {
    const D: u16 = D2 | D1 | D0;

    const D01: u16 = D0 | D1;
    const D12: u16 = D1 | D2;
    const D02: u16 = D0 | D2;

    // The last bcd digit
    const D2: u16 = 0b0000_0000_0000_1000;
    const DG: u16 = 0b0000_0000_0000_0100;
    const DH: u16 = 0b0000_0000_0000_0010;
    const DI: u16 = 0b0000_0000_0000_0001;

    // The middle bcd digit
    const D1: u16 = 0b0000_0000_1000_0000;
    const DD: u16 = 0b0000_0000_0100_0000;
    const DE: u16 = 0b0000_0000_0010_0000;
    const DF: u16 = 0b0000_0000_0001_0000;

    // The first bcd digit
    const D0: u16 = 0b0000_1000_0000_0000;
    const DA: u16 = 0b0000_0100_0000_0000;
    const DB: u16 = 0b0000_0010_0000_0000;
    const DC: u16 = 0b0000_0001_0000_0000;

    const B0: u16 = 0b0000_0000_0000_0001;

    // You can do DPD encoding using a few approaches, such as a table lookup or with a few boolean operations,
    // but we use this simple approach of shifting each bit into position because it's straight-forward.
    // We branch on the most significant bits of each BCD digit, which results in 8 possible
    // encodings for our 3 digits. If we wanted to optimize this, we'd pre-compute a table for all
    // 1000 3-digit numbers.
    let dpd = match bcd & D {
        // Three small digits
        // bcd: 0abc0def0ghi
        // dpd:   abcdef0ghi
        0 => {
            let bit0 = (bcd & DA) >> 1;
            let bit1 = (bcd & DB) >> 1;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = bcd & DD;
            let bit4 = bcd & DE;
            let bit5 = bcd & DF;

            let bit6 = 0;

            let bit7 = bcd & DG;
            let bit8 = bcd & DH;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // The last digit is large
        // bcd: 0abc0def100i
        // dpd:   abcdef100i
        D2 => {
            let bit0 = (bcd & DA) >> 1;
            let bit1 = (bcd & DB) >> 1;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = bcd & DD;
            let bit4 = bcd & DE;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = 0;
            let bit8 = 0;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // The middle digit is large
        // bcd: 0abc100f0ghi
        // dpd:   abcghf101i
        D1 => {
            let bit0 = (bcd & DA) >> 1;
            let bit1 = (bcd & DB) >> 1;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = (bcd & DG) << 4;
            let bit4 = (bcd & DH) << 4;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = 0;
            let bit8 = B0 << 1;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // The first digit is large
        // bcd: 100c0def0ghi
        // dpd:   ghcdef110i
        D0 => {
            let bit0 = (bcd & DG) << 7;
            let bit1 = (bcd & DH) << 7;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = bcd & DD;
            let bit4 = bcd & DE;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = B0 << 2;
            let bit8 = 0;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // The first two digits are large
        // bcd: 100c100f0ghi
        // dpd:   ghc00f111i
        D01 => {
            let bit0 = (bcd & DG) << 7;
            let bit1 = (bcd & DH) << 7;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = 0;
            let bit4 = 0;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = B0 << 2;
            let bit8 = B0 << 1;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // The last two digits are large
        // bcd: 0abc100f100i
        // dpd:   abc10f111i
        D12 => {
            let bit0 = (bcd & DA) >> 1;
            let bit1 = (bcd & DB) >> 1;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = B0 << 6;
            let bit4 = 0;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = B0 << 2;
            let bit8 = B0 << 1;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // The first and last digits are large
        // bcd: 100c0def100i
        // dpd:   dec01f111i
        D02 => {
            let bit0 = (bcd & DD) << 3;
            let bit1 = (bcd & DE) << 3;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = 0;
            let bit4 = B0 << 5;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = B0 << 2;
            let bit8 = B0 << 1;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        // All three digits are large
        // bcd: 100c100f100i
        // dpd:   xxc11f111i
        D => {
            let bit0 = 0;
            let bit1 = 0;
            let bit2 = (bcd & DC) >> 1;

            let bit3 = B0 << 6;
            let bit4 = B0 << 5;
            let bit5 = bcd & DF;

            let bit6 = B0 << 3;

            let bit7 = B0 << 2;
            let bit8 = B0 << 1;
            let bit9 = bcd & DI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9
        }
        _ => unreachable!(),
    };

    // We need to write 10 bits into our byte buffer.
    //
    // These 10 bits will always cross a byte boundary, so we shift it
    // into the next 2 bytes of the buffer.
    //
    // Let's say we have:
    //
    // dpd    : dddddddddd
    // dpd_buf: aaaaaaaabbbbbbbbcccccccc
    //
    // We want to splat the 10 bits of `dpd` into the next 2 8-bit bytes
    // of `dpd_buf`. We start from an offset of 0, and then write 8
    // bits of `dpd` into byte `a`:
    //
    // dpd    : --------dd
    // dpd_buf: ddddddddbbbbbbbbcccccccc
    //
    // We've got two bytes left to write, so to get them into the right
    // place we shift them over, and then write them into byte `b`:
    //
    // dpd    : ----------
    // dpd_buf: ddddddddddbbbbbbcccccccc
    //
    // The next time we come around, we need to start from the offset we
    // left off in `b`:
    //
    // dpd    :         --dddddddddd
    // dpd_buf: ddddddddddbbbbbbcccccccc
    //
    // And write into the trailing byte `c`, just like last time:
    //
    // dpd    :         --------dddd
    // dpd_buf: ddddddddddddddddcccccccc
    //
    // dpd    :         ------------
    // dpd_buf: ddddddddddddddddddddcccc

    let decimal_byte_shift = (*decimal_bit_index % 8) as u32;
    let decimal_byte_index = *decimal_bit_index / 8;

    decimal[decimal_byte_index] |= (dpd << decimal_byte_shift) as u8;
    decimal[decimal_byte_index + 1] |= (dpd >> (8 - decimal_byte_shift)) as u8;

    *decimal_bit_index += 10;
}

/**
Decompress densely packed decimal (DPD) into binary coded decimal (BCD).

There are some details on what BCD and DPD are in the encoding function.
*/
fn decode_dpd_declet_to_bcd(decimal: &[u8], decimal_bit_index: &mut usize) -> u16 {
    // Follow the reverse process of encoding.
    //
    // There's some details on how the 10 DPD bits are written across 2 bytes in the encoder.

    *decimal_bit_index -= 10;

    let decimal_byte_shift = (*decimal_bit_index % 8) as u32;
    let decimal_byte_index = *decimal_bit_index / 8;

    let dpd0 = (decimal[decimal_byte_index] as u16) >> decimal_byte_shift;
    let dpd1 = (decimal[decimal_byte_index + 1] as u16) << (8 - decimal_byte_shift);

    let dpd = dpd0 | dpd1;

    // The last decoding group
    const B0: u16 = 0b0000_0000_0000_0001u16;
    const B1: u16 = 0b0000_0000_0000_0010u16;
    const B2: u16 = 0b0000_0000_0000_0100u16;
    const B3: u16 = 0b0000_0000_0000_1000u16;

    // The first decoding group
    const B4: u16 = 0b0000_0000_0001_0000u16;
    const B5: u16 = 0b0000_0000_0010_0000u16;

    // Extra bits that need to be shifted to decode
    const B6: u16 = 0b0000_0000_0100_0000u16;
    const B7: u16 = 0b0000_0000_1000_0000u16;
    const B8: u16 = 0b0000_0001_0000_0000u16;
    const B9: u16 = 0b0000_0010_0000_0000u16;

    const B13: u16 = B1 | B3;
    const B23: u16 = B2 | B3;
    const B123: u16 = B1 | B2 | B3;

    const B1235: u16 = B1 | B2 | B3 | B5;
    const B1236: u16 = B1 | B2 | B3 | B6;
    const B12356: u16 = B1 | B2 | B3 | B5 | B6;

    // The last bcd digit
    const BA: u16 = B9;
    const BB: u16 = B8;
    const BC: u16 = B7;

    // The middle bcd digit
    const BD: u16 = B6;
    const BE: u16 = B5;
    const BF: u16 = B4;

    // The first bcd digit
    const BG: u16 = B2;
    const BH: u16 = B1;
    const BI: u16 = B0;

    match dpd {
        // Three small digits
        // bcd: 0abc0def0ghi
        // dpd:   abcdef0ghi
        dpd if dpd & B3 == 0 => {
            let bit0 = 0;
            let bit1 = (dpd & BA) << 1;
            let bit2 = (dpd & BB) << 1;
            let bit3 = (dpd & BC) << 1;

            let bit4 = 0;
            let bit5 = dpd & BD;
            let bit6 = dpd & BE;
            let bit7 = dpd & BF;

            let bit8 = 0;
            let bit9 = dpd & BG;
            let bit10 = dpd & BH;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // The last digit is large
        // bcd: 0abc0def100i
        // dpd:   abcdef100i
        dpd if dpd & B123 == B3 => {
            let bit0 = 0;
            let bit1 = (dpd & BA) << 1;
            let bit2 = (dpd & BB) << 1;
            let bit3 = (dpd & BC) << 1;

            let bit4 = 0;
            let bit5 = dpd & BD;
            let bit6 = dpd & BE;
            let bit7 = dpd & BF;

            let bit8 = B0 << 3;
            let bit9 = 0;
            let bit10 = 0;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // The middle digit is large
        // bcd: 0abc100f0ghi
        // dpd:   abcghf101i
        dpd if dpd & B123 == B13 => {
            let bit0 = 0;
            let bit1 = (dpd & BA) << 1;
            let bit2 = (dpd & BB) << 1;
            let bit3 = (dpd & BC) << 1;

            let bit4 = B0 << 7;
            let bit5 = 0;
            let bit6 = 0;
            let bit7 = dpd & BF;

            let bit8 = 0;
            let bit9 = (dpd & B6) >> 4;
            let bit10 = (dpd & B5) >> 4;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // The first digit is large
        // bcd: 100c0def0ghi
        // dpd:   ghcdef110i
        dpd if dpd & B123 == B23 => {
            let bit0 = B0 << 11;
            let bit1 = 0;
            let bit2 = 0;
            let bit3 = (dpd & BC) << 1;

            let bit4 = 0;
            let bit5 = dpd & BD;
            let bit6 = dpd & BE;
            let bit7 = dpd & BF;

            let bit8 = 0;
            let bit9 = (dpd & B9) >> 7;
            let bit10 = (dpd & B8) >> 7;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // The first two digits are large
        // bcd: 100c100f0ghi
        // dpd:   ghc00f111i
        dpd if dpd & B12356 == B123 => {
            let bit0 = B0 << 11;
            let bit1 = 0;
            let bit2 = 0;
            let bit3 = (dpd & BC) << 1;

            let bit4 = B0 << 7;
            let bit5 = 0;
            let bit6 = 0;
            let bit7 = dpd & BF;

            let bit8 = 0;
            let bit9 = (dpd & B9) >> 7;
            let bit10 = (dpd & B8) >> 7;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // The last two digits are large
        // bcd: 0abc100f100i
        // dpd:   abc10f111i
        dpd if dpd & B12356 == B1236 => {
            let bit0 = 0;
            let bit1 = (dpd & BA) << 1;
            let bit2 = (dpd & BB) << 1;
            let bit3 = (dpd & BC) << 1;

            let bit4 = B0 << 7;
            let bit5 = 0;
            let bit6 = 0;
            let bit7 = dpd & BF;

            let bit8 = B0 << 3;
            let bit9 = 0;
            let bit10 = 0;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // The first and last digits are large
        // bcd: 100c0def100i
        // dpd:   dec01f111i
        dpd if dpd & B12356 == B1235 => {
            let bit0 = B0 << 11;
            let bit1 = 0;
            let bit2 = 0;
            let bit3 = (dpd & BC) << 1;

            let bit4 = 0;
            let bit5 = (dpd & B9) >> 3;
            let bit6 = (dpd & B8) >> 3;
            let bit7 = dpd & BF;

            let bit8 = B0 << 3;
            let bit9 = 0;
            let bit10 = 0;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        // All three digits are large
        // bcd: 100c100f100i
        // dpd:   xxc11f111i
        dpd if dpd & B12356 == B12356 => {
            let bit0 = B0 << 11;
            let bit1 = 0;
            let bit2 = 0;
            let bit3 = (dpd & BC) << 1;

            let bit4 = B0 << 7;
            let bit5 = 0;
            let bit6 = 0;
            let bit7 = dpd & BF;

            let bit8 = B0 << 3;
            let bit9 = 0;
            let bit10 = 0;
            let bit11 = dpd & BI;

            bit0 | bit1 | bit2 | bit3 | bit4 | bit5 | bit6 | bit7 | bit8 | bit9 | bit10 | bit11
        }
        _ => unreachable!(),
    }
}

// These methods follow the formulas given in the IEEE754-2019 standard.
//
// The standard defines the following parameters for decimal floating points that determine
// the range of exponent and significand values they can encode:
//
// - `k`: storage width in bits.
// - `p`: precision in digits.
// - `emax`: the maximum exponent.
// - `bias`: the value to bias exponents with, so that they're always positive integers.

/**
Calculate the number of digits a decimal with a given bit-width can hold.
*/
pub(crate) fn precision_digits(storage_width_bits: usize) -> usize {
    // p = 9k / 32 - 2
    9 * storage_width_bits / 32 - 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::FixedBinaryBuf;

    use std::str;

    #[test]
    fn precision_32() {
        assert_eq!(7, precision_digits(32));
    }

    #[test]
    fn precision_64() {
        assert_eq!(16, precision_digits(64));
    }

    #[test]
    fn precision_96() {
        assert_eq!(25, precision_digits(96));
    }

    #[test]
    fn precision_128() {
        assert_eq!(34, precision_digits(128));
    }

    #[test]
    fn precision_160() {
        assert_eq!(43, precision_digits(160));
    }

    #[test]
    fn precision_256() {
        assert_eq!(70, precision_digits(256));
    }

    #[test]
    fn encode_decode_dpd_declet_all() {
        for b0 in b'0'..=b'9' {
            for b1 in b'0'..=b'9' {
                for b2 in b'0'..=b'9' {
                    let digits = [b0, b1, b2];
                    let digits = str::from_utf8(&digits).expect("digit is valid UTF8");

                    let mut dpd = [0, 0];
                    let mut i = 0;

                    encode_bcd_declet_to_dpd(
                        encode_ascii_declet_to_bcd([b2, b1, b0]),
                        &mut dpd,
                        &mut i,
                    );

                    let decoded_ascii =
                        decode_bcd_declet_to_ascii(decode_dpd_declet_to_bcd(&dpd, &mut i));

                    assert_eq!([b0, b1, b2], decoded_ascii, "{}", digits);
                }
            }
        }
    }

    #[test]
    fn encode_decode_dpd_declets_across_bytes() {
        let digits = "277386910789029981476348954311894750984836542397645";

        let mut dpd = [0u8; 64];
        let mut i = 0;

        // Declets are encoded in reverse
        let digits_rev = {
            let mut digits = Vec::from(digits);
            digits.reverse();
            digits
        };
        for ascii in digits_rev.chunks(3) {
            let bcd = encode_ascii_declet_to_bcd([ascii[0], ascii[1], ascii[2]]);
            encode_bcd_declet_to_dpd(bcd, &mut dpd, &mut i);
        }

        let mut decoded_digits = Vec::new();
        while i > 0 {
            let bcd = decode_dpd_declet_to_bcd(&dpd, &mut i);
            let ascii = decode_bcd_declet_to_ascii(bcd);

            decoded_digits.extend(ascii);
        }

        assert_eq!(
            digits,
            str::from_utf8(&decoded_digits).expect("digits are valid UTF8")
        );
    }

    #[test]
    fn encode_decode_significand_trailing_digits() {
        // NOTE: This is exactly the number of digits that fills a 64bit decimal's trailing significand
        let digits = "129054729387659";

        let mut decimal = FixedBinaryBuf::<8, i32>::ZERO;

        encode_significand_trailing_digits(&mut decimal, [digits.as_bytes()]);

        let decoded = decode_significand_trailing_declets(&decimal)
            .flatten()
            .map(|b| b as char)
            .collect::<String>();

        assert_eq!(digits, decoded);
    }

    #[test]
    fn encode_decode_significand_trailing_digits_repeat() {
        for digit in b'0'..=b'9' {
            let mut decimal = FixedBinaryBuf::<8, i32>::ZERO;

            encode_significand_trailing_digits_repeat(&mut decimal, digit);

            assert!(decode_significand_trailing_declets(&decimal)
                .flatten()
                .all(|b| b == digit));
        }
    }
}
