use std::fmt;

fn main() -> Result<(), decstr::Error> {
    let decimal = decstr::Bitstring::try_parse_str("123.44")?;

    println!("{}", decimal);
    println!("{:?}", decimal.as_le_bytes());
    
    let small = decstr::Bitstring::from(1u8);
    let large = decstr::Bitstring::from(u128::MAX);

    assert_eq!(4, small.as_le_bytes().len());
    assert_eq!(20, large.as_le_bytes().len());

    Ok(())
}
