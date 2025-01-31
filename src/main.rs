use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Error;
use std::net::TcpListener;
use std::{fs, thread};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

mod http;
use http::HttpMethod;

use crate::http::request::Request;
use crate::http::response::Content;
use crate::http::response::Response;
use crate::http::ApplicationContentType;
use crate::http::ContentType;
use crate::http::Status;
use crate::http::TextContentType;

const BUF_SIZE: usize = 1024;
const GZIP_ENCODING: &str = "gzip";

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

fn handle_request(req: &Request) -> Response {
    let mut status: Status;
    let mut content: Option<Content>;
    let request_path = req.get_path();
    if request_path.eq("/") {
        status = Status::Ok;
        content = None;
    } else if request_path.eq("/user-agent") {
        status = Status::Ok;
        content = Some(Content {
            content_type: ContentType::Text(TextContentType::Plain),
            body: req
                .get_headers()
                .get("User-Agent")
                .unwrap()
                .as_bytes()
                .to_vec(),
            encoding: None,
        });
    } else if request_path.starts_with("/echo/") {
        status = Status::Ok;
        content = Some(Content {
            content_type: ContentType::Text(TextContentType::Plain),
            body: request_path
                .trim_start_matches("/echo/")
                .as_bytes()
                .to_vec(),
            encoding: None,
        });
    } else if request_path.starts_with("/files/") {
        let filename = request_path.trim_start_matches("/files/");
        let file_path: String = get_file_root_dir()
            .map(|file_root_dir| file_root_dir + filename)
            .expect("Could not read the `--directory` flag value.");
        match req.get_method() {
            HttpMethod::Get => match read_file_content(&file_path) {
                Ok(_content) => {
                    status = Status::Ok;
                    content = Some(_content);
                }
                Err(err) => {
                    dbg!("Error when reading file at {}: {:?}", &file_path, &err);
                    status = Status::NotFound;
                    content = None;
                }
            },
            HttpMethod::Post => match File::create(&file_path) {
                Ok(mut file) => {
                    match req
                        .get_body()
                        .as_ref()
                        .map(|body| file.write(body.as_bytes()))
                    {
                        Some(Err(err)) => {
                            dbg!("Error when writing to file at {}: {:?}", &file_path, &err);
                            status = Status::InternalServerError;
                            content = None;
                        }
                        _ => {
                            status = Status::Created;
                            content = None;
                        }
                    }
                }
                Err(err) => {
                    dbg!("Error when creating file at {}: {:?}", &file_path, &err);
                    status = Status::InternalServerError;
                    content = None;
                }
            },
        }
    } else {
        status = Status::NotFound;
        content = None;
    }

    let accepted_encodings: HashSet<&str> = req
        .get_headers()
        .get("Accept-Encoding")
        .iter()
        .flat_map(|list| list.split(','))
        .map(str::trim)
        .collect::<HashSet<&str>>();

    if accepted_encodings.contains(GZIP_ENCODING) {
        if let Some(_content) = content.as_ref() {
            match gzip(_content.body.as_slice()) {
                Ok(payload) => {
                    content = content.map(|c| Content {
                        content_type: c.content_type,
                        body: payload,
                        encoding: Some(GZIP_ENCODING.to_owned()),
                    });
                }
                Err(err) => {
                    dbg!("Failed to Gzip the content: {}", err);
                    status = Status::InternalServerError;
                    content = None;
                }
            }
        }
    }

    let mut headers: HashMap<String, String> = HashMap::new();
    if let Some(_content) = content.as_ref() {
        headers.insert(
            "Content-Type".to_string(),
            _content.content_type.to_string(),
        );
        headers.insert(
            "Content-Length".to_string(),
            _content.body.len().to_string(),
        );
        if let Some(encoding) = _content.encoding.as_ref() {
            headers.insert("Content-Encoding".to_string(), encoding.clone());
        }
    }

    Response {
        http_version: req.get_http_version().to_owned(),
        status: status,
        headers: headers,
        content: content,
    }
}

fn handle_connection(mut stream: TcpStream) {
    let (bytes_read, buf) =
        read_data::<BUF_SIZE>(&mut stream).expect("Failed to read data from stream.");
    if bytes_read > 0 {
        let req =
            Request::from_raw(&buf[..bytes_read]).expect("Failed to read request from raw input.");
        let res = handle_request(&req);
        dbg!("Response: {}", res.to_string());
        stream
            .write(res.as_bytes().as_slice())
            .expect("Failed to write to the incoming connection's stream.");
    }
}

fn read_file_content(path: &str) -> Result<Content, Error> {
    fs::read_to_string(&path).map(|content| Content {
        content_type: ContentType::Application(ApplicationContentType::OctetStream),
        body: content.as_bytes().to_vec(),
        encoding: None, // TODO: set encoding according to the file's extension
    })
}

fn get_file_root_dir() -> Option<String> {
    std::env::args().nth(2)
}

fn gzip(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes)?;
    encoder.finish()
}
