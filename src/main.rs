// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                thread::spawn(move || handle_connection(_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // Create a mutable buffer of fixed size
    let bytes_read = stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let request_string = String::from_utf8_lossy(&buffer[..bytes_read]).to_string(); // Convert the buffer to a string
    println!("{} {}", bytes_read, request_string);

    let request = HttpRequest::from_str(&request_string).unwrap();
    let (response_status, response_body) = match request.target {
        HttpTarget::Root => (HttpStatus::Ok, "Hello, World!".to_string()),
        HttpTarget::Echo(s) => (HttpStatus::Ok, s),
        HttpTarget::UserAgent => (HttpStatus::Ok, request.user_agent.unwrap_or_default()),
        HttpTarget::NotFound => (HttpStatus::NotFound, String::new()),
    };
    let response = HttpResponse {
        version: HttpVersion::Http11,
        status: response_status,
        content_type: HttpContentType::TextPlain,
        content_length: response_body.len(),
        body: response_body,
    };
    stream
        .write(response.to_string().as_bytes())
        .expect("Failed to write response buffer");
}

#[allow(dead_code)]
struct HttpRequest {
    method: HttpMethod,
    target: HttpTarget,
    version: HttpVersion,
    host: Option<String>,
    user_agent: Option<String>,
    accept: Option<String>,
    accept_encoding: Vec<HttpEncoding>,
}

impl HttpRequest {
    fn new(method: HttpMethod, target: HttpTarget, version: HttpVersion) -> Self {
        HttpRequest {
            method,
            target,
            version,
            host: None,
            user_agent: None,
            accept: None,
            accept_encoding: Vec::new(),
        }
    }
}

impl FromStr for HttpRequest {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut request_lines = s.lines();
        let mut start_line = match request_lines.next() {
            Some(line) => line.split_whitespace(),
            None => return Err(()),
        };
        let method = parse_method(start_line.next())?;
        let target = parse_target(start_line.next())?;
        let version = parse_version(start_line.next())?;
        let mut request = HttpRequest::new(method, target, version);
        while let Some(line) = request_lines.next() {
            match line {
                line if line.to_lowercase().starts_with("host: ") => {
                    request.host = parse_host(line).ok()
                }
                line if line.to_lowercase().starts_with("user-agent: ") => {
                    request.user_agent = parse_user_agent(line).ok()
                }
                line if line.to_lowercase().starts_with("accept: ") => {
                    request.accept = parse_accept(line).ok()
                }
                line if line.to_lowercase().starts_with("accept-encoding: ") => {
                    request.accept_encoding = parse_accept_encoding(line).ok().unwrap_or(Vec::new())
                }
                "" => break,
                _ => continue,
            }
        }

        Ok(request)
    }
}

struct HttpResponse {
    version: HttpVersion,
    status: HttpStatus,
    content_type: HttpContentType,
    content_length: usize,
    body: String,
}

impl ToString for HttpResponse {
    fn to_string(&self) -> String {
        format!(
            "{} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            self.version.as_ref(),
            self.status.as_ref(),
            self.content_type.to_string(),
            self.content_length,
            self.body
        )
    }
}

enum HttpMethod {
    Get,
}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
        }
    }
}

enum HttpTarget {
    Root,
    Echo(String),
    UserAgent,
    NotFound,
}

enum HttpEncoding {
    Gzip,
    Compress,
    Deflate,
    Br,
}

impl AsRef<str> for HttpEncoding {
    fn as_ref(&self) -> &str {
        match self {
            HttpEncoding::Gzip => "gzip",
            HttpEncoding::Compress => "compress",
            HttpEncoding::Deflate => "deflate",
            HttpEncoding::Br => "br",
        }
    }
}

enum HttpVersion {
    Http11,
}

impl AsRef<str> for HttpVersion {
    fn as_ref(&self) -> &str {
        match self {
            HttpVersion::Http11 => "HTTP/1.1",
        }
    }
}

enum HttpStatus {
    Ok = 200,
    NotFound = 404,
}

impl AsRef<str> for HttpStatus {
    fn as_ref(&self) -> &str {
        match self {
            HttpStatus::Ok => "200 OK",
            HttpStatus::NotFound => "404 NOT FOUND",
        }
    }
}

enum HttpContentType {
    TextPlain,
}

impl ToString for HttpContentType {
    fn to_string(&self) -> String {
        match self {
            HttpContentType::TextPlain => "text/plain".to_string(),
        }
    }
}

fn parse_method(s: Option<&str>) -> Result<HttpMethod, ()> {
    match s {
        Some("GET") => Ok(HttpMethod::Get),
        _ => Err(()),
    }
}

fn parse_target(s: Option<&str>) -> Result<HttpTarget, ()> {
    match s {
        Some("/") => Ok(HttpTarget::Root),
        Some(s) if s.starts_with("/echo/") => Ok(HttpTarget::Echo(
            s.strip_prefix("/echo/").unwrap().to_string(),
        )),
        Some("/user-agent") => Ok(HttpTarget::UserAgent),
        _ => Ok(HttpTarget::NotFound),
    }
}

fn parse_version(s: Option<&str>) -> Result<HttpVersion, ()> {
    match s {
        Some("HTTP/1.1") => Ok(HttpVersion::Http11),
        _ => Err(()),
    }
}

fn parse_host(s: &str) -> Result<String, ()> {
    match s {
        s if s.to_lowercase().starts_with("host: ") => {
            Ok(s.split(":").nth(1).unwrap().trim().to_string())
        }
        _ => Err(()),
    }
}

fn parse_user_agent(s: &str) -> Result<String, ()> {
    match s {
        s if s.to_lowercase().starts_with("user-agent: ") => {
            Ok(s.split(":").nth(1).unwrap().trim().to_string())
        }
        _ => Err(()),
    }
}

fn parse_accept(s: &str) -> Result<String, ()> {
    match s {
        s if s.to_lowercase().starts_with("accept: ") => {
            Ok(s.split(":").nth(1).unwrap().trim().to_string())
        }
        _ => Err(()),
    }
}

fn parse_accept_encoding(s: &str) -> Result<Vec<HttpEncoding>, ()> {
    match s {
        s if s.to_lowercase().starts_with("accept-encoding: ") => {
            let encodings = s.split(":").nth(1).unwrap().trim().split(",");
            let mut result = Vec::new();
            for encoding in encodings {
                match encoding.trim() {
                    "gzip" => result.push(HttpEncoding::Gzip),
                    "compress" => result.push(HttpEncoding::Compress),
                    "deflate" => result.push(HttpEncoding::Deflate),
                    "br" => result.push(HttpEncoding::Br),
                    _ => continue,
                }
            }
            Ok(result)
        }
        _ => Err(()),
    }
}
