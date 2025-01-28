use std::net::TcpListener;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

const BUF_SIZE: usize = 1024;

#[derive(Debug, Default)]
struct Request {
    method: String,
    path: String,
    http_version: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl Request {
    fn from_raw(input: &[u8]) -> Result<Self, String> {
        let raw = String::from_utf8_lossy(&input).into_owned();
        let lines: Vec<&str> = raw.split("\r\n").collect();

        // Parse request line
        let request_line = lines.first().ok_or("Invalid request: request is empty")?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err("Malformed request: Invalid request line: {}".to_string());
        }
        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let http_version = parts[2].to_string();
        // Parse headers
        let mut headers = Vec::new();
        let mut body_start = 0;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            match line.split_once(": ") {
                Some((key, value)) => headers.push((key.to_string(), value.to_string())),
                _ => return Err(format!("Malformed header: {}", line)),
            }
        }
        // Parse body
        let body = if body_start < lines.len() {
            Some(lines[body_start..].join("\r\n"))
        } else {
            None
        };
        Ok(Self {
            method,
            path,
            http_version,
            headers,
            body,
        })
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_connection(&mut _stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let bytes_read = stream
        .read(&mut buf[..])
        .expect("Failed to read input stream.");
    if bytes_read > 0 {
        let req =
            Request::from_raw(&buf[..bytes_read]).expect("Failed to read request from raw input.");
        let status = match req.path.as_str() {
            "/" => "200 OK",
            _ => "404 Not Found",
        };
        stream
            .write(format!("HTTP/1.1 {status}\r\n\r\n").as_bytes())
            .expect("Failed to write to the incoming connection's stream.");
    }
}
