//use bytes::BytesMut;
use mini_redis::{Frame, Result};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////
// a scratch implementation of a buffer like BytesMut
struct MyBuffer {
    buffer: Vec<u8>,
    cursor: usize,
}

impl MyBuffer {
    fn new() -> MyBuffer {
        MyBuffer {
            buffer: vec![0; 4096],
            cursor: 0,
        }
    }

    fn get_bytes(&mut self) -> &[u8] {
        &self.buffer[..self.cursor]
    }

    fn get_free_space(&mut self) -> &mut [u8] {
        // Grow the buffer if it's full
        if self.buffer.len() == self.cursor {
            self.buffer.resize(self.cursor * 2, 0);
        }

        &mut self.buffer[self.cursor..]
    }

    fn advance_cursor(&mut self, i: usize) -> std::result::Result<(), &str> {
        if self.cursor + i >= self.buffer.len() {
            return std::result::Result::Err("i is too big.");
        }

        self.cursor += i;
        std::result::Result::Ok(())
    }

    fn is_empty(&self) -> bool {
        self.cursor == 0
    }

    fn advance(&mut self, i: usize) -> std::result::Result<(), &str> {
        if i > self.cursor {
            return std::result::Result::Err("i > cursor");
        }

        self.buffer = self.buffer.split_off(i);
        self.cursor -= i;

        std::result::Result::Ok(())
    }
}

////////////////////////////////////////////////////////////
// a Wrapper for TcpStream to read & write Redis Frames
pub struct Connection {
    stream: BufWriter<TcpStream>,
    //buffer: BytesMut,
    buffer: MyBuffer,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            // Allocate the buffer with 4kb of capacity.
            //buffer: BytesMut::with_capacity(4096),
            buffer: MyBuffer::new(),
        }
    }

    /// Read a frame from the connection.
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // Attempt to parse a frame from the buffered data. If
            // enough data has been buffered, the frame is
            // returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // There is not enough buffered data to read a frame.
            // Attempt to read more data from the socket.
            //
            // On success, the number of bytes is returned. `0`
            // indicates "end of stream".
            let n = self.stream.read(self.buffer.get_free_space()).await?;

            if n == 0 {
                // The remote closed the connection. For this to be
                // a clean shutdown, there should be no data in the
                // read buffer. If there is, this means that the
                // peer closed the socket while sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            } else {
                self.buffer.advance_cursor(n).unwrap();
            }
        }
    }

    /// Write a frame to the connection.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
        }

        self.stream.flush().await?;

        Ok(())
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Create the `T: Buf` type.
        let mut buf = Cursor::new(self.buffer.get_bytes());

        // Check whether a full frame is available
        match Frame::check(&mut buf) {
            Ok(_) => {
                // Get the byte length of the frame
                let len = buf.position() as usize;

                // Reset the internal cursor for the
                // call to `parse`.
                buf.set_position(0);

                // Parse the frame
                let frame = Frame::parse(&mut buf)?;

                // Discard the frame from the buffer
                self.buffer.advance(len).unwrap();

                // Return the frame to the caller.
                Ok(Some(frame))
            }
            // Not enough data has been buffered
            Err(Incomplete) => Ok(None),
            // An error was encountered
            Err(e) => Err(e.into()),
        }
    }

    /// Write a decimal frame to the stream
    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}
