#![deny(missing_docs)]

//! Provides a variant of `read_exact` that succeeds on EOF if no data has been
//! read.
//!
//! # Example
//!
//! ```
//! # fn main() {
//! use std::io;
//! # fn foo() -> io::Result<()> {
//! use std::io::prelude::*;
//! use std::fs::File;
//! use read_exact::ReadExactExt;
//!
//! let mut f = try!(File::open("foo.txt"));
//! let mut buffer = [0; 10];
//! let success = try!(f.read_exact_or_eof(&mut buffer));
//! if success {
//!     // buffer is full
//! } else {
//!     // buffer contents unchanged, file was empty
//! }
//! # Ok(())
//! # }
//! # }
//! ```

use std::io;

/// An extension trait that applies to all `std::io::Read` types.
pub trait ReadExactExt {
    /// Reads exactly the number of bytes to fill `buf`, or zero.
    ///
    /// This function returns `true` upon successful read, or `false` if no
    /// data was read. No guarantees about the contents of `buf` are provided
    /// if the function returns `false` or an error.
    fn read_exact_or_eof(&mut self, buf: &mut [u8]) -> io::Result<bool>;
}

impl<T: io::Read> ReadExactExt for T {
    fn read_exact_or_eof(&mut self, mut buf: &mut [u8]) -> io::Result<bool> {
        let mut read_some = buf.is_empty();

        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    read_some = true;
                    buf = &mut {buf}[n..];
                },
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() && read_some {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(read_some)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Read};
    use super::ReadExactExt;

    #[test]
    fn eof() {
        let mut read = io::empty();
        let mut buf = [0, 0];

        let ret = read.read_exact_or_eof(&mut buf);

        assert_eq!(ret.unwrap(), false);
    }

    #[test]
    fn ok() {
        let mut read = io::repeat(1);
        let mut buf = [0, 0];

        let ret = read.read_exact_or_eof(&mut buf);

        assert_eq!(ret.unwrap(), true);
        assert_eq!(buf, [1, 1]);
    }

    #[test]
    fn unexpected_eof() {
        let mut read = io::repeat(1).take(1);
        let mut buf = [0, 0];

        let ret = read.read_exact_or_eof(&mut buf);

        assert!(ret.is_err());
    }
}
