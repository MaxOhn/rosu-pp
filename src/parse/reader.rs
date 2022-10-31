use std::io::{Error as IoError, ErrorKind as IoErrorKind};

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
use std::io::{BufRead, BufReader, Read};

#[cfg(feature = "async_tokio")]
use tokio::io::{AsyncBufReadExt, AsyncRead as Read, BufReader};

#[cfg(feature = "async_std")]
use async_std::io::{prelude::BufReadExt, BufReader, Read};

use crate::ParseError;

#[derive(Eq, PartialEq)]
enum Encoding {
    Utf8,
    Utf16,
}

pub(crate) struct FileReader<R> {
    buf: Vec<u8>,
    encoding: Encoding,

    #[cfg(feature = "async_std")]
    inner: BufReader<R>,
    #[cfg(feature = "async_tokio")]
    inner: BufReader<R>,
    #[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
    inner: BufReader<R>,
}

macro_rules! read_until {
    ($self:expr) => {{
        #[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
        {
            $self.inner.read_until(b'\n', &mut $self.buf)
        }

        #[cfg(any(feature = "async_std", feature = "async_tokio"))]
        {
            $self.inner.read_until(b'\n', &mut $self.buf).await
        }
    }};
}

#[allow(unused_macro_rules)]
macro_rules! impl_reader {
    () => {
        impl<R: Read> FileReader<R> {
            impl_reader!(@NEW);
            impl_reader!(@NEXT_LINE);
        }
    };
    (async) => {
        impl<R: Read + Unpin> FileReader<R> {
            impl_reader!(@NEW);
            impl_reader!(@ASYNC NEXT_LINE);
        }
    };
    (@NEW) => {
            pub(crate) fn new(src: R) -> Self {
                Self {
                    buf: Vec::with_capacity(32),
                    encoding: Encoding::Utf8,
                    inner: BufReader::new(src),
                }
            }
    };
    (@NEXT_LINE) => {
        pub(crate) fn next_line(&mut self) -> Result<usize, IoError> {
            impl_reader!(@NEXT_LINE_BODY, self);
        }
    };
    (@ASYNC NEXT_LINE) => {
        pub(crate) async fn next_line(&mut self) -> Result<usize, IoError> {
            impl_reader!(@NEXT_LINE_BODY, self);
        }
    };
    (@NEXT_LINE_BODY, $self:ident) => {
        loop {
            $self.buf.clear();
            let bytes = read_until!($self)?;

            if bytes == 0 {
                return Ok(bytes);
            }

            $self.truncate();

            if !$self.buf.is_empty() {
                return Ok(bytes);
            }
        }
    };
}

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
impl_reader!();

#[cfg(any(feature = "async_tokio", feature = "async_std"))]
impl_reader!(async);

impl<R> FileReader<R> {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn is_initial_empty_line(&mut self) -> bool {
        if self.buf.starts_with(&[239, 187, 191]) {
            // UTF-8
            self.buf.rotate_left(3);

            self.buf.len() == 3
        } else {
            if self.buf.starts_with(&[255, 254]) {
                // UTF-16
                self.encoding = Encoding::Utf16;
                // will rotate by 1 so this truncates `255`
                let mut sub = 1;

                if self.buf.len() >= 3 {
                    // truncate one `0` that was left after truncating `\n` when reading the line.
                    // additionally truncate `0\r` if possible.
                    sub += 1 + 2
                        * (self.buf.len() >= 4 && self.buf[self.buf.len() - 2] == b'\r') as usize;
                }

                self.buf.rotate_left(1);
                self.buf.truncate(self.buf.len() - sub);
                self.decode_utf16();
            }

            self.buf.is_empty()
        }
    }

    pub(crate) fn version(&self) -> Result<u8, ParseError> {
        self.buf
            .iter()
            .position(|&byte| byte == b'o')
            .and_then(|idx| {
                self.buf[idx..]
                    .starts_with(b"osu file format v")
                    .then_some(idx + 17)
            })
            .map(|idx| {
                let mut n = 0;

                for byte in &self.buf[idx..] {
                    if !(b'0'..=b'9').contains(byte) {
                        break;
                    }

                    n = 10 * n + (*byte & 0xF);
                }

                n
            })
            .ok_or(ParseError::IncorrectFileHeader)
    }

    /// Returns the bytes inbetween '[' and ']'.
    pub(crate) fn get_section(&self) -> Option<&[u8]> {
        if self.buf[0] == b'[' {
            if let Some(end) = self.buf[1..].iter().position(|&byte| byte == b']') {
                return Some(&self.buf[1..=end]);
            }
        }

        None
    }

    /// Parse the buffer into a string, returning `None` if the UTF-8 validation fails.
    pub(crate) fn get_line(&self) -> Result<&str, ParseError> {
        std::str::from_utf8(&self.buf)
            .map_err(|e| ParseError::IoError(IoError::new(IoErrorKind::InvalidData, Box::new(e))))
    }

    pub(crate) fn get_line_ascii(&mut self) -> Result<&str, ParseError> {
        self.buf.iter_mut().for_each(|byte| {
            if *byte >= 128 {
                *byte = b'?';
            }
        });

        std::str::from_utf8(&self.buf)
            .map_err(|e| ParseError::IoError(IoError::new(IoErrorKind::InvalidData, Box::new(e))))
    }

    /// Split the buffer at the first ':', then parse the second half into a string.
    ///
    /// Returns `None` if there is no ':' or if the second half is invalid UTF-8.
    pub(crate) fn split_colon(&self) -> Option<(&[u8], &str)> {
        let idx = self.buf.iter().position(|&byte| byte == b':')?;
        let front = &self.buf[..idx];
        let back = std::str::from_utf8(&self.buf[idx + 1..]).ok()?;

        Some((front, back.trim_start()))
    }

    /// Truncate away trailing `\r\n`, `\n`, content after `//` and whitespace
    fn truncate(&mut self) {
        if self.encoding == Encoding::Utf16 {
            self.decode_utf16();
        }

        // necessary check for the edge case `//\r\n`
        if self.buf.starts_with(&[b'/', b'/']) {
            return self.buf.clear();
        }

        // len without "//" or alternatively len without trailing "(\r)\n"
        let len = self
            .buf
            .windows(3)
            .rev()
            .step_by(2)
            .zip(1..)
            .find_map(|(window, i)| {
                if window[1] == b'/' {
                    if window[0] == b'/' {
                        return Some(self.buf.len() - 2 * i - 1);
                    } else if window[2] == b'/' {
                        return Some(self.buf.len() - 2 * i);
                    }
                }

                None
            })
            .unwrap_or_else(|| match &self.buf[..] {
                [.., b'\r', b'\n'] => self.buf.len() - 2,
                [.., b'\n'] => self.buf.len() - 1,
                _ => self.buf.len(),
            });

        // trim whitespace
        let len = self.buf[..len]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, byte)| (!matches!(byte, b' ' | b'\t')).then_some(i + 1))
            .unwrap_or(0);

        self.buf.truncate(len);
    }

    /// Assumes the buffer is of the form `[_, a, 0, b, 0, c, ...]` so it removes
    /// the first element and all the 0's, turning it into `[a, b, c, ...]`.
    fn decode_utf16(&mut self) {
        // remove the 0's
        let limit = self.buf.len() / 2 + 1;

        for i in 2..limit {
            self.buf.swap(i, i * 2 - 1);
        }

        self.buf.truncate(limit);

        // remove the first element
        // panics if buffer is empty
        self.buf.rotate_left(1);
        self.buf.truncate(self.buf.len() - 1);
    }
}
