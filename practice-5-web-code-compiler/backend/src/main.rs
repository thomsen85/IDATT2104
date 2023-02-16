use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, thread, process::Command, fs,
};

use base64::{
    engine::general_purpose,
    Engine as _,
};
use network_common::{http::{Request, Method, Response}, websockets::SocketStream};
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
    println!("New Stream Incomming");
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Request = buf_reader
        .lines()
        .map(|result| result.unwrap_or("".to_string()))
        .map(|mut f| {f.push('\n'); f})
        .collect::<String>()
        .into();

    dbg!(&http_request);
    if let Some(upgrade) = http_request.headers.get("Upgrade") {
        if upgrade == "websocket" {
            handle_socket_connection(http_request, stream);
        } else {
            todo!("Other Upgrade Types not implemented yet");
        }
    } else {
        handle_http_request(http_request, stream);
    }
}

fn handle_socket_connection(request: Request, stream: TcpStream) {
    let mut stream = SocketStream::accept(request, stream).expect("Only implemented for websockets");
    let mut read_stream = stream.try_clone().expect("Could not clone reading stream");
    println!("Websocket Connection Established");
    thread::spawn(move || {
        loop {
            let message = read_stream.read_message_blocking().expect("Could not read message");
            println!("Message Recived: {:?}", message.payload);
            
        }
    });

    // Command::new("docker")
    //     .arg("exec")
    //     .arg("-it")
    //     .arg("rust_websockets")
    //     .arg("bash")
    //     .spawn()
    //     .expect("Could not spawn docker exec bash");

    blocking_counter(3);
    println!("Sending Message...");
    let message = SocketMessage::new("Hello :)".to_string());
    stream.send_message(&message).unwrap();
}

fn handle_http_request(request: Request, stream: TcpStream) {
    match request.method {
        Method::GET => todo!("Get not implemented yet"),
        Method::POST => handle_post_request(request, stream),
        _ => todo!("Other methods not implemented yet"),
    }
}

fn handle_post_request(request: Request, mut stream: TcpStream) {

    match request.path.as_str() {
        "/compile" => {
            
            println!("Request body: {}", &request.body);

            let response = Response::new();
            dbg!(&response.as_string());
            stream.write_all(&response.as_bytes()).unwrap();

        },
        _ => todo!("Other paths not implemented yet"),
    }
}

fn blocking_counter(secs: u64) {    
    for i in (1..=secs).rev() {
        println!("Blocking counter: {}", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}