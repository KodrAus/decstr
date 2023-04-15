# `decstr`: IEEE 754 decimal bitstrings

[![decstr](https://github.com/KodrAus/decstr/actions/workflows/ci.yml/badge.svg)](https://github.com/KodrAus/decstr/actions/workflows/ci.yml)
[![Latest version](https://img.shields.io/crates/v/decstr.svg)](https://crates.io/crates/decstr)
[![Documentation Latest](https://docs.rs/decstr/badge.svg)](https://docs.rs/decstr)

This library implements an IEEE 754 decimal floating point compatible encoding in pure Rust. It's intended to support the exchange and storage of arbitrary precision numbers in a consistent and portable way.

This library does not implement decimal arithmetic. It only supports conversion.

The source is written to be explorable for anybody interested in understanding the IEEE 754 standards for decimal floating points, and hackable for anybody wanting to adapt parts of the implementation for their own needs.

# Getting started

Add `decstr` to your `Cargo.toml`:

```toml
[dependencies.decstr]
version = "0.1.1"
```

Any Rust primitive numeric type can be encoded in a `Bitstring`:

```rust
let decimal = decstr::Bitstring::try_parse_str("123.44")?;

// 123.44
println!("{}", decimal);

// [196, 73, 48, 34]
println!("{:?}", decimal.as_le_bytes());
```

The `Bitstring` type picks the smallest encoding size that will fit a given value:

```rust
let small = decstr::Bitstring::from(1u8);
let large = decstr::Bitstring::from(u128::MAX);

assert_eq!(4, small.as_le_bytes().len());
assert_eq!(20, large.as_le_bytes().len());
```

When the `arbitrary-precision` feature is enabled, decimals of any size can be encoded:

```rust
let decimal = decstr::BigBitstring::try_parse_str(include_str!("pi.txt"))?;

// 3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458700660631558817488152092096282925409171536436789259036001133053054882046652138414695194151160943305727036575959195309218611738193261179310511854807446237996274956735188575272489122793818301194912
println!("{}", decimal);

// [156, 105, 19, 152, 15, 187, 139, 242, 164, 92, 245, 58, 83, 59, 247, 116, 121, 126, 147, 145, 13, 115, 25, 41, 100, 249, 132, 181, 11, 238, 17, 99, 148, 216, 54, 191, 214, 107, 195, 233, 133, 53, 7, 78, 52, 218, 108, 77, 33, 46, 82, 27, 225, 16, 21, 83, 204, 18, 128, 13, 89, 61, 111, 163, 173, 241, 36, 216, 170, 74, 122, 232, 32, 141, 147, 29, 99, 27, 51, 216, 128, 99, 41, 223, 41, 156, 146, 96, 58, 120, 185, 181, 64, 182, 140, 69, 180, 65, 131, 113, 58, 115, 77, 46, 167, 154, 128, 114, 170, 101, 120, 227, 215, 18, 185, 77, 75, 76, 220, 174, 230, 238, 241, 128, 144, 250, 224, 176, 227, 135, 90, 137, 201, 110, 181, 144, 112, 181, 11, 92, 16, 162, 80, 124, 36, 46, 114, 196, 76, 173, 242, 140, 132, 69, 183, 96, 42, 151, 56, 217, 48, 161, 22, 181, 11, 20, 121, 111, 120, 68, 180, 170, 216, 16, 91, 127, 128, 140, 50, 208, 139, 195, 148, 165, 150, 123, 168, 10, 105, 239, 191, 89, 158, 161, 83, 220, 156, 134, 27, 89, 76, 143, 246, 59, 118, 101, 101, 67, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34]
println!("{:?}", decimal.as_le_bytes());
```

# Status

This library is quite new. It's functional, but not optimized and likely contains bugs.

# IEEE 754

If you've ever used Rust's `f64`, C#'s `double`, or JavaScript's `Number`, you've been using an implementation of IEEE 754 binary (base-2) floating points. Recent versions of the same standard also specify decimal (base-10) floating point formats. They're not quite as ubiquitous as the binary ones, but interesting in their own right.

If you don't have access to a copy of the IEEE 754 standard, you can check out the open [General Decimal Arithmetic](https://speleotrove.com/decimal/) standard. It's compatible with IEEE 754.
