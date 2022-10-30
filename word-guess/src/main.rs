use std::io::BufRead;

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn new(args: &[String]) -> Self {
        assert!(args.len() >= 3);

        let query = args[1].clone();
        let file_path = args[2].clone();

        Config { query, file_path }
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    log::info!("Starting program!");

    println!("Hello, world!");
    log::info!("Reading user input and adding default values!");
    let mut args: Vec<String> = std::env::args().collect();
    // NOTE: No switch and match does not help here because cascading would be cool...
    if args.len() == 1 {
        args.push("Hello there".to_string());
    }
    if args.len() == 2 {
        args.push("input.txt".to_string());
    }

    for (i, arg) in args.iter().enumerate() {
        println!("Arg {i}: {arg}");
    }

    let config = Config::new(&args);
    println!("Searching for '{}' in '{}'", config.query, config.file_path);

    let mut result: Vec<String> = vec![];

    log::info!("Getting the lines!");
    let file = std::fs::File::open(config.file_path)?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        println!("Line: {}", line);

        if line.contains(&config.query) {
            result.push(line);
        }
    }

    println!("\nResults:");
    for line in result {
        println!("{}", line);
    }

    log::info!("Finished.");
    Ok(())
}
