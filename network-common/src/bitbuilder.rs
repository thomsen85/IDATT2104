

#[derive(Debug, Clone)]
pub struct BitBuilder {
    bytes: Vec<u8>,
    bit_pos: u8,
}

impl BitBuilder {
    pub fn new() -> Self {
        Self {
            bytes: vec![0],
            bit_pos: 0,
        }
    }

    pub fn get_bit(&self, index: usize) -> Option<bool> {
        let byte_index = index / 8;
        if byte_index >= self.bytes.len() {
            return None;
        }
        if byte_index + 1 == self.bytes.len() && (index % 8) as u8 >= self.bit_pos {
            return None;
        }

        let byte = self.bytes[byte_index];
        let bit = (byte >> (7 - (index % 8))) & 1;
        Some(bit == 1)
    }

    pub fn get_bits<T>(&self, range: T) -> Option<Vec<bool>>
    where
        T: IntoIterator<Item = usize>,
    {
        let mut result = Vec::new();
        for i in range {
            if let Some(bit) = self.get_bit(i) {
                result.push(bit);
            } else {
                return None;
            }
        }
        Some(result)
    }

    pub fn get_bytes<T>(&self, range: T) -> Option<Vec<u8>>
    where
        T: IntoIterator<Item = usize>,
    {
        let mut result = Vec::new();
        for i in range {
            if let Some(byte) = self.get_byte(i) {
                result.push(byte);
            } else {
                return None;
            }
        }
        Some(result)
    }

    pub fn get_byte(&self, index: usize) -> Option<u8> {
        let byte_index = index;
        if byte_index >= self.bytes.len() {
            return None;
        }

        let byte = self.bytes[byte_index];
        Some(byte)
    }

    pub fn from_bit_string(input: &str) -> Self {
        let mut builder = Self::new();
        for c in input.chars() {
            if c == '1' {
                builder.push_bit(true);
            } else if c == '0' {
                builder.push_bit(false);
            }
        }
        builder
    }

    pub fn push_bit(&mut self, bit: bool) {
        if self.bit_pos >= 8 {
            self.bit_pos = 0;
            self.bytes.push(0);
        }

        if bit {
            *self.bytes.last_mut().unwrap() |= 1 << (7 - self.bit_pos)
        }

        self.bit_pos += 1
    }

    pub fn append_bits(&mut self, bits: &Vec<bool>) {
        for bit in bits {
            self.push_bit(*bit);
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.bytes.push(0);
        }
        // Adding to last
        *self.bytes.last_mut().unwrap() |= byte >> self.bit_pos;
        if self.bit_pos == 0 {
            self.bytes.push(0);
        } else {
            self.bytes.push(byte << (8 - self.bit_pos));
        }
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.push_byte(*byte);
        }
    }

    pub fn push_u32(&mut self, input: u32) {
        let b1: u8 = ((input >> 24) & 0xff) as u8;
        let b2: u8 = ((input >> 16) & 0xff) as u8;
        let b3: u8 = ((input >> 8) & 0xff) as u8;
        let b4: u8 = (input & 0xff) as u8;

        self.push_byte(b1);
        self.push_byte(b2);
        self.push_byte(b3);
        self.push_byte(b4);
    }

    pub fn get_bitstring(&self) -> String {
        let mut result = String::new();

        for i in 0..self.len() {
            if let Some(b) = self.get_bit(i) {
                if i % 4 == 0 {
                    result.push(' ');
                }
                if b {
                    result.push('1');
                } else {
                    result.push('0');
                }
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.bytes.len() * 8 - (8 - self.bit_pos as usize)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
