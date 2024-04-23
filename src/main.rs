// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => handle_connection(_stream),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request: Vec<String> = BufReader::new(&stream)
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    request.iter().for_each(|line| println!("{}", line));

    let request_line = HttpRequest::from_str(request[0].as_str()).unwrap();
    let (response_status, response_body) = match request_line.target {
        HttpTarget::Root => (HttpStatus::Ok, "Hello, World!".to_string()),
        HttpTarget::Echo(ref s) => (HttpStatus::Ok, s.clone()),
        HttpTarget::NotFound => (HttpStatus::NotFound, "".to_string()),
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
    host: String,
    user_agent: String,
}

impl FromStr for HttpRequest {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut line = s.split_whitespace();
        let method = HttpMethod::from_str(line.next().unwrap())?;
        let target = HttpTarget::from_str(line.next().unwrap())?;
        let version = HttpVersion::from_str(line.next().unwrap())?;
        let host = line.next().unwrap_or("Fail to parse host").to_string();
        let user_agent = line
            .next()
            .unwrap_or("Fail to parse user agent")
            .to_string();

        Ok(HttpRequest {
            method,
            target,
            version,
            host,
            user_agent,
        })
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

impl FromStr for HttpMethod {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            _ => Err(()),
        }
    }
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
    NotFound,
}

impl FromStr for HttpTarget {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "/" => Ok(HttpTarget::Root),
            s if s.starts_with("/echo/") => Ok(HttpTarget::Echo(
                s.strip_prefix("/echo/").unwrap().to_string(),
            )),
            _ => Ok(HttpTarget::NotFound),
        }
    }
}

enum HttpVersion {
    Http11,
}

impl FromStr for HttpVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(HttpVersion::Http11),
            _ => Err(()),
        }
    }
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
