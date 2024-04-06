use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

// response from a 5-channel LEDENET controller:
// pos  0  1  2  3  4  5  6  7  8  9 10 11 12 13
//    81 25 23 61 21 06 38 05 06 f9 01 00 0f 9d
//     |  |  |  |  |  |  |  |  |  |  |  |  |  |
//     |  |  |  |  |  |  |  |  |  |  |  |  |  checksum
//     |  |  |  |  |  |  |  |  |  |  |  |  color mode (f0 colors were set, 0f whites, 00 all were set)
//     |  |  |  |  |  |  |  |  |  |  |  cool-white  0x00 to 0xFF
//     |  |  |  |  |  |  |  |  |  |  version number
//     |  |  |  |  |  |  |  |  |  warmwhite  0x00 to 0xFF
//     |  |  |  |  |  |  |  |  blue  0x00 to 0xFF
//     |  |  |  |  |  |  |  green  0x00 to 0xFF
//     |  |  |  |  |  |  red 0x00 to 0xFF
//     |  |  |  |  |  speed: 0x01 = highest 0x1f is lowest
//     |  |  |  |  Mode WW(01), WW+CW(02), RGB(03), RGBW(04), RGBWW(05)
//     |  |  |  preset pattern
//     |  |  off(23)/on(24)
//     |  type
//     msg head
//

fn get_checksum(buf: &[u8]) -> u8 {
    buf.iter().fold(0u64, |a, b| a + (*b as u64)) as u8
}

fn get_info(mut stream: impl Read + Write) -> Result<[u8; 14], io::Error> {
    let mut query_buffer: Vec<u8> = vec![];
    query_buffer.push(0x81);
    query_buffer.push(0x8A);
    query_buffer.push(0x8B);
    query_buffer.push(get_checksum(&query_buffer));
    stream.write(&query_buffer)?;

    let mut buf: [u8; 14] = [0; 14];
    stream.read(&mut buf)?;

    Ok(buf)
}

#[derive(Debug)]
pub enum MagicHomeActionError {
    NotConnected,
    IoError(io::Error),
}

pub struct MagicHome {
    stream: Option<TcpStream>,
}

impl MagicHome {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn connect(&mut self, addr: &str) -> Result<(), io::Error> {
        let stream = TcpStream::connect(addr)?;
        let _ = get_info(&stream)?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool, MagicHomeActionError> {
        let stream = self
            .stream
            .as_ref()
            .ok_or(MagicHomeActionError::NotConnected)?;

        let info = get_info(stream).map_err(|e| MagicHomeActionError::IoError(e))?;
        let is_enabled = if info[2] == 0x24 { false } else { true };
        Ok(is_enabled)
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn set_color(&mut self, rgb: [u8; 3]) -> Result<(), MagicHomeActionError> {
        let mut stream = self
            .stream
            .as_ref()
            .ok_or(MagicHomeActionError::NotConnected)?;

        let mut buf = vec![];
        buf.push(0x31);
        for i in 0..rgb.len() {
            buf.push(rgb[i]);
        }
        buf.push(0x00);
        buf.push(0xF0);
        buf.push(0x0F);
        buf.push(get_checksum(&buf));
        stream
            .write(&buf)
            .map_err(|e| MagicHomeActionError::IoError(e))?;

        Ok(())
    }

    pub fn on_off(&mut self) -> Result<(), MagicHomeActionError> {
        let mut stream = self
            .stream
            .as_ref()
            .ok_or(MagicHomeActionError::NotConnected)?;

        let mut buf = match self.is_enabled()? {
            true => vec![0x71, 0x24, 0x0F],
            false => vec![0x71, 0x23, 0x0F],
        };
        buf.push(get_checksum(&buf));
        stream
            .write(&buf)
            .map_err(|e| MagicHomeActionError::IoError(e))?;

        Ok(())
    }
}
