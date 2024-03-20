use clap::Parser;
use regex::Regex;
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    directory: String,
}

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
    // let request_lines = buf_reader.lines();

    let (status_line, content_type, content) = parse_request_line(buf_reader);

    let length = content.len();

    let response = format!(
        "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\n\r\n{content}"
    );

    stream
        .write_all(response.as_bytes())
        .expect("error: couldn't write response");
}

fn parse_request_line(
    mut buf_reader: BufReader<&mut TcpStream>,
) -> (&'static str, &'static str, String) {
    let mut request_line = String::new();
    buf_reader
        .read_line(&mut request_line)
        .expect("error: couldn't read request line");

    let re =
        Regex::new(r"(?<method>\S+) (?<path>\S+) HTTP/1.1").expect("error: Couldn't compile regex");

    if re.is_match(&request_line) {
        let (_, [method, path]) = re
            .captures(&request_line)
            .expect("error: Couldn't get path")
            .extract();

        if path == "/" {
            return ("HTTP/1.1 200 OK", "text/plain", "".to_string());
        }
        if path == "/user-agent" {
            loop {
                let mut request_line = String::new();
                let r = buf_reader
                    .read_line(&mut request_line)
                    .expect("error: couldn't read request line");
                if r < 3 {
                    //detect empty line
                    break;
                }
                if request_line.starts_with("User-Agent:") {
                    return (
                        "HTTP/1.1 200 OK",
                        "text/plain",
                        (&request_line[12..]).trim().to_string(),
                    );
                }
            }
        }
        if path.starts_with("/echo/") {
            return ("HTTP/1.1 200 OK", "text/plain", (&path[6..]).to_string());
        }
        if path.starts_with("/files/") {
            let args = Args::parse();

            let dir = args.directory;
            let file_name = &path[7..];

            let file_path = format!("{dir}/{file_name}");

            match method {
                "GET" => {
                    let file = fs::read_to_string(file_path);
                    if file.is_ok() {
                        return ("HTTP/1.1 200 OK", "application/octet-stream", file.unwrap());
                    }
                }
                "POST" => {
                    let mut content_length = 0;
                    loop {
                        let mut request_line = String::new();
                        let r = buf_reader
                            .read_line(&mut request_line)
                            .expect("error: couldn't read request line");
                        if r < 3 {
                            //detect empty line
                            break;
                        }
                        if request_line.starts_with("Content-Length:") {
                            content_length = (&request_line[15..])
                                .trim()
                                .parse::<usize>()
                                .expect("error: couldn't parse content-length");
                        }
                    }

                    let mut file_content = vec![0; content_length]; //New Vector with size of Content
                    buf_reader
                        .read_exact(&mut file_content)
                        .expect("error: couldn't read body");

                    fs::write(file_path, file_content).expect("error: couldn't write the file");
                    return ("HTTP/1.1 201 OK", "text/plain", "".to_string());
                }
                &_ => {}
            }
        }
    }

    ("HTTP/1.1 404 OK", "text/plain", "".to_string())
}
