use meval;
use std::io;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    let mut buf = vec![0; 1024];
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, addr);

        let s = std::str::from_utf8(&buf)
            .expect("Cant convert input to utf8")
            .split("\n")
            .nth(0)
            .unwrap();

        println!("Recived: {:?}", s);
        let mut ans = match meval::eval_str(s) {
            Ok(v) => v.to_string(),
            Err(e) => e.to_string(),
        };
        ans.push('\n');

        let len = sock.send_to(ans.as_bytes(), addr).await?;
        println!("{:?} bytes sent", len);
    }
}
