use std::io::Write;
use crossterm::{cursor, ExecutableCommand, QueueableCommand};
use crossterm::event::KeyCode;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use text2art::{BasicFonts, Font, Printer};

enum Command {
    NewMessage(super::TeamsMessage),
    Input(String),
    Reinit,
}

fn setup_username(connection: &mut std::net::TcpStream) -> String {
    println!("Please choose a username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();
    let trimmed_username = username.trim();
    if "" == trimmed_username {
        println!("The username needs to be something, are you trying edge cases here???");
        std::process::exit(1);
    }
    let message = super::TeamsMessage::NewUser(trimmed_username.to_string());

    super::send(&message, connection).unwrap();

    trimmed_username.to_string()
}

pub fn run() -> Result<(), std::io::Error> {
    ctrlc::set_handler(|| {
        std::io::stdout()
            .queue(ResetColor).unwrap()
            .queue(cursor::Show).unwrap()
            .queue(LeaveAlternateScreen).unwrap()
            .flush().unwrap()
        ;
        std::process::exit(0);
    }).expect("Could not register the ctrl-c handler!");

    let font = Font::from_basic(BasicFonts::Big).unwrap();
    let prntr = Printer::with_font(font);
    let teams_logo = prntr.render_text("Teams").unwrap();

    std::io::stdout().execute(EnterAlternateScreen)?;

    std::io::stdout()
        .queue(SetForegroundColor(Color::Blue))?
        .queue(SetBackgroundColor(Color::Black))?
        .queue(Clear(ClearType::All))?
        .queue(cursor::Hide)?
        .queue(cursor::MoveTo(1, 3))?
        .queue(Print(format_args!("{}\n\n{}\n\n{}", teams_logo, "not microsoft (c)", "Loading")))?
        .flush()?
    ;

    for _ in 0..3 {
        print!(". ");
        std::io::stdout().flush()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    std::io::stdout()
        .queue(Clear(ClearType::All))?
        .queue(SetForegroundColor(Color::White))?
        .queue(cursor::Show)?
        .queue(cursor::MoveTo(0, 0))?
        .flush()?
    ;

    let mut connection = std::net::TcpStream::connect("127.0.0.1:7474").expect("Could not establish connection!");

    let username = setup_username(&mut connection);
    std::io::stdout()
        .queue(Clear(ClearType::All))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(Print(format_args!("Hello {}!\n\n", username)))?
        .flush()?
    ;

    connection.set_nonblocking(true).unwrap();
    let connection = std::sync::Arc::new(std::sync::Mutex::new(connection));

    let (sx, rx) = std::sync::mpsc::channel::<Command>();

    let s_read = sx.clone();
    let read_connection_clone = std::sync::Arc::clone(&connection);
    std::thread::spawn(move || {
        loop {
            match super::try_recv(&mut read_connection_clone.lock().unwrap()) {
                Ok(option) => if let Some(message) = option {
                    s_read.send(Command::NewMessage(message)).unwrap();
                },
                Err(_) => s_read.send(Command::Reinit).unwrap(),
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    let s_new_message = sx.clone();
    std::thread::spawn(move || {
        let mut cursor_pos = 0;
        std::io::stdout()
            .execute(cursor::MoveTo(0, crossterm::terminal::size().unwrap().1)).unwrap()
            .execute(Print("> ")).unwrap()
        ;

        loop {
            let mut message = String::new();
            loop {
                match crossterm::event::read() {
                    Ok(event) => match event {
                        crossterm::event::Event::Key(crossterm::event::KeyEvent { code, .. }) => {
                            match code {
                                // TODO: Implement other features!
                                KeyCode::Enter => {
                                    // TODO: Reprint the whole screen as enter screws everthing up! (use tui-rs for cool layouts?)
                                    std::io::stdout()
                                        .execute(cursor::MoveToPreviousLine(1)).unwrap()
                                        .execute(Clear(ClearType::UntilNewLine)).unwrap()
                                        .execute(cursor::MoveToNextLine(1)).unwrap()
                                        .execute(Clear(ClearType::CurrentLine)).unwrap()
                                        .execute(Print("> ")).unwrap()
                                    ;
                                    break;
                                },
                                KeyCode::Char(c) => {
                                    message.push(c);
                                    cursor_pos += 1;
                                },
                                KeyCode::Backspace => {
                                    message.remove(cursor_pos - 1);
                                    cursor_pos -= 1;
                                },
                                _ => {},
                            }
                        },
                        crossterm::event::Event::Paste(string) => {
                            message.push_str(&string);
                        },
                        crossterm::event::Event::Resize(_, new_height) => {
                            std::io::stdout()
                                .execute(Clear(ClearType::CurrentLine)).unwrap()
                                .execute(cursor::MoveTo(0, new_height)).unwrap()
                                .execute(Print(format_args!("> {}", message))).unwrap()
                            ;
                        },
                        _ => {},
                    },
                    Err(e) => {
                        log::error!("Event error {:?}", e);
                        continue;
                    }
                }
            }

            s_new_message.send(Command::Input(message.trim_end().to_string())).unwrap();
            message.clear();
        }
    });

    let mut print_row = 3;

    loop {
        match rx.recv().unwrap() {
            Command::NewMessage(teams_message) => match teams_message {
                super::TeamsMessage::Message(m) => {
                    std::io::stdout()
                        .execute(cursor::SavePosition).unwrap()
                        .execute(cursor::MoveTo(0, print_row)).unwrap()
                        .execute(Print(format_args!("New message!\n'{}': '{}'\n\n", m.user, m.message))).unwrap()
                        .execute(cursor::RestorePosition).unwrap()
                    ;
                    print_row += 1;
                },
                _ => {},
            },
            Command::Input(i) => {
                if i == "exit" {
                    if let Some(err) = super::send(& super::TeamsMessage::UserExit(username.to_string()), &mut connection.lock().unwrap()).err() {
                        log::error!("Could not unregister client! {:?}", err);
                    }
                    println!("Exiting...");
                    break;
                } else {
                    let parts: Vec<&str> = i.split(":").collect();
                    if parts.len() != 2 {
                        std::io::stdout()
                            .execute(cursor::SavePosition).unwrap()
                            .execute(cursor::MoveTo(0, crossterm::terminal::size().unwrap().1 - 2)).unwrap()
                            .execute(Clear(ClearType::UntilNewLine)).unwrap()
                            .execute(Print("!! User: Message <- that's the format. Please try again !!")).unwrap()
                            .execute(cursor::RestorePosition).unwrap()
                        ;
                        continue;
                    }

                    if super::send(& super::TeamsMessage::Message(
                        super::Message{
                            user: parts[0].to_string(),
                            message: parts[1].to_string(),
                        },
                    ), &mut connection.lock().unwrap()).is_ok() {
                        continue;
                    }

                    sx.send(Command::Reinit).unwrap();
                }
            },
            Command::Reinit => {
                let mut connection_ref = connection.lock().unwrap();
                let mut success = false;
                for i in 1..5 {
                    if let Ok(tcp_stream) = std::net::TcpStream::connect("127.0.0.1:7474") {
                        let mut tcp_stream = tcp_stream;
                        let message = super::TeamsMessage::NewUser(username.clone());
                        super::send(&message, &mut tcp_stream).unwrap();
                        tcp_stream.set_nonblocking(true).unwrap();
                        *connection_ref = tcp_stream;
                        success = true;
                        break;
                    }

                    println!("Could not reconnect, trying again in {i} seconds...");
                    std::thread::sleep(std::time::Duration::from_secs(i));
                }

                if success {
                    continue;
                }

                log::info!("Could not reconnect after timout, exiting");
                std::process::exit(0);
            },
        }
    }

    Ok(())
}
