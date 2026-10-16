use std::{
    io::{BufRead, BufReader},
    net::{SocketAddr, UdpSocket},
    process::{Child, Command, Stdio},
    time::Duration,
};

struct Probe(Child);

impl Drop for Probe {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn exchanges_reference_handshake_over_loopback_udp() {
    let mut probe = Probe(
        Command::new(env!("CARGO_BIN_EXE_soe-probe"))
            .arg("127.0.0.1:0")
            .stdout(Stdio::piped())
            .spawn()
            .expect("start probe"),
    );
    let mut output = BufReader::new(probe.0.stdout.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let address: SocketAddr = line
        .trim()
        .strip_prefix("LISTENING ")
        .expect("probe reports its bound address")
        .parse()
        .unwrap();

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    socket.send_to(b"malformed", address).unwrap();
    socket
        .send_to(include_bytes!("fixtures/session-request.bin"), address)
        .unwrap();
    let mut buffer = [0u8; 128];
    let (count, peer) = socket.recv_from(&mut buffer).unwrap();
    assert_eq!(peer, address);
    assert_eq!(
        &buffer[..count],
        include_bytes!("fixtures/session-response.bin")
    );
    assert!(probe.0.wait().unwrap().success());
}
