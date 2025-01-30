use std::collections::HashMap;
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
    headers: HashMap<String, String>,
    body: Option<String>,
}

#[derive(Debug)]
struct Response {
    http_version: String,
    status: String,
    content_type: String,
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
        let mut headers = HashMap::new();
        let mut body_start = 0;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            match line.split_once(": ") {
                Some((key, value)) => {
                    headers.insert(key.to_string(), value.to_string());
                }
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

impl Response {
    //HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc
    fn to_string(&self) -> String {
        let http_version = &self.http_version;
        let status = &self.status;
        let content_type = &self.content_type;
        let body = if let Some(_body) = &self.body {
            _body
        } else {
            ""
        };
        let content_length: u64 = body.len() as u64;

        format!("{http_version} {status}\r\nContent-type: {content_type}\r\nContent-Length: {content_length}\r\n\r\n{body}")
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
        dbg!("{:#?}", &req);

        let status: String;
        let body: Option<String>;
        if req.path.eq("/") {
            status = String::from("200 OK");
            body = None;
        } else if req.path.eq("/user-agent") {
            status = String::from("200 OK");
            body = req.headers.get("User-Agent").cloned();
        } else if req.path.starts_with("/echo/") {
            status = String::from("200 OK");
            body = Some(req.path.trim_start_matches("/echo/").to_string());
        } else {
            status = String::from("404 Not Found");
            body = None;
        }
        let res: Response = Response {
            http_version: req.http_version.clone(),
            status: status,
            content_type: String::from("text/plain"),
            body: body,
        };
        dbg!("{:#?}", &res);

        stream
            .write(res.to_string().as_bytes())
            .expect("Failed to write to the incoming connection's stream.");
    }
}
