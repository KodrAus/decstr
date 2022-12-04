/*!
An IEEE754-2019 compatible decimal interchange encoding.

This module implements an arbitrary precision binary format that's stored in a contiguous byte buffer.
Compared to the text format, the binary format is compact. The number can be classified by examining
a single byte. These buffers can be persisted or sent over networks to other processes consistently.
*/

mod buf;
mod combination;
mod exponent;
mod num;
mod significand;

pub use self::{
    buf::*,
    combination::*,
    exponent::*,
    num::*,
    significand::*,
};
