use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use network_common::bitbuilder::BitBuilder;
use network_common::http::Request;
use network_common::thread_pool::Pool;

use sha1::{Digest, Sha1};

const URL: &str = "127.0.0.1:7888";
const HASH_KEY: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

fn main() {
    let listener = TcpListener::bind(URL).unwrap();
    let mut pool = Pool::new(3);
    pool.start();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.post(move || handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Request = buf_reader
        .lines()
        .map(|result| result.unwrap_or("".to_string()))
        .take_while(|line| !line.is_empty())
        .map(|mut f| {
            f.push('\n');
            f
        })
        .collect::<String>()
        .into();

    println!("{}", "#".repeat(40));
    println!("{:?}", http_request);
    println!("{}", "-".repeat(40));

    // Hash Key
    let mut web_sock_key = http_request
        .headers
        .get("Sec-WebSocket-Key")
        .expect("No header Sec-WebSocket-Key")
        .to_owned();
    web_sock_key.push_str(HASH_KEY);

    let mut hasher = Sha1::new();
    hasher.update(web_sock_key.as_bytes());

    let hash = general_purpose::STANDARD.encode(hasher.finalize());

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r
Upgrade: websocket\r
Connection: Upgrade\r
Sec-WebSocket-Accept: {hash}\r
Sec-WebSocket-Protocol: chat\r\n
"
    );

    println!("{}", response);

    stream.write_all(response.as_bytes()).unwrap();

    loop {
        let mut buf = [0; 1024];
        println!("{:?}", stream.read(&mut buf));

        println!("{:?}", buf);
        let message = get_message(&buf);

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn get_message(message: &[u8]) -> String {
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

    println!(
        "fin: {}, rsv1: {}, rsv2: {}, rsv3: {}, opcode: {}, mask: {}, payload_len: {}",
        fin, rsv1, rsv2, rsv3, opcode, mask, payload_len,
    );


    let mut res = String::new();

    let base = 6;
    
    let mut mask_i = 0;
    let mut message_i = 0;

    while message_i < (payload_len/8) as usize {
        dbg!(message[base + message_i], mask_key[mask_i]);

        let unmasked = message[base + message_i] ^ mask_key[mask_i];
        res.push(unmasked as char);
        message_i += 1;
        mask_i = (mask_i + 1) % 4;
    }

    println!("Result: {}", res);
    res
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
