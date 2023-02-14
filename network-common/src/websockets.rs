use std::net::TcpListener;

use crate::bitbuilder::BitBuilder;

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
    pub fn new(payload: String) -> Self{
        Self {
            payload_len: payload.len() as u32,
            payload,
            ..Default::default()
        }
    }
    pub fn from_message(message: &[u8]) -> Self {
        let mut bitbuilder = BitBuilder::new();
        bitbuilder.append_bytes(message);

        let fin = bitbuilder.get_bit(0).unwrap();
        let rsv1 = bitbuilder.get_bit(1).unwrap();
        let rsv2 = bitbuilder.get_bit(2).unwrap();
        let rsv3 = bitbuilder.get_bit(3).unwrap();
        let opcode = bit_vec_to_u32(bitbuilder.get_bits(4..8).unwrap());
        let mask = bitbuilder.get_bit(8).unwrap();
        let payload_len = bit_vec_to_u32(bitbuilder.get_bits(9..16).unwrap());
        let mask_key = bitbuilder.get_bytes(2..=5).unwrap();

        let mut payload = String::new();
        let base = 6;
        let mut mask_i = 0;
        let mut message_i = 0;

        while message_i < (payload_len / 8) as usize {
            let unmasked = message[base + message_i] ^ mask_key[mask_i];
            payload.push(unmasked as char);
            message_i += 1;
            mask_i = (mask_i + 1) % 4;
        }

        Self {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            mask,
            payload_len,
            mask_key,
            payload,
        }
    }

    pub fn set_payload(&mut self, payload: String) {
        self.payload_len = payload.len() as u32;
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
        println!("Bitbuilder bits after some headers: {}", bitbuilder.len());

        bitbuilder.append_bits(&payload_len_to_bits(self.payload_len));
        println!("Bitbuilder bits after headers: {}", bitbuilder.len());

        bitbuilder.append_bytes(self.payload.as_bytes());

        println!("{}", bitbuilder.get_bitstring());
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
pub struct SocketConnection {
    tcp_listener: TcpListener,
}

fn bit_vec_to_u32(bit_vec: Vec<bool>) -> u32 {
    let mut res = 0;
    for (i, bit) in bit_vec.iter().enumerate() {
        if *bit {
            res += 2u32.pow(i as u32);
        }
    }
    res
}
