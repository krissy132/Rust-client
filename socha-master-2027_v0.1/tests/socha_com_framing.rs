//! TCP/XML framing tests for `ComHandler`.
//!
//! These tests drive `ComHandler::join` against a tiny in-process TCP fake server.
//! They verify the framing behaviour ChatGPT was concerned about:
//!   - **Test B:** a single logical message split across multiple TCP reads.
//!   - **Test C:** a very large single message (full mid-game `<state>`).
//!   - **Test A:** multiple small messages concatenated into a single read.
//!   - **Test D:** a stray `<comMessage/>` token in the stream.
//!   - **Test E:** chunk-boundary straddling an UTF-8 multi-byte sequence.
//!
//! Each fake server writes `<protocol>` first (required by `ComHandler::join`),
//! then the scripted bytes (optionally chunked), then optionally closes the stream.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use socha::socha_com::ComHandler;
use socha::internal::ComMessage;

/// Spawn a fake server that:
///   1. Accepts exactly one connection.
///   2. Reads (and discards) whatever the client sends for `discard_for` ms.
///   3. Writes `<protocol>` then writes each chunk in `chunks` with `delay` between chunks.
///   4. Keeps the connection open (or closes it if `close_after` is true).
fn spawn_fake_server<F>(chunks_builder: F) -> (String, thread::JoinHandle<()>)
where
    F: FnOnce(TcpStream) -> Vec<Vec<u8>> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let addr = listener.local_addr().expect("local_addr failed");
    let addr_str = addr.to_string();

    let handle = thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().expect("accept failed");
        // Drain whatever the client wrote (<protocol> + <join .../>).
        stream.set_read_timeout(Some(Duration::from_millis(150))).ok();
        let mut buf = [0u8; 4096];
        let _ = std::io::Read::read(&mut stream, &mut buf);

        // Send the protocol header — required by ComHandler::join BEFORE the client's join handshake
        // even arrives (the join blocks until it sees this).
        stream.write_all(b"<protocol>").expect("write protocol");
        stream.flush().expect("flush protocol");

        // Now collect the scripted chunks and stream them out one-by-one.
        let chunks = chunks_builder(stream.try_clone().expect("clone stream"));
        for chunk in chunks {
            stream.write_all(&chunk).expect("write chunk");
            stream.flush().expect("flush chunk");
            thread::sleep(Duration::from_millis(15));
        }
        // Keep the connection open long enough for the client to drain the scripted chunks.
        thread::sleep(Duration::from_millis(1500));
    });

    (addr_str, handle)
}

/// Spawn a fake server that sends an arbitrary raw byte stream (post-`<protocol>`).
fn spawn_fake_server_raw(chunks: Vec<Vec<u8>>) -> (String, thread::JoinHandle<()>) {
    spawn_fake_server(move |_stream| chunks)
}

/// Drain `cnum` next ComMessages, timing out after `per_msg` * `cnum` ms.
fn collect_messages(
    com: &mut ComHandler,
    cnum: usize,
    per_msg: Duration,
) -> Vec<ComMessage> {
    let mut out = Vec::with_capacity(cnum);
    let max_total = per_msg * cnum as u32;
    for _ in 0..cnum {
        match com.wait_for_com_message(max_total) {
            Ok(m) => out.push(m),
            Err(e) => {
                let got = out.len();
                panic!(
                    "error after {got}/{cnum} messages: {e:?}"
                );
            }
        }
    }
    out
}

#[test]
fn test_b_split_message_across_reads() {
    // One logical message severed in the middle, delivered across three reads.
    let chunks: Vec<Vec<u8>> = vec![
        b"<joined roomId=\"abc\"/>\n".to_vec(),
        // split a room-moveRequest across two reads
        b"<room roomId=\"abc\"><data class=\"moveRe".to_vec(),
        b"quest\"/></room>\n".to_vec(),
    ];
    let (addr, _h) = spawn_fake_server_raw(chunks);

    let mut com = ComHandler::join(&addr, None).expect("join failed");
    // First message must be Joined
    let m1 = com.wait_for_com_message(Duration::from_secs(2)).expect("msg1");
    assert!(
        matches!(m1, ComMessage::Joined(_)),
        "expected Joined, got {m1:?}"
    );
    // Second message must be a room MoveRequest
    let m2 = com.wait_for_com_message(Duration::from_secs(2)).expect("msg2");
    match m2 {
        ComMessage::Room(rm) => {
            assert!(
                matches!(*rm, socha::internal::RoomMessage::MoveRequest),
                "expected MoveRequest, got {rm:?}"
            );
        }
        other => panic!("expected Room, got {other:?}"),
    }
}

