//! Studio: a tiny loopback-only HTTP/1.1 endpoint for developer tooling.
//!
//! The gateway polls it from its single-threaded loop (no threads, no new
//! dependencies). One request per connection; requests and bodies are bounded.
//! See notes/Design/Studio Toolkit.md.
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};

const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 1024 * 1024;

pub struct Request {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Read one request (head and Content-Length body) with a short timeout.
pub fn read_request(stream: &mut TcpStream) -> io::Result<Request> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    let mut data = Vec::new();
    let mut chunk = [0_u8; 4096];
    let head_end = loop {
        if let Some(at) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break at + 4;
        }
        if data.len() > MAX_HEAD {
            return Err(invalid("request head too large"));
        }
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(invalid("connection closed before the request head"));
        }
        data.extend_from_slice(&chunk[..read]);
    };
    let head = std::str::from_utf8(&data[..head_end]).map_err(|_| invalid("non-UTF-8 head"))?;
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split(' ');
    let method = first.next().unwrap_or_default().to_owned();
    let path = first.next().unwrap_or_default().to_owned();
    let length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim().parse::<usize>())
        .transpose()
        .map_err(|_| invalid("bad Content-Length"))?
        .unwrap_or(0);
    if length > MAX_BODY {
        return Err(invalid("request body too large"));
    }
    let mut body = data[head_end..].to_vec();
    while body.len() < length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(invalid("connection closed before the body"));
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(length);
    Ok(Request { method, path, body })
}

pub fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn reads_a_post_and_answers() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let client = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            stream
                .write_all(b"POST /api/command HTTP/1.1\r\nHost: x\r\nContent-Length: 17\r\n\r\n{\"text\":\"here\"}  ")
                .unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).unwrap();
            answer
        });
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream).unwrap();
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/api/command");
        assert_eq!(request.body, b"{\"text\":\"here\"}  ");
        write_response(&mut stream, 200, "application/json", b"{}").unwrap();
        drop(stream);
        let answer = client.join().unwrap();
        assert!(answer.starts_with("HTTP/1.1 200 OK"));
        assert!(answer.ends_with("{}"));
    }
}
