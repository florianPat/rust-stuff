use std::io::{BufRead, Write};
use std::net::TcpStream;
use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Message {
    user: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum TeamsMessage  {
    NewUser(String),
    UserExit(String),
    Message(Message),
}

fn try_recv(stream: &mut TcpStream) -> Option<TeamsMessage> {
    let mut reader = std::io::BufReader::new(stream);
    let received_bytes: &[u8];
    match reader.fill_buf() {
        Ok(received) => received_bytes = received,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                log::error!("Could not get response!");
            }

            return None;
        }
    }
    let _len = received_bytes.len();
    let deserialized_message: TeamsMessage = serde_json::from_slice(&received_bytes).expect("Could not deserialize!");
    log::info!("Request: {:?}", deserialized_message);

    Some(deserialized_message)
}

fn recv(stream: &mut TcpStream) -> TeamsMessage {
    let mut reader = std::io::BufReader::new(stream);
    let received: &[u8] = reader.fill_buf().unwrap();
    let deserialized_message: TeamsMessage = serde_json::from_slice(&received).expect("Could not deserialize!");
    log::info!("Request: {:?}", deserialized_message);

    deserialized_message
}

fn send(message: &TeamsMessage, stream: &mut TcpStream) {
    let serialized_response = serde_json::to_string(&message).expect("Could not serialize response!");
    log::info!("Send response: {}", serialized_response);
    let bytes = serialized_response.as_bytes();
    stream.write(bytes).expect("Could not send response!");
    stream.flush().unwrap();
}
