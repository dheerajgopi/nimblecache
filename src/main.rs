use std::{io::{Read, Write}, net::TcpListener};

fn main() {
    println!("Starting TCP listener on port 6379");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut buff = [0; 512];

                loop {
                    // read the stream and do nothing with it for now
                    let read_count = stream.read(&mut buff).unwrap();
                    if read_count == 0 {
                        break;
                    }
    
                    // hardcoded PONG response
                    stream.write(b"+PONG\r\n").unwrap();
                }

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
