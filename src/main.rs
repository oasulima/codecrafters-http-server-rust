use regex::Regex;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");

                thread::spawn(|| {
                    handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_lines = buf_reader.lines();

    let (status_line, content) = parse_request_line(request_lines);

    println!("content: {content}");

    let length = content.len();

    let response = format!(
        "{status_line}\r\nContent-Type: text/plain\r\nContent-Length: {length}\r\n\r\n{content}"
    );

    stream
        .write_all(response.as_bytes())
        .expect("error: couldn't write response");
}

fn parse_request_line(
    mut request_lines: std::io::Lines<BufReader<&mut TcpStream>>,
) -> (&'static str, String) {
    let request_line = request_lines
        .next()
        .expect("error: Empty request")
        .expect("error: couldn't parse request line");

    let re = Regex::new(r"GET /(?<path>\S*) HTTP/1.1").expect("error: Couldn't compile regex");

    if re.is_match(&request_line) {
        let (_, [path]) = re
            .captures(&request_line)
            .expect("error: Couldn't get path")
            .extract();

        println!("path: {path}");

        if path.is_empty() {
            return ("HTTP/1.1 200 OK", "".to_string());
        }
        if path == "user-agent" {
            while let Some(request_line) = request_lines.next() {
                let request_line = request_line.expect("error: couldn't parse request line");
                if request_line.starts_with("User-Agent:") {
                    return ("HTTP/1.1 200 OK", (&request_line[12..]).to_string());
                }
            }
        }
        if path.starts_with("echo/") {
            return ("HTTP/1.1 200 OK", (&path[5..]).to_string());
        }
    }

    ("HTTP/1.1 404 OK", "".to_string())
}
