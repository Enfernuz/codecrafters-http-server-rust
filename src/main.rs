use std::collections::HashMap;
use std::net::TcpListener;
use std::{fs, thread};
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
struct Content {
    content_type: String,
    body: String,
}

#[derive(Debug)]
struct Response {
    http_version: String,
    status: String,
    headers: HashMap<String, String>,
    content: Option<Content>,
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
        let headers: String = self
            .headers
            .iter()
            .map(|(key, val)| format!("{}: {}", key, val))
            .collect::<Vec<String>>()
            .join("\r\n");
        let body: &str = if let Some(content) = &self.content {
            &format!("{}", &content.body)
        } else {
            ""
        };

        format!("{http_version} {status}\r\n{headers}\r\n\r\n{body}")
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
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
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let bytes_read = stream
        .read(&mut buf[..])
        .expect("Failed to read input stream.");
    if bytes_read > 0 {
        let req =
            Request::from_raw(&buf[..bytes_read]).expect("Failed to read request from raw input.");
        dbg!("{:#?}", &req);

        let status: String;
        let mut headers: HashMap<String, String> = HashMap::new();
        let content: Option<Content>;
        if req.path.eq("/") {
            status = String::from("200 OK");
            content = None;
        } else if req.path.eq("/user-agent") {
            status = String::from("200 OK");
            content = Some(Content {
                content_type: "text/plain".to_string(),
                body: req.headers.get("User-Agent").unwrap().to_string(),
            });
        } else if req.path.starts_with("/echo/") {
            status = String::from("200 OK");
            content = Some(Content {
                content_type: "text/plain".to_string(),
                body: req.path.trim_start_matches("/echo/").to_string(),
            });
        } else if req.path.starts_with("/files/") {
            let root_dir = std::env::args()
                .nth(2)
                .expect("Could not read the `--directory` flag value.");
            let filename = req.path.trim_start_matches("/files/");
            let path: String = root_dir + filename;
            let file_content = fs::read_to_string(&path);
            match file_content {
                Ok(_content) => {
                    status = String::from("200 OK");
                    content = Some(Content {
                        content_type: "application/octet-stream".to_string(),
                        body: _content,
                    })
                }
                Err(e) => {
                    dbg!("Error when reading file at {}: {:?}", &path, &e);
                    status = String::from("404 Not Found");
                    content = None;
                }
            }
        } else {
            status = String::from("404 Not Found");
            content = None;
        }

        if let Some(_content) = content.as_ref() {
            headers.insert("Content-Type".to_string(), _content.content_type.clone());
            headers.insert(
                "Content-Length".to_string(),
                _content.body.len().to_string(),
            );
        }

        let res: Response = Response {
            http_version: req.http_version.clone(),
            status: status,
            headers: headers,
            content: content,
        };
        dbg!("{:#?}", &res);

        stream
            .write(res.to_string().as_bytes())
            .expect("Failed to write to the incoming connection's stream.");
    }
}
