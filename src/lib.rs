use std::io::{Read, Write, Error};
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

fn get_state(mut stream: impl Read + Write) -> Result<[u8; 14], Error> {
    let mut query_buf: Vec<u8> = vec![];
    query_buf.push(0x81);
    query_buf.push(0x8A);
    query_buf.push(0x8B);
    query_buf.push(get_checksum(&query_buf));
    stream.write(&query_buf)?;

    let mut feedback_buf: [u8; 14] = [0; 14];
    stream.read(&mut feedback_buf)?;

    Ok(feedback_buf)
}

#[derive(Debug)]
pub struct MagicHomeState {
    pub is_enabled: bool,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
pub enum MagicHomeActionError {
    NotConnected,
    IoError(Error),
}

pub struct MagicHome {
    stream: Option<TcpStream>,
}

impl MagicHome {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn connect(&mut self, addr: &str) -> Result<(), Error> {
        let stream = TcpStream::connect(addr)?;
        let _ = get_state(&stream)?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn state(&mut self) -> Result<MagicHomeState, MagicHomeActionError> {
        let stream = self
            .stream
            .as_ref()
            .ok_or(MagicHomeActionError::NotConnected)?;

        let state = get_state(stream).map_err(|e| MagicHomeActionError::IoError(e))?;

        Ok(MagicHomeState {
            is_enabled: state[2] != 0x24,
            red: state[6],
            green: state[7],
            blue: state[8],
        })
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

    pub fn power(&mut self, value: bool) -> Result<(), MagicHomeActionError> {
        let mut stream = self
            .stream
            .as_ref()
            .ok_or(MagicHomeActionError::NotConnected)?;

        let power_byte = if value { 0x23 } else { 0x24 };
        let mut buf = vec![0x71, power_byte, 0x0F];
        buf.push(get_checksum(&buf));
        stream
            .write(&buf)
            .map_err(|e| MagicHomeActionError::IoError(e))?;

        Ok(())
    }
}
