#![feature(test)]
extern crate test;

use sval_number::Bitstring;

fn finite_cases() -> &'static [&'static str] {
    &[
        "123",
        "-0",
        "53346.6547e34",
        "-2432",
        "1",
        "-475765.35342",
        "-673873458673",
        "0",
        "-232.65473443e236",
        "673873458673",
        "1e17",
    ]
}

#[bench]
fn decimal_from_str_finite(b: &mut test::Bencher) {
    b.iter(|| {
        for case in finite_cases() {
            test::black_box(Bitstring::try_parse_str(case).unwrap());
        }
    });
}

#[bench]
fn decimal_to_str_finite(b: &mut test::Bencher) {
    use std::fmt::Write;

    let mut buf = String::new();

    let cases = finite_cases()
        .iter()
        .map(|case| Bitstring::try_parse_str(case).unwrap())
        .collect::<Vec<_>>();

    b.iter(|| {
        for case in &cases {
            buf.clear();

            write!(&mut buf, "{}", case).unwrap();
        }
    })
}

#[bench]
fn libdecimal128_from_str_finite(b: &mut test::Bencher) {
    b.iter(|| {
        for case in finite_cases() {
            let d: dec::Decimal128 = case.parse().unwrap();

            test::black_box(d);
        }
    });
}

#[bench]
fn libdecimal128_to_str_finite(b: &mut test::Bencher) {
    use std::fmt::Write;

    let mut buf = String::new();

    let cases = finite_cases()
        .iter()
        .map(|case| case.parse().unwrap())
        .collect::<Vec<dec::Decimal128>>();

    b.iter(|| {
        for case in &cases {
            buf.clear();

            write!(&mut buf, "{}", case).unwrap();
        }
    })
}
