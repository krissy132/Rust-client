//! Buffer-limit regression test.
//!
//! ChatGPT's risk P3.13: a corrupted/malformed stream that the XML parser can never
//! finish would cause the receive buffer to grow without bound. After the fix in
//! `try_read_new`, the buffer is capped at `MAX_BUF_BYTES` (1 MiB) and `try_read_new`
//! returns an error when crossed.
//!
//! Strategy: have a fake server send `<protocol>` then a **very large** payload that is
//! not valid XML (e.g. raw bytes with no closing tag). Then call `wait_for_com_message`
//! repeatedly and assert the client does NOT loop silently — it returns an error eventually
//! once the buffer crosses the 1 MiB threshold.

use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use socha::socha_com::ComHandler;

#[test]
fn malformed_stream_eventually_yields_buffer_overflow_error() {
    // The fake server sends <protocol> then a never-ending stream of "<room …>"-ish bytes
    // with no self-close, leaving the parser unable to finish.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local_addr").to_string();

    let handle = thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().expect("accept");
        // Drain the client's <protocol> + <join/>.
        stream.set_read_timeout(Some(Duration::from_millis(150))).ok();
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);

        stream.write_all(b"<protocol>").expect("write protocol");
        stream.flush().expect("flush protocol");

        // Now spam a malformed-but-XMLish byte sequence far past 1 MiB. Pick a base unit
        // resembling a real tag so a bug that *would* parse it is also caught.
        let chunk = b"<room roomId=\"abc\"><data class=\"memento\"><state>";
        // Send enough bytes to comfortably exceed the 1 MiB cap; pause briefly between
        // writes to give the client time to read & accumulate.
        let target_bytes = 1 << 21; // 2 MiB
        let mut sent = 0;
        while sent < target_bytes {
            match stream.write(chunk) {
                Ok(n) => {
                    sent += n;
                }
                Err(_) => {
                    // Socket buffer full — give the client time to drain.
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }
        let _ = stream.flush();
        // Hold the connection open long enough that the client can absorb all bytes and
        // trip its buffer limit. 30s comfortably covers cross-thread scheduling jitter.
        thread::sleep(Duration::from_secs(30));
        // Hold the connection open long enough that the client can absorb all bytes and
        // trip its buffer limit. 30s comfortably covers cross-thread scheduling jitter.
        thread::sleep(Duration::from_secs(30));
    });

    let mut com = ComHandler::join(&addr, None).expect("join failed");

    // Drive `try_for_com_message` in a tight loop. The fixed-size socket-buffer starts
    // filling; the client drains it; eventually `try_read_new` notices the receive
    // buffer is above `MAX_BUF_BYTES` and returns Err. We assert the error surfaces.
    let max_iterations = 200_000;
    let start = std::time::Instant::now();
    let max_duration = Duration::from_secs(45);
    let mut got_overflow_error = false;
    let mut ok_count = 0usize;
    let mut none_count = 0usize;
    for i in 0..max_iterations {
        if start.elapsed() > max_duration {
            break;
        }
        match com.try_for_com_message() {
            Ok(Some(_)) => ok_count += 1,
            Ok(None) => none_count += 1,
            Err(e) => {
                let s = format!("{e:?}");
                if s.contains("buffer exceeded") || s.contains("FailedToBuildRoomMessage") {
                    got_overflow_error = true;
                    break;
                }
                eprintln!("note: test ended with non-overflow error after {} iters, ok={}, none={}: {s}", i, ok_count, none_count);
                break;
            }
        }
    }
    assert!(
        got_overflow_error,
        "expected the client to eventually surface a buffer-overflow error after spamming; \
         after {}s iters done, ok={ok_count}, none={none_count}; \
         if this fails the limit guard was removed or `try_read_new` lost its check.",
        start.elapsed().as_secs(),
    );
    let _ = handle;
}
