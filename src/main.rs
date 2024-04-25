// Uncomment this block to pass the first stage
use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    str::FromStr,
    thread,
};

const IP_ADDR: &str = "127.0.0.1";
const PORT: &str = "4221";

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let mut ip_addr = IP_ADDR.to_string();
    let mut port = PORT.to_string();
    let mut directory = "".to_string();

    let mut args_iter = env::args().filter(|arg| !arg.trim().is_empty());
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--ip" => args_iter
                .next()
                .filter(|ip| !ip.trim().is_empty())
                .map(|ip| ip_addr = ip),
            "--port" => args_iter
                .next()
                .filter(|p| !p.trim().is_empty())
                .map(|p| port = p),
            "--directory" => args_iter
                .next()
                .filter(|d| !d.trim().is_empty())
                .map(|d| directory = d.to_string()),
            _ => continue,
        };
    }

    let addr = format!("{}:{}", ip_addr, port);
    let listener = TcpListener::bind(&addr).expect(format!("Failed to bind: {}", &addr).as_str());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory = directory.clone();
                thread::spawn(move || handle_connection(stream, directory.as_str()));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, directory: &str) {
    let mut buffer = [0; 1024]; // Create a mutable buffer of fixed size
    let bytes_read = stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let request_string = String::from_utf8_lossy(&buffer[..bytes_read]).to_string(); // Convert the buffer to a string
    println!("{} {}", bytes_read, request_string);

    let request = HttpRequest::from_str(&request_string).unwrap();
    let (response_status, response_content_type, response_body) = match request.target {
        HttpTarget::Root => (
            HttpStatus::Ok,
            HttpContentType::TextPlain,
            "Hello, World!".to_string(),
        ),
        HttpTarget::Echo(s) => (HttpStatus::Ok, HttpContentType::TextPlain, s),
        HttpTarget::UserAgent => (
            HttpStatus::Ok,
            HttpContentType::TextPlain,
            request.user_agent.expect("Failed to get user agent"),
        ),
        HttpTarget::Files(s) => match query_file(&s, directory) {
            Ok(file_content) => (
                HttpStatus::Ok,
                HttpContentType::Application(HttpApplicationContentType::OctetStream),
                file_content,
            ),
            _ => (
                HttpStatus::NotFound,
                HttpContentType::Application(HttpApplicationContentType::OctetStream),
                "".to_string(),
            ),
        },
        HttpTarget::NotFound => (
            HttpStatus::NotFound,
            HttpContentType::TextPlain,
            String::new(),
        ),
    };
    let response = HttpResponse {
        version: request.version,
        status: response_status,
        content_type: response_content_type,
        content_length: response_body.len(),
        body: response_body,
    };
    stream
        .write(response.to_string().as_bytes())
        .expect("Failed to write response buffer");
}

fn query_file(path: &str, directory: &str) -> Result<String, std::io::Error> {
    match directory {
        directory if path.starts_with(directory) && Path::new(path).exists() => {
            fs::read_to_string(path)
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        )),
    }
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
            self.version.to_string(),
            self.status.to_string(),
            self.content_type.to_string(),
            self.content_length,
            self.body
        )
    }
}

enum HttpMethod {
    Get,
}

enum HttpTarget {
    Root,
    Echo(String),
    UserAgent,
    Files(String),
    NotFound,
}

enum HttpEncoding {
    Gzip,
    Compress,
    Deflate,
    Br,
}

enum HttpVersion {
    Http11,
}

impl ToString for HttpVersion {
    fn to_string(&self) -> String {
        match self {
            HttpVersion::Http11 => "HTTP/1.1".to_string(),
        }
    }
}

enum HttpStatus {
    Ok = 200,
    NotFound = 404,
}

impl ToString for HttpStatus {
    fn to_string(&self) -> String {
        match self {
            HttpStatus::Ok => "200 OK".to_string(),
            HttpStatus::NotFound => "404 NOT FOUND".to_string(),
        }
    }
}

enum HttpContentType {
    TextPlain,
    Application(HttpApplicationContentType),
}

impl ToString for HttpContentType {
    fn to_string(&self) -> String {
        match self {
            HttpContentType::TextPlain => "text/plain".to_string(),
            HttpContentType::Application(content_type) => content_type.to_string(),
        }
    }
}

enum HttpApplicationContentType {
    OctetStream,
}

impl ToString for HttpApplicationContentType {
    fn to_string(&self) -> String {
        match self {
            HttpApplicationContentType::OctetStream => "application/octet-stream".to_string(),
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
        Some(s) if s.starts_with("/files/") => Ok(HttpTarget::Files(
            s.strip_prefix("/files/").unwrap().to_string(),
        )),
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
