// Guards the datetime query-param rewrite in scripts/fix-datetime-query-params.py: the
// upstream rust generator serializes chrono DateTime query params via Display, which is
// not RFC 3339 and is rejected by the Stedi API. If a regeneration ever reintroduces the
// raw `to_string()` serialization, this test fails.
#![cfg(feature = "core")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::JoinHandle;

/// Serves exactly one request, returning its request line ("GET /path?query HTTP/1.1").
fn capture_one_request(listener: TcpListener) -> JoinHandle<String> {
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        let head = String::from_utf8_lossy(&buf[..n]).to_string();
        let body = r#"{"items":[]}"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).unwrap();
        head.lines().next().unwrap_or_default().to_string()
    })
}

#[tokio::test]
async fn datetime_query_params_are_rfc3339() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = capture_one_request(listener);

    let config = autogen_stedi::core::apis::configuration::Configuration {
        base_path: format!("http://{addr}"),
        ..Default::default()
    };
    let start = chrono::DateTime::parse_from_rfc3339("2026-07-17T00:00:00Z").unwrap();
    let _ = autogen_stedi::core::apis::default_api::list_polling_transactions(
        &config,
        None,
        None,
        Some(start),
    )
    .await;

    let request_line = server.join().unwrap();
    // ':' is form-encoded as %3A by reqwest's query serializer.
    assert!(
        request_line.contains("startDateTime=2026-07-17T00%3A00%3A00Z"),
        "startDateTime must be RFC 3339 (2026-07-17T00:00:00Z); request was: {request_line}"
    );
}
