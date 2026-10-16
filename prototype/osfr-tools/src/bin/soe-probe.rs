//! First transport slice only: the legacy SOE session handshake, on loopback.
//! Reliable data, gateway login, characters and world simulation are not implemented.
use osfr_tools::session::SessionRequest;
use std::{
    env, io,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

fn session_response(packet: &[u8]) -> Result<Vec<u8>, &'static str> {
    Ok(SessionRequest::parse(packet)?.response())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 1 {
        return Err(
            "usage: soe-probe 127.0.0.1:PORT (one valid handshake, 10-second idle timeout)".into(),
        );
    }
    let address: SocketAddr = args[0].parse()?;
    if !address.ip().is_loopback() {
        return Err("this probe only binds a loopback address".into());
    }
    let socket = UdpSocket::bind(address)?;
    socket.set_read_timeout(Some(Duration::from_secs(10)))?;
    println!("LISTENING {}", socket.local_addr()?);
    let mut buffer = [0; 65535];
    loop {
        let (count, peer) = socket.recv_from(&mut buffer)?;
        match session_response(&buffer[..count]) {
            Ok(response) => {
                if socket.send_to(&response, peer)? != response.len() {
                    return Err(io::Error::other("short UDP send").into());
                }
                println!(
                    "SESSION_RESPONSE sent; transport probe complete, game login not implemented"
                );
                return Ok(());
            }
            Err(reason) => eprintln!("Ignored datagram: {reason}"),
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const REQUEST: &[u8] = include_bytes!("../../tests/fixtures/session-request.bin");
    const RESPONSE: &[u8] = include_bytes!("../../tests/fixtures/session-response.bin");
    #[test]
    fn matches_supplied_dotnet_writer() {
        assert_eq!(session_response(REQUEST).unwrap(), RESPONSE);
    }
    #[test]
    fn echoes_session_and_buffer_fields() {
        let mut request = REQUEST.to_vec();
        request[6..10].copy_from_slice(&0xfedcba98u32.to_be_bytes());
        request[10..14].copy_from_slice(&1024u32.to_be_bytes());
        let response = session_response(&request).unwrap();
        assert_eq!(&response[2..6], &request[6..10]);
        assert_eq!(&response[13..17], &request[10..14]);
    }
    #[test]
    fn rejects_wrong_protocol_and_invalid_lengths() {
        assert!(session_response(&REQUEST[..10]).is_err());
        let mut request = REQUEST.to_vec();
        request[14] = b'X';
        assert!(session_response(&request).is_err());
        request.copy_from_slice(REQUEST);
        request[2..6].copy_from_slice(&256u32.to_be_bytes());
        assert!(session_response(&request).is_err());
    }
}