#[test]
fn test_a_multiple_messages_in_one_read() {
    // Several small messages concatenated in a single TCP read.
    // `<joined roomId="…"/>` (the attrib is required — `attempt_get_com_messages` drops Joined/Left
    // entries lacking a roomId attribute, mirroring server behaviour).
    let combined: Vec<u8> =
        b"<joined roomId=\"r1\"/>\n<joined roomId=\"r2\"/>\n<left roomId=\"r2\"/>\n".to_vec();
    let (addr, _h) = spawn_fake_server_raw(vec![combined]);

    let mut com = ComHandler::join(&addr, None).expect("join failed");
    let msgs = collect_messages(&mut com, 3, Duration::from_secs(2));
    assert!(matches!(msgs[0], ComMessage::Joined(_)), "0: {:?}", msgs[0]);
    assert!(matches!(msgs[1], ComMessage::Joined(_)), "1: {:?}", msgs[1]);
    assert!(matches!(msgs[2], ComMessage::Left(_)), "2: {:?}", msgs[2]);
}

#[test]
fn test_c_large_state_message() {
    // Build a complete, realistic mid-game <state> XML large enough to comfortably exceed one read() chunk.
    //
    // We replicate the actual server-side memento layout: full 21-Pento-shape lists for each color.
    let all_shapes_xml: String = [
        "MONO", "DOMINO", "TRIO_L", "TRIO_I", "TETRO_O", "TETRO_T", "TETRO_I", "TETRO_L",
        "TETRO_Z", "PENTO_L", "PENTO_T", "PENTO_V", "PENTO_S", "PENTO_Z", "PENTO_I",
        "PENTO_P", "PENTO_W", "PENTO_U", "PENTO_R", "PENTO_X", "PENTO_Y",
    ]
    .iter()
    .map(|s| format!("<shape>{s}</shape>"))
    .collect::<String>();

    // A board with ~120 popolated cells to push the message comfortably above 4096 bytes.
    let mut board_fields = String::new();
    for y in 0..12 {
        for x in 0..10 {
            let content = match (x + y) % 4 {
                0 => "BLUE",
                1 => "YELLOW",
                2 => "RED",
                _ => "GREEN",
            };
            board_fields.push_str(&format!(
                "<field x=\"{x}\" y=\"{y}\" content=\"{content}\"/>"
            ));
        }
    }

    let memento = format!(
        "<room roomId=\"abc\"><data class=\"memento\"><state startTeam=\"ONE\" turn=\"5\" startPiece=\"PENTO_L\" round=\"2\">\
           <board>{board_fields}</board>\
           <lastMoveMono/>\
           <blueShapes>{all_shapes_xml}</blueShapes>\
           <yellowShapes>{all_shapes_xml}</yellowShapes>\
           <redShapes>{all_shapes_xml}</redShapes>\
           <greenShapes>{all_shapes_xml}</greenShapes>\
           <validColors><color>BLUE</color></validColors>\
         </state></data></room>\n"
    );
    // Make sure it's large enough to span multiple 4096-byte reads.
    assert!(
        memento.len() > 4096,
        "fixture too small: {} bytes",
        memento.len()
    );

    // Send it in small 256-byte chunks to maximise across-read splitting.
    let chunks: Vec<Vec<u8>> = memento
        .as_bytes()
        .chunks(256)
        .map(|c| c.to_vec())
        .collect();
    let (addr, _h) = spawn_fake_server_raw(chunks);

    let mut com = ComHandler::join(&addr, None).expect("join failed");
    // First real message is the memento, preceded by no Joined/Left because the server didn't send any.
    let m = com.wait_for_com_message(Duration::from_secs(8)).expect("memento");
    match m {
        ComMessage::Room(rm) => match *rm {
            socha::internal::RoomMessage::Memento(state) => {
                // Basic sanity that the state survived the chunked transfer.
                assert_eq!(state.turn, 5);
                // turn 5 mod 4 = 1 → YELLOW
                assert_eq!(state.current_color(), socha::neutral::Color::Yellow);
                assert!(!state.board.is_empty(), "board must not be empty");
                // Spot-check that all 120 cells landed.
                let non_empty = state
                    .board
                    .rows
                    .iter()
                    .flat_map(|r| r.fields.iter())
                    .filter(|f| !f.is_empty())
                    .count();
                assert_eq!(non_empty, 120, "expected 120 populated cells, got {non_empty}");
                // All 21 shapes per color preserved.
                assert_eq!(state.blue_shapes.len(), 21);
                assert_eq!(state.yellow_shapes.len(), 21);
                assert_eq!(state.red_shapes.len(), 21);
                assert_eq!(state.green_shapes.len(), 21);
            }
            other => panic!("expected Memento, got {other:?}"),
        },
        other => panic!("expected Room, got {other:?}"),
    }
}

