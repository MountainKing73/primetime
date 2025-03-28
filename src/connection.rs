use std::io::{Cursor, Read};
use std::net::TcpStream;

struct Connection {
    stream: TcpStream,
    buffer: [u8; 1028],
    buf_size: usize,
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: [0; 1028],
            buf_size: 0,
        }
    }

    fn get_value(&mut self) -> Option<String> {
        // TODO: Change this to check if there is a line in the buffer, if not, read some more
        // also change to return Result and return connection reset error if buf_size is 0
        if self.buf_size > 0 {}
        self.buf_size = match self.stream.read(&mut self.buffer) {
            Ok(n) => n,
            Err(e) => panic!("Error reading: {}", e),
        };

        if self.buf_size == 0 {
            return None;
        }

        let mut buf = Cursor::new(&self.buffer[0..self.buf_size]);

        get_line(&mut buf)
    }
}
