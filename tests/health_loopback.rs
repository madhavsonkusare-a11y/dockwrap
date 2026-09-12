use local_store::runtime::{HealthProbe, HttpHealthProbe};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

#[test]
fn localhost_health_reaches_an_ipv4_only_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(12);
        loop {
            if let Ok((mut stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .unwrap();
                let mut request = String::new();
                BufReader::new(&mut stream).read_line(&mut request).unwrap();
                assert_eq!(request, "GET / HTTP/1.0\r\n");
                stream
                    .write_all(b"HTTP/1.0 200 OK\r\nContent-Length: 0\r\n\r\n")
                    .unwrap();
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "probe never reached IPv4 listener"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    let ready = HttpHealthProbe.ready(&format!("http://localhost:{port}"));
    server.join().unwrap();
    assert!(ready);
}

/// A server that routes on the whole Host header, the way Joplin does, has
/// to be sent the port — or it answers 404 and the app is never "healthy".
#[test]
fn the_health_probe_names_the_port_in_host() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut host = String::new();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                break;
            }
            if let Some(value) = line.strip_prefix("Host: ") {
                host = value.trim().to_owned();
            }
        }
        let reply: &[u8] = if host == format!("localhost:{port}") {
            b"HTTP/1.0 302 Found\r\nLocation: /login\r\nContent-Length: 0\r\n\r\n"
        } else {
            b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n"
        };
        stream.write_all(reply).unwrap();
        host
    });
    let ready = HttpHealthProbe.ready(&format!("http://localhost:{port}"));
    let host = server.join().unwrap();
    assert!(ready, "the probe sent Host: {host}");
}
