mod teams;

fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    log::info!("Teams starting up, deciding if server of client!");
    let args: Vec<_> = std::env::args().collect();

    let program_name = match args.len() {
        1 => &args[0],
        2 => &args[1],
        _ => {
            log::info!("Too many args, 2 is the maximum!");
            std::process::exit(1);
        },
    };

    if program_name.contains("client") {
        log::info!("Choose client, so let's go!");
        return teams::client::run();
    } else if program_name.contains("server") {
        log::info!("Choose server, so let's go!");
        return teams::server::run();
    } else {
        log::error!("Could not determine which startup func to run!");
    }

    println!("No config chosen, 'client' or 'server' have to be in the program name!");
    std::process::exit(1);
}
