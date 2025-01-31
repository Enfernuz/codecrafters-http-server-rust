use std::collections::HashMap;
use std::io::Error;
use std::net::TcpListener;
use std::{fs, thread};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

mod http;

use crate::http::request::Request;
use crate::http::response::Content;
use crate::http::response::Response;
use crate::http::Status;

const BUF_SIZE: usize = 1024;

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

fn read_data<const N: usize>(stream: &mut TcpStream) -> Result<(usize, [u8; N]), Error> {
    let mut buf: [u8; N] = [0; N];
    let result = stream.read(&mut buf[..]);
    match result {
        Ok(bytes_read) => Ok((bytes_read, buf)),
        Err(err) => Err(err),
    }
}

fn handle_connection(mut stream: TcpStream) {
    let (bytes_read, buf) =
        read_data::<BUF_SIZE>(&mut stream).expect("Failed to read data from stream.");
    if bytes_read > 0 {
        let req =
            Request::from_raw(&buf[..bytes_read]).expect("Failed to read request from raw input.");
        dbg!("{:#?}", &req);

        let status: Status;
        let content: Option<Content>;
        let request_path = req.get_path();
        if request_path.eq("/") {
            status = Status::Ok;
            content = None;
        } else if request_path.eq("/user-agent") {
            status = Status::Ok;
            content = Some(Content {
                content_type: "text/plain".to_string(),
                body: req.get_headers().get("User-Agent").unwrap().to_owned(),
            });
        } else if request_path.starts_with("/echo/") {
            status = Status::Ok;
            content = Some(Content {
                content_type: "text/plain".to_string(),
                body: request_path.trim_start_matches("/echo/").to_string(),
            });
        } else if request_path.starts_with("/files/") {
            let root_dir = std::env::args()
                .nth(2)
                .expect("Could not read the `--directory` flag value.");
            let filename = request_path.trim_start_matches("/files/");
            let path: String = root_dir + filename;
            let file_content = fs::read_to_string(&path);
            match file_content {
                Ok(_content) => {
                    status = Status::Ok;
                    content = Some(Content {
                        content_type: "application/octet-stream".to_string(),
                        body: _content,
                    })
                }
                Err(e) => {
                    dbg!("Error when reading file at {}: {:?}", &path, &e);
                    status = Status::NotFound;
                    content = None;
                }
            }
        } else {
            status = Status::NotFound;
            content = None;
        }

        let mut headers: HashMap<String, String> = HashMap::new();
        if let Some(_content) = content.as_ref() {
            headers.insert("Content-Type".to_string(), _content.content_type.clone());
            headers.insert(
                "Content-Length".to_string(),
                _content.body.len().to_string(),
            );
        }

        let res: Response = Response {
            http_version: req.get_http_version().to_owned(),
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