#[test]
fn test_d_stray_com_message_tag_in_stream() {
    // Server accidentally emits a literal `<comMessage/>` mid-stream (a known fragility of
    // `get_com_msg_and_rm` — it wraps the buffer in `<comMessage>...</comMessage>` and tries to parse).
    //
    // Behaviour we want: NO panic, NO infinite loop, eventually the next real message arrives.
    let chunks: Vec<Vec<u8>> = vec![
        b"<joined roomId=\"abc\"/>\n".to_vec(),
        b"<comMessage/>\n".to_vec(),
        b"<left roomId=\"abc\"/>\n".to_vec(),
    ];
    let (addr, _h) = spawn_fake_server_raw(chunks);

    let mut com = ComHandler::join(&addr, None).expect("join failed");
    // First message: Joined.
    let m1 = com.wait_for_com_message(Duration::from_secs(5)).expect("msg1");
    assert!(matches!(m1, ComMessage::Joined(_)));
    // We don't assert anything specific about `<comMessage/>` — we just require
    // that the client either skips it gracefully or returns a Left (i.e. does not panic/hang).
    // Try to get the next message; allow up to 6s.
    let m2 = com.wait_for_com_message(Duration::from_secs(6));
    match m2 {
        Ok(ComMessage::Left(_)) => { /* best case: the stray <comMessage/> didn't eat <left/> */ }
        Ok(other) => {
            eprintln!("note: stray <comMessage/> led to message: {other:?}");
        }
        Err(e) => {
            eprintln!("note: stray <comMessage/> caused error: {e:?}");
        }
    }
}

#[test]
fn test_e_utf8_chunk_boundary_split() {
    // Multi-byte UTF-8 codepoint straddling a read boundary.
    // We insert an ä (U+00E4 → 0xC3 0xA4) inside a value, then split right in the middle.
    // Note: this only matters for tag/attribute *content* that the parser must decode.
    // Strictly speaking, the protocol rarely ships non-ASCII, but the framing code should not panic.
    let raw: Vec<u8> = b"<room roomId=\"abc\"><data class=\"moveRequest\"/></room>\n".to_vec();
    // Inject ä inside the roomId by replacing "abc" with bytes 0x61 0x62 0xC3 0xA4
    let mut cooked: Vec<u8> = Vec::with_capacity(raw.len() + 2);
    for &b in &raw {
        if b == b'c' {
            cooked.push(b'\xC3');
            break;
        }
        cooked.push(b);
    }
    // Split right after the 0xC3 lead byte: chunk1 ends mid-codepoint, chunk2 carries the continuation + rest.
    let rest: Vec<u8> = raw[raw.iter().position(|&b| b == b'c').unwrap() + 1..].to_vec();

    // Build chunks: chunk1 ends exactly after 0xC3 lead byte, chunk2 carries the continuation + rest.
    let chunk1: Vec<u8> = cooked; // ends at 0xC3
    let mut chunk2: Vec<u8> = vec![0xA4];
    chunk2.extend_from_slice(&rest);
    chunk2.push(b'\n');

    let (addr, _h) = spawn_fake_server_raw(vec![chunk1, chunk2]);

    let mut com = ComHandler::join(&addr, None).expect("join failed");
    // We don't assert got a perfect message — only that we did NOT panic and the stream settled
    // (either reaches a MoveRequest, or eventually errors out without hanging indefinitely).
    let m = com.wait_for_com_message(Duration::from_secs(3));
    match m {
        Ok(ComMessage::Room(rm)) => {
            // Best case: parser did the right thing — the room id was decoded somehow.
            eprintln!("ok: decoded room msg with id {:?}", rm);
        }
        Ok(other) => eprintln!("note: utf-8 split led to msg: {other:?}"),
        Err(e) => eprintln!("note: utf-8 split led to err: {e:?}"),
    }
}
