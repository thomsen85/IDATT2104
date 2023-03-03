use std::{net::{TcpListener, TcpStream}, io::{BufReader, BufRead}, fs, process::{Command, Stdio}, sync::{Mutex, Arc}, collections::VecDeque, thread};

use network_common::{thread_pool::Pool, http::Request, websockets::{SocketStream, SocketMessage}};
use serde::{Serialize, Deserialize};

const URL: &str = "127.0.0.1:7888";

fn main() {
    let listener = TcpListener::bind(URL).unwrap();
    let mut pool = Pool::new(10);
    let client_queues: Arc<Mutex<Vec<(u32, VecDeque<DrawInstruction>)>>> = Arc::new(Mutex::new(Vec::new()));

    pool.start();
    let mut id = 0;
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        client_queues.lock().unwrap().push((id, VecDeque::new()));
        let client_queues = client_queues.clone();
        pool.post(move || handle_connection(id, stream, client_queues));
        id += 1;
    }
}

fn handle_connection(id: u32, mut stream: TcpStream, client_queues: Arc<Mutex<Vec<(u32, VecDeque<DrawInstruction>)>>>) {
    let mut buf_reader = BufReader::new(&mut stream);

    let received: Vec<u8> = buf_reader.fill_buf().unwrap().to_vec();
    buf_reader.consume(received.len());

    let http_request = Request::from(String::from_utf8(received).unwrap());

    dbg!(&http_request);
    if let Some(upgrade) = http_request.headers.get("Upgrade") {
        if upgrade == "websocket" {
            handle_socket_connection(http_request, stream, id, client_queues);
        } else {
            todo!("Other Upgrade Types not implemented yet");
        }
    } 
    // } else {
    //     //todo!("Non-Upgrade Requests not implemented yet")
    // }
}

fn handle_socket_connection(request: Request, stream: TcpStream, id: u32, client_queues: Arc<Mutex<Vec<(u32, VecDeque<DrawInstruction>)>>>) {
    let mut stream =
        SocketStream::accept(request, stream).expect("Only implemented for websockets");

    let mut write_stream = stream.try_clone().expect("Could not clone reading stream");
    
    println!("Websocket Connection Established");

    // Writing Thread
    let write_client_queues = client_queues.clone();
    thread::spawn(move || {
        loop {
            // Cheap solution, should be replaced with a condition variable.
            thread::sleep(std::time::Duration::from_millis(50));
    
            for (client_id, queue) in write_client_queues.lock().unwrap().iter_mut() {
                if *client_id == id {
                    while let Some(draw_instruction) = queue.pop_front() {
                        let message = SocketMessage::new(serde_json::to_string(&draw_instruction).unwrap());
                        if write_stream.send_message(&message).is_err() {

                            break;
                        }
                    }
                }
            }
        }
    });

    loop {
        let message = match stream
            .read_message_blocking() {
                Ok(message) => message,
                Err(_) => break,
            };
        println!("Message Recived: {:?}", message.payload);
        let draw_instruction: DrawInstruction = serde_json::from_str(&message.payload).unwrap();
        client_queues.lock().unwrap().iter_mut().for_each(|(client_id, queue)| {
            if *client_id != id {
                queue.push_back(draw_instruction);
            }
        });
        if message.payload == "stop" {
            break;
        }
    }

    let index = client_queues.lock().unwrap().iter().position(|(client_id, _)| *client_id == id).unwrap();
    client_queues.lock().unwrap().remove(index);
    if stream.close().is_err() {
        println!("Stream not closed Properly");
    }
    println!("Websocket Connection Closed");
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct DrawInstruction {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}