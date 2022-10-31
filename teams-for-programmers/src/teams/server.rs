use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread::{JoinHandle};

enum MainThreadMessageType {
    Stream(TcpStream),
    CtrlC(()),
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let deserialized_message = super::recv(&mut stream);

        match deserialized_message {
            Ok(request) => match request {
                Some(request) => match request {
                    super::TeamsMessage::NewUser(username) => {
                        log::info!("new user with username {}", username);
                    },
                    super::TeamsMessage::UserExit(username) => {
                        log::info!("User leaves teams. Bye bye {}", username);
                        return;
                    },
                    super::TeamsMessage::Message(m) => {
                        log::info!("New message for user {} with message {}", m.user, m.message);
                    },
                },
                None => {
                    log::warn!("Message type not known!");
                    continue;
                }
            },
            Err(e) => {
                log::error!("Could not read, disconnect {:?}", e);
                return;
            }
        }

        let response = super::TeamsMessage::Message(super::Message{user: "Flo!".to_string(), message: "New message".to_string()});
        if let Err(e) = super::send(&response, &mut stream) {
            log::error!("Could not send, disconnect {:?}", e);
            return;
        }
    }
}

fn setup_ctrlc_handler(sx: &Sender<MainThreadMessageType>) {
    log::info!("Ctrl-c setup");

    let s_ctrlc = sx.clone();
    ctrlc::set_handler(move || {
        s_ctrlc.send(MainThreadMessageType::CtrlC(())).unwrap_or_else(|e| {
            log::error!("{:?}", e);
            std::process::exit(1);
        })
    }).expect("Could not set ctrl-c handler!");
}

fn setup_tcp_listener(sx: &Sender<MainThreadMessageType>) {
    log::info!("Bind to port 7474");
    let s_stream = sx.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:7474").expect("Could not bind!");
    std::thread::spawn(move || {
        // NOTE: Could also do this with non-blocking mode and epoll but we can also just
        // use the channel for this...
        // listener.set_nonblocking(true).expect("Could not switch to non-blocking mode");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    log::info!("new connection!");
                    s_stream.send(MainThreadMessageType::Stream(stream)).expect("Could not send stream over channel!");
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        log::info!("would block, but we do not care we do not use nonblocking at this point");
                        continue;
                    }

                    log::warn!("could not establish connection: {:?}", e);
                }
            }
        }
    });
}

pub fn run() -> Result<(), std::io::Error> {
    log::info!("Server setup...");
    let mut workers: Vec<JoinHandle<()>> = vec![];
    let mut should_shutdown = false;
    let (sx, rx) = std::sync::mpsc::channel::<MainThreadMessageType>();

    setup_ctrlc_handler(&sx);
    setup_tcp_listener(&sx);

    while !should_shutdown {
        let stream = rx.recv().unwrap_or(MainThreadMessageType::CtrlC(()));
        match stream {
            MainThreadMessageType::Stream(stream) => {
                let join_handle = std::thread::spawn(move || {
                    log::info!("handle connection");
                    handle_connection(stream);
                    log::info!("close connection");
                });
                workers.push(join_handle);
            }
            MainThreadMessageType::CtrlC(_) => should_shutdown = true
        }
    }

    log::info!("graceful shutdown. Join all handlers");
    for worker in workers {
        match worker.join() {
            Err(e) => log::error!("Could not join thread: {:?}", e),
            _ => {}
        }
    }

    log::info!("EXIT");
    Ok(())
}