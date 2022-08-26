#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::ReadSeek;
use rand;
use rocket::futures::io::Cursor;
use rocket::futures::AsyncWriteExt;
use rocket::tokio::io::ReadBuf;
use rocket::tokio::runtime::Handle;
use rocket::{
    futures::executor::block_on,
    tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt},
};
use std::io::SeekFrom;
use std::pin::Pin;
use std::task::{Context, Poll};

pub type Ranges = Vec<(u64, u64)>;

/// [https://golang.org/src/mime/multipart/writer.go?s=478:513#L84](https://golang.org/src/mime/multipart/writer.go?s=478:513#L84)
/// Boundaries can be no more than 70 characters long.
fn random_boundary() -> String {
    use rand::RngCore;

    let mut x = [0 as u8; 30];
    rand::thread_rng().fill_bytes(&mut x);
    (&x[..])
        .iter()
        .map(|&x| format!("{:x}", x))
        .fold(String::new(), |mut a, x| {
            a.push_str(x.as_str());
            a
        })
}

/// An adapter for a ReadSeek that will stream the provided byteranges
/// with multipart/byteranges headers and boundaries.
pub struct MultipartReader<'a> {
    stream_len: u64,
    stream: Pin<Box<dyn ReadSeek + 'a>>,
    ranges: Ranges,

    // Current index in the ranges vector being served
    idx: usize,
    pub boundary: String,

    // the content type to be sent along with each range boundary
    content_type: String,

    // Buffer to write boundaries into.
    buffer: Cursor<Vec<u8>>,

    // if a boundary header has already been written this will be true
    wrote_boundary_header: bool,
}

impl<'a> MultipartReader<'a> {
    pub fn new<T>(
        stream: T,
        stream_len: u64,
        content_type: impl Into<String>,
        ranges: Vec<(u64, u64)>,
    ) -> MultipartReader<'a>
    where
        T: AsyncRead + AsyncSeek + Send + 'a,
    {
        let stream = Box::pin(stream);

        return MultipartReader {
            stream_len,
            stream,
            ranges,
            idx: 0,
            boundary: random_boundary(),
            content_type: content_type.into(),
            buffer: Cursor::new(Vec::new()),
            wrote_boundary_header: false,
        };
    }

    /// write the boundary into the buffer
    fn write_boundary(&mut self) -> std::io::Result<()> {
        let handle = Handle::current();
        handle.enter();
        let boundary = format!("\r\n--{}\r\n", self.boundary);
        block_on(self.buffer.write_all(boundary.as_bytes()))
    }

