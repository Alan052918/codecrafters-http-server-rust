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
    let response_status = match request_line.target.as_str() {
        "/" => HttpStatus::Ok,
        _ => HttpStatus::NotFound,
    };
    let response = HttpResponse {
        version: HttpVersion::Http11,
        status: response_status,
    };
    stream
        .write(response.to_string().as_bytes())
        .expect("Failed to write response buffer");
}

#[allow(dead_code)]
struct HttpRequest {
    method: HttpMethod,
    target: String,
    version: HttpVersion,
}

impl FromStr for HttpRequest {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut line = s.split_whitespace();
        let method = HttpMethod::from_str(line.next().unwrap())?;
        let target = line.next().unwrap().to_string();
        let version = HttpVersion::from_str(line.next().unwrap())?;

        Ok(HttpRequest {
            method,
            target,
            version,
        })
    }
}

struct HttpResponse {
    version: HttpVersion,
    status: HttpStatus,
}

impl ToString for HttpResponse {
    fn to_string(&self) -> String {
        format!("{} {}\r\n\r\n", self.version.as_ref(), self.status.as_ref())
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
