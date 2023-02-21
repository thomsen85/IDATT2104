use std::{
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread, fs, process::{Stdio, Command},
};

use network_common::thread_pool::Pool;
use network_common::websockets::SocketMessage;
use network_common::{
    http::{Method, Request, Response},
    websockets::SocketStream,
};
use serde::Deserialize;

const URL: &str = "127.0.0.1:7888";

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
    let mut buf_reader = BufReader::new(&mut stream);
    println!("Reading Incomming");

    let received: Vec<u8> = buf_reader.fill_buf().unwrap().to_vec();
    buf_reader.consume(received.len());

    let http_request = Request::from(String::from_utf8(received).unwrap());

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
    let mut stream =
        SocketStream::accept(request, stream).expect("Only implemented for websockets");
        
    let mut read_stream = stream.try_clone().expect("Could not clone reading stream");
    println!("Websocket Connection Established");
    let message = read_stream
        .read_message_blocking()
        .expect("Could not read message");
    println!("Message Recived: {:?}", message.payload);

    let code = fs::read_to_string(format!("programs/p_{}.rs", message.payload)).unwrap();

    let mut output = Command::new("docker")
            .arg("run")
            .arg("-t")
            .arg("--rm")
            .arg("rust:latest")
            .arg("bash")
            .arg("-c")
            .arg(format!("cargo new program && cd program && printf '{}' > src/main.rs && cargo run", code))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute docker run command");

        {
            let stdout = output.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();
    
            for line in stdout_lines {
                let message = SocketMessage::new(line.unwrap());
                stream.send_message(&message).unwrap();

            }
        }

        output.wait().unwrap();
        println!("Program Executed");
        stream.close();

}

fn handle_http_request(request: Request, stream: TcpStream) {
    match request.method {
        Method::GET => todo!("Get not implemented yet"),
        Method::POST => handle_post_request(request, stream),
        _ => todo!("Other methods not implemented yet"),
    }
}

#[derive(Debug, Deserialize)]
struct Body {
    id: String,
    code: String
}

fn handle_post_request(request: Request, mut stream: TcpStream) {
    match request.path.as_str() {
        "/compile" => {
            let body: Body = serde_json::from_str(&request.body).unwrap();
            let mut file = std::fs::File::create(&format!("programs/p_{}.rs", body.id)).unwrap();
            file.write_all(body.code.as_bytes()).unwrap();

            let mut response = Response::new();
            response.headers.insert("Access-Control-Allow-Origin".to_owned(), "http://localhost:5174".to_owned());
            stream.write_all(&response.as_bytes()).unwrap();
            println!("Response sendt");
        }
        _ => todo!("Other paths not implemented yet"),
    }
}


#[cfg(test)]
mod tests {
    use std::{process::Stdio, io::{BufReader, BufRead}};

    #[test]
    fn test_docker_run_command() {
        use std::process::Command;
        let code = "fn main() {
            println!(\"Hello Program\");
        }";

        let mut output = Command::new("docker")
            .arg("run")
            .arg("-t")
            .arg("--rm")
            .arg("rust:latest")
            .arg("bash")
            .arg("-c")
            .arg(format!("cargo new program && cd program && printf '{}' > src/main.rs && cargo run", code))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute docker run command");

        {
            let stdout = output.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();
    
            for line in stdout_lines {
                println!("Read: {}", line.unwrap());
            }
        }

        output.wait().unwrap();
    }

}