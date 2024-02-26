use std::io::{self, Read, Write};
use uuid::Uuid;

pub struct MultipartWriter {
    boundary: String,
    pub data: Vec<u8>,
    first: bool,
}

impl MultipartWriter {
    pub fn new() -> MultipartWriter {
        MultipartWriter {
            boundary: format!("boundary-{}", Uuid::new_v4()),
            first: true,
            data: Vec::new(),
        }
    }

    pub fn new_with_boundary(boundary: &str) -> MultipartWriter {
        MultipartWriter {
            boundary: boundary.to_string(),
            first: true,
            data: Vec::new(),
        }
    }

    pub fn add(self: &mut Self, reader: &mut dyn Read) -> io::Result<u64> {
        // writer for the result
        let mut writer = std::io::BufWriter::new(&mut self.data);

        // write the boundary
        if !self.first {
            writer.write_all(b"\r\n").unwrap();
        }

        writer.write_all(b"--").unwrap();
        writer.write_all(self.boundary.as_bytes()).unwrap();
        writer.write_all(b"\r\n").unwrap();

        // write the content
        io::copy(reader, &mut writer)
    }
}
