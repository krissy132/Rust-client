//! Disconnect / unexpected-close tests for `ComHandler`.
//!
//! ChatGPT's risk P3.9: the server may close the connection at any point — during the
//! initial handshake, while the client is computing a move, or while it is sending one.
//! We must never panic; we should surface a clean `ReceiveErr`.
//!
//! Each variant sends `<protocol>` then closes the stream abruptly. We confirm the
//! corresponding `ComHandler::join` / `wait_for_com_message` returns an Err rather than
//! panicking or hanging.

use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use socha::socha_com::ComHandler;

fn spawn_server_then_close(close_after_protocol: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local_addr").to_string();

    thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().expect("accept");
        // Drain what we can shortly.
        stream.set_read_timeout(Some(Duration::from_millis(150))).ok();
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);

        if !close_after_protocol {
            // Don't even send <protocol>; just close.
            return;
        }
        let _ = stream.write_all(b"<protocol>");
        let _ = stream.flush();
        // Drop immediately — closes the TCP connection.
    });

    addr
}

#[test]
fn disconnect_before_protocol_returns_err_no_panic() {
    // Server accepts the connection but immediately closes without sending `<protocol>`.
    let addr = spawn_server_then_close(false);
    let res = ComHandler::join(&addr, None);
    assert!(
        res.is_err(),
        "expected Err when server closes before <protocol>; got Ok"
    );
}

#[test]
fn disconnect_after_protocol_then_wait_returns_err_no_panic() {
    // Server sends <protocol>, then immediately drops the connection.
    let addr = spawn_server_then_close(true);
    let mut com = ComHandler::join(&addr, None).expect("join ok (we have <protocol>)");
    // Trying to read the next message must Err, not panic.
    let res = com.wait_for_com_message(Duration::from_secs(2));
    assert!(
        res.is_err(),
        "expected Err after server disconnect; got Ok with {res:?}"
    );
}

/// A server that delivers a valid `<joined/>` and a `<data class="moveRequest"/>`,
/// accepts the resulting move XML, then abruptly closes the stream while the client is
/// waiting for the next message. The client must surface an error, not hang or panic.
#[test]
fn disconnect_mid_session_returns_err_no_panic() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local_addr").to_string();

    let _h = thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().expect("accept");
        stream.set_read_timeout(Some(Duration::from_millis(200))).ok();
        // Drain handshake briefly.
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);

        // Send protocol + joined + moveRequest, then close.
        stream.write_all(b"<protocol><joined roomId=\"abc\"/><room roomId=\"abc\"><data class=\"moveRequest\"/></room>").expect("send fixture");
        stream.flush().expect("flush");

        // Give client a moment to consume.
        thread::sleep(Duration::from_millis(200));
        // Drop the stream; the client should see EOF/reset.
    });

    let mut com = ComHandler::join(&addr, None).expect("join ok");
    // First message: Joined.
    let m1 = com.wait_for_com_message(Duration::from_secs(2)).expect("joined");
    assert!(matches!(m1, socha::internal::ComMessage::Joined(_)));

    // Second: MoveRequest.
    let m2 = com.wait_for_com_message(Duration::from_secs(2)).expect("request");
    match m2 {
        socha::internal::ComMessage::Room(rm) => {
            assert!(matches!(*rm, socha::internal::RoomMessage::MoveRequest));
        }
        other => panic!("expected MoveRequest, got {other:?}"),
    }

    // Third: server closed — must Err, not panic.
    let res = com.wait_for_com_message(Duration::from_secs(2));
    assert!(res.is_err(), "expected Err after server closed mid-session");
}
