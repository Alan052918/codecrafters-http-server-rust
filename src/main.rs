// Uncomment this block to pass the first stage
use std::{
    env,
    fmt::Debug,
    fs,
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
            "" => break,
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

    let request = HttpRequest::from_str(&request_string).expect("Failed to parse request");
    println!("target:{:?}", request.target);
    let response = match request.target {
        HttpTarget::Root => handle_root(),
        HttpTarget::Echo(s) => handle_echo(&s),
        HttpTarget::UserAgent => {
            handle_user_agent(&request.user_agent.expect("Failed to get user agent"))
        }
        HttpTarget::Files(filename) => {
            handle_files(&filename, directory, &request.method, &request.body)
        }
        HttpTarget::NotFound => handle_not_found(),
    };
    stream
        .write(response.to_string().as_bytes())
        .expect("Failed to write response buffer");
}

fn handle_root() -> HttpResponse {
    HttpResponse {
        version: HttpVersion::Http11,
        status: HttpStatus::Ok,
        content_type: HttpContentType::TextPlain,
        content_length: 13,
        body: "Hello, World!".to_string(),
    }
}

fn handle_echo(s: &str) -> HttpResponse {
    HttpResponse {
        version: HttpVersion::Http11,
        status: HttpStatus::Ok,
        content_type: HttpContentType::TextPlain,
        content_length: s.len(),
        body: s.to_string(),
    }
}

fn handle_user_agent(user_agent: &str) -> HttpResponse {
    HttpResponse {
        version: HttpVersion::Http11,
        status: HttpStatus::Ok,
        content_type: HttpContentType::TextPlain,
        content_length: user_agent.len(),
        body: user_agent.to_string(),
    }
}

fn handle_files(
    filename: &str,
    directory: &str,
    request_method: &HttpMethod,
    request_body: &str,
) -> HttpResponse {
    match request_method {
        HttpMethod::Get => match get_file(filename, directory) {
            Ok(file_content) => HttpResponse {
                version: HttpVersion::Http11,
                status: HttpStatus::Ok,
                content_type: HttpContentType::Application(HttpApplicationContentType::OctetStream),
                content_length: file_content.len(),
                body: file_content,
            },
            _ => HttpResponse {
                version: HttpVersion::Http11,
                status: HttpStatus::NotFound,
                content_type: HttpContentType::Application(HttpApplicationContentType::OctetStream),
                content_length: 0,
                body: String::new(),
            },
        },
        HttpMethod::Post => match post_file(filename, directory, request_body) {
            Ok(()) => HttpResponse {
                version: HttpVersion::Http11,
                status: HttpStatus::Created,
                content_type: HttpContentType::Application(HttpApplicationContentType::OctetStream),
                content_length: 0,
                body: String::new(),
            },
            _ => HttpResponse {
                version: HttpVersion::Http11,
                status: HttpStatus::NotFound,
                content_type: HttpContentType::Application(HttpApplicationContentType::OctetStream),
                content_length: 0,
                body: String::new(),
            },
        },
    }
}

fn get_file(filename: &str, directory: &str) -> Result<String, std::io::Error> {
    let path_string = format!("{}{}", directory, filename);
    println!("filename:{}", filename);
    println!("directory:{}", directory);
    println!("path_string:{}", path_string);
    let path = Path::new(path_string.as_str());
    if path.exists() {
        fs::read_to_string(path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ))
    }
}

fn post_file(filename: &str, directory: &str, content: &str) -> Result<(), std::io::Error> {
    let path_string = format!("{}{}", directory, filename);
    let path = Path::new(path_string.as_str());
    if path.exists() {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(path)
            .expect("Failed to open file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to file");
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ))
    }
}

fn handle_not_found() -> HttpResponse {
    HttpResponse {
        version: HttpVersion::Http11,
        status: HttpStatus::NotFound,
        content_type: HttpContentType::TextPlain,
        content_length: 0,
        body: String::new(),
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
    body: String,
}

impl HttpRequest {
    fn new(method: HttpMethod, target: HttpTarget, version: HttpVersion, body: String) -> Self {
        HttpRequest {
            method,
            target,
            version,
            host: None,
            user_agent: None,
            accept: None,
            accept_encoding: Vec::new(),
            body,
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
        let mut request = HttpRequest::new(method, target, version, String::new());
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
        if let Some(body) = request_lines.next() {
            request.body = body.to_string();
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
            "{} {}\r\nContent-Type: {:?}\r\nContent-Length: {}\r\n\r\n{}",
            self.version.to_string(),
            self.status.to_string(),
            self.content_type,
            self.content_length,
            self.body
        )
    }
}

#[allow(dead_code)]
enum HttpMethod {
    Get,
    Post,
}

enum HttpTarget {
    Root,
    Echo(String),
    UserAgent,
    Files(String),
    NotFound,
}

impl Debug for HttpTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpTarget::Root => write!(f, "/"),
            HttpTarget::Echo(s) => write!(f, "/echo/{}", s),
            HttpTarget::UserAgent => write!(f, "/user-agent"),
            HttpTarget::Files(s) => write!(f, "/files/{}", s),
            HttpTarget::NotFound => write!(f, "Not Found"),
        }
    }
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
    Created = 201,
    NotFound = 404,
}

impl ToString for HttpStatus {
    fn to_string(&self) -> String {
        match self {
            HttpStatus::Ok => "200 OK".to_string(),
            HttpStatus::Created => "201 CREATED".to_string(),
            HttpStatus::NotFound => "404 NOT FOUND".to_string(),
        }
    }
}

enum HttpContentType {
    TextPlain,
    Application(HttpApplicationContentType),
}

impl Debug for HttpContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpContentType::TextPlain => write!(f, "text/plain"),
            HttpContentType::Application(content_type) => write!(f, "{:?}", content_type),
        }
    }
}

enum HttpApplicationContentType {
    OctetStream,
}

impl Debug for HttpApplicationContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpApplicationContentType::OctetStream => write!(f, "application/octet-stream"),
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
