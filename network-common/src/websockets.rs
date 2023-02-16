use std::{net::{TcpListener, TcpStream, Shutdown}, io::{Read, BufReader, BufRead, Write, self}};

use crate::{bitbuilder::BitBuilder, http::Request};
use base64::{
    self,
    Engine as _,
};
use sha1::{Digest, Sha1};
const HASH_KEY: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
pub struct SocketMessage {
    fin: bool,
    rsv1: bool,
    rsv2: bool,
    rsv3: bool,
    opcode: u32,
    mask: bool,
    payload_len: u32,
    mask_key: Vec<u8>,
    pub payload: String
}

impl SocketMessage {
    pub fn new(payload: String) -> Self {
        Self {
            payload_len: payload.len() as u32 + 1, // +1 for the null byte
            payload,
            ..Default::default()
        }
    }
    pub fn from_message(message: &[u8]) -> io::Result<Self> {
        let mut bitbuilder = BitBuilder::new();
        bitbuilder.append_bytes(message);

        let fin = bitbuilder.get_bit(0).unwrap();
        let rsv1 = bitbuilder.get_bit(1).unwrap();
        let rsv2 = bitbuilder.get_bit(2).unwrap();
        let rsv3 = bitbuilder.get_bit(3).unwrap();
        let opcode = bit_vec_to_u32(&bitbuilder.get_bits(4..8).unwrap());
        let mask = bitbuilder.get_bit(8).unwrap();
        let payload_len = bit_vec_to_u32(&bitbuilder.get_bits(9..16).unwrap());

        if payload_len == 126 {
            todo!("Extended payload length not yet implementeted");
        } else if payload_len == 127 {
            todo!("Extended payload length not yet implementeted");
        }

        let mask_key = bitbuilder.get_bytes(2..=5).unwrap();

        let mut payload = String::new();
        let base = 6;
        let mut mask_i = 0;
        let mut message_i = 0;

        while message_i < payload_len as usize {
            let unmasked = message[base + message_i] ^ mask_key[mask_i];
            payload.push(unmasked as char);
            message_i += 1;
            mask_i = (mask_i + 1) % 4;
        }

        Ok(Self {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            mask,
            payload_len,
            mask_key,
            payload,
        })
    }

    pub fn set_payload(&mut self, payload: String) {
        self.payload_len = payload.len() as u32 + 1; // +1 for the null byte
        self.payload = payload;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bitbuilder = BitBuilder::new();

        bitbuilder.push_bit(self.fin);
        bitbuilder.push_bit(self.rsv1);
        bitbuilder.push_bit(self.rsv2);
        bitbuilder.push_bit(self.rsv3);

        bitbuilder.push_bit(self.opcode & 8 == 8);
        bitbuilder.push_bit(self.opcode & 4 == 4);
        bitbuilder.push_bit(self.opcode & 2 == 2);
        bitbuilder.push_bit(self.opcode & 1 == 1);

        bitbuilder.push_bit(self.mask);

        bitbuilder.append_bits(&payload_len_to_bits(self.payload_len));

        bitbuilder.append_bytes(self.payload.as_bytes());

        bitbuilder.as_bytes().to_vec()
    }
}

fn payload_len_to_bits(payload_len: u32) -> Vec<bool> {
    let mut result = Vec::new();
    for i in 0..7 {
        result.push(payload_len & 2u32.pow(i) == 2u32.pow(i));
    }
    result.reverse();
    result
}

fn bit_vec_to_u32(bit_vec: &Vec<bool>) -> u32 {
    let mut res = 0;
    for (i, bit) in bit_vec.iter().rev().enumerate() {
        if *bit {
            res += 2u32.pow(i as u32);
        }
    }
    res
}

impl Default for SocketMessage {
    fn default() -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: 1,
            mask: false,
            payload_len: 0,
            mask_key: vec![],
            payload: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct SocketStream {
    tcp_stream: TcpStream,
}


impl SocketStream {
    pub fn accept(http_request: Request, mut tcp_stream: TcpStream) -> Result<Self, std::io::Error> {
        if let Some(upgrade) = http_request.headers.get("Upgrade") {
            if upgrade != "websocket" {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Not a websocket request"));
            }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Not a websocket request"));
        }


        let accept_response = Self::_get_accept_response(&http_request);
        tcp_stream.write_all(accept_response.as_bytes())?;

        Ok(Self {
            tcp_stream
        })
    }

    fn _get_accept_response(http_request: &Request) -> String {
        let mut web_sock_key = http_request
        .headers
        .get("Sec-WebSocket-Key")
        .expect("No header Sec-WebSocket-Key")
        .to_owned();
    web_sock_key.push_str(HASH_KEY);

    let mut hasher = Sha1::new();
    hasher.update(web_sock_key.as_bytes());

    let hash = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

    format!("\
        HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {hash}\r\n\
        Sec-WebSocket-Protocol: chat\r\n\n"
    )
}

    pub fn send_message(&mut self, message: &SocketMessage) -> io::Result<()> {
        self.tcp_stream.write_all(&message.to_bytes())
    }

    pub fn read_message_blocking(&mut self) -> io::Result<SocketMessage> {
        let buf = &mut [0; 1024];
        self.tcp_stream.read(buf)?;
        let msg = SocketMessage::from_message(buf)?;

        if msg.opcode == 8 {
            self.tcp_stream.shutdown(Shutdown::Both)?;
        }
        Ok(msg)
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            tcp_stream: self.tcp_stream.try_clone()?
        })
    }
}


