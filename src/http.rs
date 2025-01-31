pub mod request {

    use std::collections::HashMap;

    use super::HttpMethod;

    #[derive(Debug, Default)]
    pub struct Request {
        method: HttpMethod,
        path: String,
        http_version: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    }

    impl Request {
        pub fn get_method(&'_ self) -> &'_ HttpMethod {
            &self.method
        }

        pub fn get_path(&'_ self) -> &'_ str {
            &self.path
        }

        pub fn get_http_version(&'_ self) -> &'_ str {
            &self.http_version
        }

        pub fn get_headers(&'_ self) -> &'_ HashMap<String, String> {
            &self.headers
        }

        pub fn get_body(&'_ self) -> &'_ Option<String> {
            &self.body
        }

        pub fn from_raw(input: &[u8]) -> Result<Self, String> {
            let raw = String::from_utf8_lossy(&input).into_owned();
            let lines: Vec<&str> = raw.split("\r\n").collect();

            // Parse request line
            let request_line = lines.first().ok_or("Invalid request: request is empty")?;
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() != 3 {
                return Err("Malformed request: Invalid request line: {}".to_string());
            }

            let method: HttpMethod = HttpMethod::from_string(parts[0]);
            let path: &str = parts[1];
            let http_version: &str = parts[2];
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
                        headers.insert(key.to_owned(), value.to_owned());
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
                path: path.to_owned(),
                http_version: http_version.to_owned(),
                headers,
                body,
            })
        }
    }
}

pub mod response {

    use std::collections::HashMap;

    use super::ContentType;

    #[derive(Debug)]
    pub struct Content {
        pub content_type: ContentType,
        pub body: String,
    }

    #[derive(Debug)]
    pub struct Response {
        pub http_version: String,
        pub status: super::Status,
        pub headers: HashMap<String, String>,
        pub content: Option<Content>,
    }

    impl Response {
        //HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc
        pub fn to_string(&self) -> String {
            let http_version = &self.http_version;
            let status = &self.status.to_string();
            let headers = &self
                .headers
                .iter()
                .map(|(key, val)| format!("{}: {}", key, val))
                .collect::<Vec<String>>()
                .join("\r\n");
            let body = if let Some(content) = &self.content {
                &format!("{}", &content.body)
            } else {
                ""
            };

            format!("{http_version} {status}\r\n{headers}\r\n\r\n{body}")
        }
    }
}

#[derive(Debug, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
}

#[derive(Debug)]
pub enum Status {
    Ok,
    Created,
    NotFound,
    InternalServerError,
}

impl HttpMethod {
    pub fn to_string(&'_ self) -> &'_ str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }

    pub fn from_string(string: &str) -> HttpMethod {
        match string {
            "GET" => Self::Get,
            "POST" => Self::Post,
            _ => panic!("Unable to parse HTTP method from {}", string),
        }
    }
}

impl Status {
    pub fn get_status_code(&self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::Created => 201,
            Self::NotFound => 404,
            Self::InternalServerError => 500,
        }
    }

    pub fn get_text_code(&'_ self) -> &'_ str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::NotFound => "Not Found",
            Self::InternalServerError => "Internal Server Error",
        }
    }

    pub fn to_string(&self) -> String {
        format!("{} {}", self.get_status_code(), self.get_text_code())
    }
}

#[derive(Debug)]
pub enum TextContentType {
    Plain,
}

#[derive(Debug)]
pub enum ApplicationContentType {
    OctetStream,
}

#[derive(Debug)]
pub enum ContentType {
    Text(TextContentType),
    Application(ApplicationContentType),
}

impl TextContentType {
    fn to_string(&self) -> &str {
        match self {
            Self::Plain => "plain",
        }
    }
}

impl ApplicationContentType {
    fn to_string(&self) -> &str {
        match self {
            Self::OctetStream => "octet-stream",
        }
    }
}

impl ContentType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Text(sub_type) => format!("text/{}", sub_type.to_string()),
            Self::Application(sub_type) => format!("application/{}", sub_type.to_string()),
        }
    }
}
