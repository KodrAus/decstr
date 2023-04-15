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

# Status

This library is quite new. It's functional, but not optimized and likely contains bugs.

# IEEE 754

If you've ever used Rust's `f64`, C#'s `double`, or JavaScript's `Number`, you've been using an implementation of IEEE 754 binary (base-2) floating points. Recent versions of the same standard also specify decimal (base-10) floating point formats. They're not quite as ubiquitous as the binary ones, but interesting in their own right.

If you don't have access to a copy of the IEEE 754 standard, you can check out the open [General Decimal Arithmetic](https://speleotrove.com/decimal/) standard. It's compatible with IEEE 754.
