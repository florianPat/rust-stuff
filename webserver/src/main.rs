use std::io::{BufRead, Write};
use std::net::TcpStream;
use std::sync::{mpsc};
use std::sync::mpsc::TryRecvError;

fn ctrlc_channel() -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();
    match ctrlc::set_handler(move || {
        let _ = tx.send(());
    }) {
        Err(e) => panic!("Could not set ctrlc handler! {:?}", e),
        _ => {},
    }

    rx
}

fn stream_channel(listener: std::net::TcpListener) -> mpsc::Receiver<TcpStream> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    log::info!("New connection!");
                    match tx.send(stream) {
                        Err(e) => panic!("Could not send on the channel! {:?}", e),
                        _ => {},
                    }
                },
                Err(e) => log::info!("Connection failed! {:?}", e),
            }
        }
    });

    rx
}

fn main() {
    env_logger::init();
    log::info!("Starting server!");

    let listener = std::net::TcpListener::bind("127.0.0.1:7878").expect("Could not bind, uncool!");

    let ctrlc_event = ctrlc_channel();
    let connection_event = stream_channel(listener);
    let mut workers: Vec<std::thread::JoinHandle<()>> = vec![];

    loop {
        match ctrlc_event.try_recv() {
            Ok(_) => {
                log::info!("ctrl-c received, graceful shutdown...");
                for worker in workers {
                    match worker.join() {
                        Err(e) => log::info!("Could not join thread! {:?}", e),
                        _ => {},
                    }
                }
                break;
            },
            Err(e) => match e {
                TryRecvError::Empty => {},
                TryRecvError::Disconnected => panic!("Why would this happen??"),
            }
        }

        match connection_event.try_recv() {
            Ok(mut stream) => {
                let join_handle = std::thread::spawn(move || {
                    let buf_reader = std::io::BufReader::new(&mut stream);
                    let request: Vec<String> = buf_reader.lines().map(|result| result.unwrap()).take_while(|line| !line.is_empty()).collect();

                    log::info!("Request: {:?}", request);

                    let response = "HTTP/1.1 200 OK\r\n\r\nHello there";

                    stream.write_all(response.as_bytes()).unwrap();
                });

                workers.push(join_handle);
            },
            Err(e) => match e {
                TryRecvError::Empty => {},
                TryRecvError::Disconnected => panic!("Why would this happen??"),
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