    /// Write a header to buffer
    fn write_boundary_header(
        &mut self,
        header: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> std::io::Result<()> {
        let handle = Handle::current();
        handle.enter();
        let header = format!("{}: {}\r\n", header.as_ref(), value.as_ref());
        block_on(self.buffer.write_all(header.as_bytes()))
    }

    /// Write CRLF to buffer
    fn write_boundary_end(&mut self) -> std::io::Result<()> {
        let handle = Handle::current();
        handle.enter();
        block_on(self.buffer.write_all("\r\n".as_bytes()))
    }

    // Close the multipart form by sending the boundary closer field.
    fn write_boundary_closer(&mut self) -> std::io::Result<()> {
        let handle = Handle::current();
        handle.enter();
        block_on(
            self.buffer
                .write_all(format!("\r\n--{}--\r\n\r\n", &self.boundary).as_bytes()),
        )
    }

    /// Empty the buffer by truncating the underlying vector
    fn clear_buffer(&mut self) {
        let handle = Handle::current();
        handle.enter();
        block_on(rocket::futures::AsyncSeekExt::seek(
            &mut self.buffer,
            SeekFrom::Start(0),
        ))
        .unwrap();
        self.buffer.get_mut().truncate(0);
    }
}

impl<'a> AsyncRead for MultipartReader<'a> {
    fn poll_read(
        mut self: Pin<&mut MultipartReader<'a>>,
        _cs: &mut Context,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let handle = Handle::current();
        handle.enter();
        // The number of bytes read into the buffer so far.
        let mut c = 0;

        // Find the length of the header buffer to see if it's not empty.
        // If so, send the remaining contents in the buffer.
        let bufsize = self.buffer.get_ref().len() as u64;
        if bufsize > 0 {
            // If the buffer isn't empty send the content within it
            if self.buffer.position() < bufsize - 1 {
                // read operations on the buffer cannot fail.
                c = block_on(rocket::futures::AsyncReadExt::read(
                    &mut self.buffer,
                    buf.initialized_mut(),
                ))
                .unwrap();
            }

            // Clear the buffer if all of it has already been read
            if self.buffer.position() >= bufsize - 1 {
                self.clear_buffer();
            }
        }

        // If we cannot fill the buffer anymore, so return with the number of bytes read.
        if c >= buf.initialized().len() {
            return Poll::Ready(Ok(()));
        }

        // All the ranges have completed being read.
        // Finalize by sending the closing boundary and
        // Returning 0 for all future calls.
        if self.idx >= self.ranges.len() {
            if self.idx == self.ranges.len() {
                self.write_boundary_closer()?;
                block_on(rocket::futures::AsyncSeekExt::seek(
                    &mut self.buffer,
                    SeekFrom::Start(0),
                ))
                .unwrap();

                // Because this is one greater than the length
                // The function will return Ok(0) from now on
                self.idx = self.idx + 1;
                return match block_on(rocket::tokio::io::AsyncReadExt::read(
                    &mut self,
                    &mut buf.initialized_mut()[c..],
                )) {
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(e) => Poll::Ready(Err(e)),
                };
            }
            return Poll::Ready(Ok(()));
        }

        // Write the range data into the remaining space in the buffer
        let (start, end) = self.ranges[self.idx];
        let current_position = match block_on(self.stream.stream_position()) {
            Ok(pos) => pos,
            Err(e) => return Poll::Ready(Err(e)),
        };

        // If we have not written a boundary header yet, we are at the beginning of a new stream
        // Write a boundary header and prepare the stream
        if !self.wrote_boundary_header {
            match block_on(self.stream.seek(SeekFrom::Start(start))) {
                Ok(_) => (),
                Err(e) => return Poll::Ready(Err(e)),
            };
            self.write_boundary()?;
            let stream_len = self.stream_len;
            self.write_boundary_header(
                "Content-Range",
                format!("bytes {}-{}/{}", start, end, stream_len).as_str(),
            )?;
            let content_type = self.content_type.clone();
            self.write_boundary_header("Content-Type", content_type)?;
            self.write_boundary_end()?;

            self.wrote_boundary_header = true;

            // Seek the buffer back to the start to prepare for being read
            // In the next call.
            block_on(rocket::futures::AsyncSeekExt::seek(
                &mut self.buffer,
                SeekFrom::Start(0),
            ))
            .unwrap();

            // Read until the boundary_header is done being sent
            return match block_on(rocket::tokio::io::AsyncReadExt::read(
                &mut self,
                &mut buf.initialized_mut()[c..],
            )) {
                Ok(_) => Poll::Ready(Ok(())),
                Err(e) => Poll::Ready(Err(e)),
            };
        }

        // Number of bytes remaining until the end
        let remaining = (end + 1 - current_position) as usize;

        // If the number of remaining bytes in this range is less than what we can fit in the buffer,
        // then we have reached the end of this range, send the final piece
        // And increment the range idx by one to move onto the next
        // Set "wrote_boundary_header" to false, allowing the next header to be created
        if buf.initialized().len() - c >= remaining {
            match block_on(rocket::tokio::io::AsyncReadExt::read_exact(
                &mut self.stream,
                &mut buf.initialized_mut()[c..remaining + c],
            )) {
                Ok(_) => (),
                Err(e) => return Poll::Ready(Err(e)),
            };
            self.idx = self.idx + 1;
            self.wrote_boundary_header = false;
            return Poll::Ready(Ok(()));
        }

        // Read the next chunk of the range
        match block_on(rocket::tokio::io::AsyncReadExt::read(
            &mut self.stream,
            &mut buf.initialized_mut()[c..],
        )) {
            Ok(_) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
