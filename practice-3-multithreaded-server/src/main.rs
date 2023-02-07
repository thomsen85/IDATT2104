use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, fs,
};

mod thread_pool;
mod http;

use itertools::Itertools;
use thread_pool::Pool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7888").unwrap();
    let mut pool = Pool::new(3);
    pool.start();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.post(move || handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: String = buf_reader
        .lines()
        .map(|result| result.unwrap_or("".to_string()))
        .take_while(|line| !line.is_empty())
        .map(|mut f| {f.push('\n'); f})
        .collect();

    let http_request: http::Request = http_request.into();

    let mut status = "HTTP/1.1 200 OK";
    let mut result = 0;

    if http_request.path.starts_with("/add") {
        let (_, params) = http_request.path.split_at(5);
        result = params.split("&").map(|val| val.parse::<i32>().expect(&format!("Value {} is not a number", val))).sum();
        status = "HTTP/1.1 200 OK";
    } else if http_request.path.starts_with("/sub") {
        let (_, params) = http_request.path.split_at(5);
        let (a, b) = params.split("&").map(|val| val.parse::<i32>().unwrap()).next_tuple().unwrap();
        result = a - b;
        status = "HTTP/1.1 200 OK";
    } else {
        println!("Invalid path");
    }

    let contents = result.to_string();

    let html = fs::read_to_string("assets/index.html").unwrap();
    let html = html.replace("{{result}}", &contents);
    //let length = html.len();

    // \r\nContent-Length: {length}
    let headers = "Content-Type: text/html";

    let response =
        format!("{status}\r\n{headers}\r\n\r\n{html}");
    
    println!("Response: {:#?}", response);

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

