use std::io::Error as IoError;

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
use std::io::{BufRead, BufReader, Read};

#[cfg(feature = "async_tokio")]
use tokio::io::{AsyncBufReadExt, AsyncRead as Read, BufReader};

#[cfg(feature = "async_std")]
use async_std::io::{prelude::BufReadExt, BufReader, Read};

pub(crate) struct FileReader<R> {
    buf: Vec<u8>,

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
                    inner: BufReader::new(src),
                    buf: Vec::with_capacity(32),
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

#[cfg(feature = "async_tokio")]
impl_reader!(async);

#[cfg(feature = "async_std")]
impl_reader!(async);

impl<R> FileReader<R> {
    pub(crate) fn is_initial_empty_line(&mut self) -> bool {
        // U+FEFF (BOM)
        if self.buf.starts_with(&[239, 187, 191]) {
            self.buf.rotate_left(3);

            self.buf.len() == 3
        } else {
            self.buf.is_empty()
        }
    }

    pub(crate) fn version(&self) -> Option<u8> {
        if self.buf.starts_with(b"osu file format v") {
            let mut n = 0;

            for byte in &self.buf[17..] {
                if !(b'0'..=b'9').contains(byte) {
                    break;
                }

                n = 10 * n + (*byte & 0xF);
            }

            Some(n)
        } else {
            None
        }
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
    pub(crate) fn get_line(&self) -> Option<&str> {
        std::str::from_utf8(&self.buf).ok()
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
            .find_map(|(i, byte)| (!matches!(byte, b' ' | b'\t')).then(|| i + 1))
            .unwrap_or(0);

        self.buf.truncate(len);
    }
}
