use std::net::TcpStream;

enum Command {
    NewMessage(super::TeamsMessage),
    Input(String),
}

fn setup_username(connection: &mut TcpStream) -> String {
    println!("Please choose a username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();
    let trimmed_username = username.trim_end();
    let message = super::TeamsMessage::NewUser(trimmed_username.to_string());

    super::send(&message, connection);

    trimmed_username.to_string()
}

pub fn run() -> Result<(), std::io::Error> {
    let mut connection = std::net::TcpStream::connect("127.0.0.1:7474").expect("Could not establish connection!");

    let username = setup_username(&mut connection);

    connection.set_nonblocking(true).unwrap();
    let connection = std::sync::Arc::new(std::sync::Mutex::new(connection));
    let stdin = std::io::stdin();

    let (sx, rx) = std::sync::mpsc::channel::<Command>();

    let s_read = sx.clone();
    let read_connection_clone = std::sync::Arc::clone(&connection);
    std::thread::spawn(move || {
        loop {
            if let Some(message) = super::try_recv(&mut read_connection_clone.lock().unwrap()) {
                s_read.send(Command::NewMessage(message)).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    let s_new_message = sx.clone();
    std::thread::spawn(move || {
        loop {
            let mut message = String::new();
            stdin.read_line(&mut message).unwrap();
            s_new_message.send(Command::Input(message.trim_end().to_string())).unwrap();
        }
    });

    loop {
        match rx.recv().unwrap() {
            Command::NewMessage(teams_message) => match teams_message {
                super::TeamsMessage::Message(m) => println!("New message: {}:: {}", m.user, m.message),
                _ => {},
            },
            Command::Input(i) => {
                if i == "exit" {
                    super::send(& super::TeamsMessage::UserExit(username.to_string()), &mut connection.lock().unwrap());
                    println!("Exiting...");
                    break;
                } else {
                    let parts: Vec<&str> = i.split(":").collect();
                    if parts.len() != 2 {
                        println!("User: Message <- that's the format!. Please try again!");
                        continue;
                    }

                    super::send(& super::TeamsMessage::Message(
                        super::Message{
                            user: parts[0].to_string(),
                            message: parts[1].to_string(),
                        },
                    ), &mut connection.lock().unwrap());
                }
            },
        }
    }

    Ok(())
}
