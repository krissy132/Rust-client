use std::time::Duration;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use log::info;
pub mod handler_trait;
use crate::i_client_handler::handler_trait::IClientHandler;
use crate::neutral::Move;

#[derive(Debug, Clone)]
pub enum SendCommand {
    Move(Move),
    SendRaw { xml: String },
    Admin(SendAdminCommand),
}

#[derive(Debug, Clone)]
pub enum SendAdminCommand {
    Authenticate {
        pass: String,
    },
    Observe {
        room_id: String,
    },
    Pause {
        room_id: String,
        pause: bool,
    },
    /// step a pause room forward by one move
    Step {
        room_id: String,
    },
    Cancel {
        room_id: String,
    },
    /// prepares a new room
    Prepare {
        pause: bool,
    },
}

pub fn start_iclient<I>(
    addr: &str,
    opt_reservation_code: Option<&str>,
    i_client_handler: &mut I,
    thread_sleep_time: Duration,
    timeout: Duration,
) -> Result<(), crate::error::ReceiveErr>
where
    I: IClientHandler,
{
    use crate::socha_com::ComHandler;
    use crossbeam_channel::unbounded;

    let mut com = ComHandler::join(addr, opt_reservation_code)?;
    let (msg_tx, msg_rx) = unbounded::<crate::internal::ComMessage>();
    let (watch_tx, watch_rx) = unbounded::<crate::internal::ComMessage>();
    let (out_tx, out_rx) = unbounded::<SendCommand>();
    let reader_handle = std::thread::spawn(move || loop {
        // messages from server
        match com.try_for_com_message() {
            Ok(Some(msg)) => {
                let _ = msg_tx.send(msg.clone());
                let _ = watch_tx.send(msg);
            }
            Ok(None) => {
                std::thread::sleep(thread_sleep_time);
            }
            Err(_e) => {
                break;
            }
        }
        // forwarding messages from the main loop to the server
        if let Ok(out_msg) = out_rx.try_recv() {
            match out_msg {
                SendCommand::Move(mv) => {
                    let _ = com.send_move(&mv);
                }
                SendCommand::SendRaw { xml } => {
                    let _ = com.send_raw(&xml);
                }
                SendCommand::Admin(_admin_cmd) => {
                    // not wired up for the iclient_handler interface
                }
            }
        }
    });

    loop {
        use crate::internal::{ComMessage, RoomMessage};
        use crossbeam_channel::TryRecvError;

        match msg_rx.try_recv() {
            Ok(com_message) => match com_message {
                ComMessage::Joined(joined) => {
                    info!("joined room {}", joined.room_id);
                    i_client_handler.on_game_joined(&joined.room_id);
                }
                ComMessage::Left(left) => {
                    info!("left room {}", left.room_id);
                    i_client_handler.on_game_left();
                }
                ComMessage::Room(room_msg) => match *room_msg {
                    RoomMessage::Memento(state) => {
                        info!("got board: \n{}", state.board);
                        info!("turn {}", state.turn);
                        i_client_handler.on_gamestate_update(*state);
                    }
                    RoomMessage::WelcomeMessage { color } => {
                        info!("got welcome message (team={})", color);
                        i_client_handler.on_welcome_message(color);
                    }
                    RoomMessage::MoveRequest => {
                        info!("got move request");
                        let mv = i_client_handler.calculate_move();
                        let _ = out_tx.send(SendCommand::Move(mv));
                        let cancel_handler = ComCancelHandler::new_from_receiver(
                            watch_rx.clone(),
                            timeout,
                            thread_sleep_time,
                        );
                        i_client_handler.while_waiting(cancel_handler);
                    }
                    RoomMessage::Result(result) => {
                        info!("got result: \n{:#?}", result);
                        i_client_handler.on_game_result(&result);
                    }
                },
                ComMessage::Admin(_) => {}
            },
            Err(TryRecvError::Empty) => {
                std::thread::sleep(thread_sleep_time);
            }
            Err(TryRecvError::Disconnected) => {
                info!("worker channel disconnected, exiting");
                break;
            }
        }
    }
    let _ = reader_handle.join();
    Ok(())
}

use crossbeam_channel::Receiver;

pub struct ComCancelHandler {
    flag: Arc<AtomicBool>,
    watchdog_handle: Option<JoinHandle<()>>,
}

impl ComCancelHandler {
    pub fn new_from_receiver(
        rx: Receiver<crate::internal::ComMessage>,
        timeout: Duration,
        thread_sleep_time: Duration,
    ) -> Self {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let handle = std::thread::spawn(move || {
            // drain messages
            while let Ok(_m) = rx.try_recv() {}

            let start = Instant::now();
            loop {
                if flag_clone.load(Ordering::SeqCst) {
                    break;
                }
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        crate::internal::ComMessage::Room(room_msg) => {
                            if matches!(*room_msg, crate::internal::RoomMessage::MoveRequest) {
                                flag_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                        crate::internal::ComMessage::Left(_id) => {
                            flag_clone.store(true, Ordering::SeqCst);
                            break;
                        }
                        _ => {}
                    },
                    Err(crossbeam_channel::TryRecvError::Empty) => {}
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        flag_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }

                if start.elapsed() >= timeout {
                    flag_clone.store(true, Ordering::SeqCst);
                    break;
                }

                thread::sleep(thread_sleep_time);
            }
        });

        ComCancelHandler {
            flag,
            watchdog_handle: Some(handle),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

impl Drop for ComCancelHandler {
    fn drop(&mut self) {
        if let Some(h) = self.watchdog_handle.take() {
            let _ = h.join();
        }
    }
}
