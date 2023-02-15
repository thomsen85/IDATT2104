use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, thread,
};

use base64::{
    engine::general_purpose,
    Engine as _,
};
use network_common::http::Request;
use network_common::websockets::SocketMessage;
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
        .map(|mut line| {
            line.push('\n');
            line
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

    let mut read_stream = stream.try_clone().expect("Could not clone reading stream");
    thread::spawn(move || {
        println!("Starting reading thread");
        loop {
            let mut buf = [0; 1024];
            println!("Waiting for message...");
            read_stream.read(&mut buf).unwrap();
            
            let message = SocketMessage::from_message(&buf);
            println!("Message: {:?}", message.payload);
            
        }
    });

    loop {
        blocking_counter(3);
        println!("Sending Message...");
        let send_message = SocketMessage::new("Hello :)".to_string());
        stream.write_all(&send_message.to_bytes()).unwrap();
        println!("Message sendt.");
    }
}

fn blocking_counter(secs: u64) {
    for i in (1..=secs).rev() {
        println!("Blocking counter: {}", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}