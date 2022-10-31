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

fn try_recv(stream: &TcpStream) -> Result<Option<TeamsMessage>, std::io::Error> {
    let mut reader = std::io::BufReader::new(stream);
    let received_bytes: &[u8];
    match reader.fill_buf() {
        Ok(received) => received_bytes = received,
        Err(e) => return match e.kind() {
            std::io::ErrorKind::WouldBlock => Ok(None),
            std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset => {
                    log::error!("Connection lost!");
                    Err(e)
                },
            _ => Err(e),
        },
    }
    let _len = received_bytes.len();
    let deserialized_message: TeamsMessage = match serde_json::from_slice(&received_bytes) {
        Ok(m) => m,
        Err(e) => {
            if e.classify() == serde_json::error::Category::Eof {
                log::info!("EOF found!");
                return Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, e));
            }
            log::error!("Could not deserialize {:?}", e);
            return Ok(None);
        },
    };
    log::info!("Request: {:?}", deserialized_message);

    Ok(Some(deserialized_message))
}

fn recv(stream: &TcpStream) -> Result<Option<TeamsMessage>, std::io::Error> {
    let mut reader = std::io::BufReader::new(stream);
    let received: &[u8] = match reader.fill_buf() {
        Ok(r) => r,
        Err(e) => return match e.kind() {
            std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset => {
                log::error!("Connection lost!");
                Err(e)
            },
            _ => Err(e),
        },
    };
    let deserialized_message: TeamsMessage = match serde_json::from_slice(&received) {
        Ok(m) => m,
        Err(e) => {
            if e.classify() == serde_json::error::Category::Eof {
                log::info!("EOF found!");
                return Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, e));
            }
            log::error!("Could not deserialize {:?}", e);
            return Ok(None);
        },
    };
    log::info!("Request: {:?}", deserialized_message);

    Ok(Some(deserialized_message))
}

fn send(message: &TeamsMessage, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let serialized_response = serde_json::to_string(&message).expect("Could not serialize response!");
    log::info!("Send response: {}", serialized_response);
    let bytes = serialized_response.as_bytes();
    match stream.write(bytes) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset => {
                log::error!("Connection lost!");
                Err(e)
            },
            _ => Err(e),
        }
    }
}
