#[cfg(feature = "arbitrary-precision")]
fn main() -> Result<(), decstr::Error> {
    let decimal = decstr::BigBitstring::try_parse(Pi("./examples/pi.txt"))?;

    println!("{}", decimal);
    println!("{:?}", decimal.as_le_bytes());

    Ok(())
}

#[cfg(not(feature = "arbitrary-precision"))]
fn main() {}

use std::{
    fmt,
    fs,
    io::{
        self,
        Read,
    },
    str,
};

/**
An example that streams a number through `fmt::Display` from a file.

We could use `include_str!` here, but this example demonstrates `decstr`'s
stateful parsing of numbers.
*/
pub struct Pi(&'static str);

impl fmt::Display for Pi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut pi = fs::File::open(self.0).unwrap();

        let mut buf = [0; 16];

        loop {
            match pi.read(&mut buf) {
                // If we successfully read 0 bytes then we're finished
                Ok(0) => {
                    break;
                }
                // If we read some bytes then forward them through the formatter as UTF8
                Ok(n) => {
                    let buf = str::from_utf8(&buf[..n]).unwrap();
                    f.write_str(buf)?;
                }
                // If the read failed, but can be retried then try again
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                // If the read failed for any other reason then return
                Err(_) => return Err(fmt::Error),
            }
        }

        Ok(())
    }
}
