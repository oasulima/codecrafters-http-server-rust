use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");

                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader
        .lines()
        .next()
        .expect("error: Empty request")
        .expect("error: couldn't parse request");

    let status_line = if request_line == "GET / HTTP/1.1" {
        "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 404 OK"
    };

    let response = format!("{status_line}\r\n\r\n");

    stream
        .write_all(response.as_bytes())
        .expect("error: couldn't write response");
}
