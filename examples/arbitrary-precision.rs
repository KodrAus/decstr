#[cfg(feature = "arbitrary-precision")]
fn main() -> Result<(), decstr::Error> {
    let decimal = decstr::BigBitstring::try_parse_str(include_str!("pi.txt"))?;

    println!("{}", decimal);
    println!("{:?}", decimal.as_le_bytes());
    
    Ok(())
}

#[cfg(not(feature = "arbitrary-precision"))]
fn main() {
    
}
