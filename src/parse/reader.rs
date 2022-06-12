use std::io::Error as IoError;

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
use std::io::{BufRead, BufReader, Read};

#[cfg(feature = "async_tokio")]
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

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

#[cfg(not(any(feature = "async_std", feature = "async_tokio")))]
impl<R: Read> FileReader<R> {
    pub(crate) fn new(src: R) -> Self {
        Self {
            inner: BufReader::new(src),
            buf: Vec::with_capacity(32),
        }
    }

    pub(crate) fn next_line(&mut self) -> Result<usize, IoError> {
        loop {
            self.buf.clear();
            let bytes = self.inner.read_until(b'\n', &mut self.buf)?;

            // TODO: check on lines like `       // comment continuation`
            if !skip_line(&self.buf) {
                return Ok(bytes);
            }
        }
    }
}

#[cfg(feature = "async_tokio")]
impl<R: AsyncRead + Unpin> FileReader<R> {
    pub(crate) fn new(src: R) -> Self {
        Self {
            inner: BufReader::new(src),
            buf: Vec::with_capacity(32),
        }
    }

    pub(crate) async fn next_line(&mut self) -> Result<usize, IoError> {
        loop {
            self.buf.clear();
            let bytes = self.inner.read_until(b'\n', &mut self.buf).await?;

            // TODO: check on lines like `       // comment continuation`
            if !skip_line(&self.buf) {
                return Ok(bytes);
            }
        }
    }
}

#[cfg(feature = "async_std")]
impl<R: Read + Unpin> FileReader<R> {
    pub(crate) fn new(src: R) -> Self {
        Self {
            inner: BufReader::new(src),
            buf: Vec::with_capacity(32),
        }
    }

    pub(crate) async fn next_line(&mut self) -> Result<usize, IoError> {
        loop {
            self.buf.clear();
            let bytes = self.inner.read_until(b'\n', &mut self.buf).await?;

            // TODO: check on lines like `       // comment continuation`
            if !skip_line(&self.buf) {
                return Ok(bytes);
            }
        }
    }
}

impl<R> FileReader<R> {
    pub(crate) fn is_initial_empty_line(&self) -> bool {
        let pats: [&[u8]; 5] = [
            &[b' '],
            &[b'\t'],
            &[b'\n'],
            &[b'\r'],
            &[239, 187, 191], // U+FEFF
        ];

        consists_of(&self.buf, pats)
    }

    pub(crate) fn version(&self) -> Option<u8> {
        const OSU_FILE_HEADER: &[u8] = b"osu file format v";

        self.find(OSU_FILE_HEADER)
            .and_then(|i| self.buf.get(i + OSU_FILE_HEADER.len()..))
            .map(|infix| {
                let mut n = 0;

                for byte in infix {
                    if !(b'0'..=b'9').contains(byte) {
                        break;
                    }

                    n = 10 * n + (*byte & 0xF);
                }

                n
            })
    }

    pub(crate) fn get_section(&self) -> Option<&[u8]> {
        if self.buf[0] == b'[' {
            if let Some(end) = self.find(&[b']']) {
                return Some(&self.buf[1..end]);
            }
        }

        None
    }

    pub(crate) fn get_line(&self) -> &str {
        // TODO: benchmark if necessary
        // trim comment so the utf8 validation skips it
        let end = self
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
            .unwrap_or(self.buf.len());

        std::str::from_utf8(&self.buf[..end]).unwrap().trim_end()
    }

    pub(crate) fn split_colon(&self) -> Option<(&[u8], &str)> {
        let idx = self.buf.iter().position(|&byte| byte == b':')?;
        let front = &self.buf[..idx];
        let back = std::str::from_utf8(&self.buf[idx + 1..]).ok()?;

        Some((front, back.trim()))
    }

    fn find(&self, pat: &[u8]) -> Option<usize> {
        self.buf
            .windows(pat.len())
            .enumerate()
            .find_map(|(i, window)| (window == pat).then(|| i))
    }
}

fn skip_line(line: &[u8]) -> bool {
    !line.is_empty()
        && (matches!(line[0], b'\n' | b' ' | b'_') || (line.len() >= 2 && &line[..2] == b"//"))
}

/// Check if `src` is a combination of the given patterns.
fn consists_of<const N: usize>(src: &[u8], pats: [&[u8]; N]) -> bool {
    let max_len = pats.iter().map(|pat| pat.len()).max().unwrap_or(0);
    let limit = src.len().saturating_sub(max_len);
    let mut i = 0;

    'outer1: while i < limit {
        for pat in pats {
            if &src[i..i + pat.len()] == pat {
                i += pat.len();

                continue 'outer1;
            }
        }

        return false;
    }

    'outer2: while i < src.len() {
        for pat in pats {
            if i + pat.len() <= src.len() && &src[i..i + pat.len()] == pat {
                i += pat.len();

                continue 'outer2;
            }
        }

        return false;
    }

    true
}
