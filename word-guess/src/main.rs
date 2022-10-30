use std::io::BufRead;
use rand::Rng;

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

fn minigrep() -> Result<(), std::io::Error> {
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

fn repeater() -> Result<(), std::io::Error> {
    println!("Write stuff and press <enter> so i can repeat it!!:");
    let mut io_line = String::new();
    std::io::stdin().read_line(&mut io_line)?;
    println!("Read stuff from user input: {}", io_line);

    Ok(())
}

fn some_randoms(rng: &mut rand::rngs::ThreadRng, min: u32, max: u32) -> () {
    let mut randoms: Vec<u32> = Vec::new();
    for _ in 0..10 {
        randoms.push(rng.gen_range(min..=max));
    }
    println!("Some random numbers: {:?}", randoms);
}

fn guessing_game(rng: &mut rand::rngs::ThreadRng) -> Result<(), std::io::Error> {
    let mut io_line = String::new();

    let result_random = rng.gen_range(1..=10);
    loop {
        let guess: i32;
        println!("Guess the number!\nYour guess?");
        io_line.clear();
        std::io::stdin().read_line(&mut io_line)?;
        match io_line.trim_end().parse::<i32>() {
            Ok(x) => {
                guess = x;
            },
            Err(e) => {
                println!("That's just no a number... {}. Context: {:?}", io_line, e);
                continue;
            },
        }

        println!("You guessed {guess}");
        if result_random == guess {
            println!("Right!!!");
            return Ok(());
        } else if guess > result_random {
            println!("Your guess is to high! (Try again!)");
        } else if guess < result_random {
            println!("Your guess is to low! (Try again!)");
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    log::info!("Starting program!");
    println!("Hello, world!");

    minigrep()?;

    match repeater() {
        Err(e) => panic!("Could not repeat... {:?}", e),
        _ => {}
    }

    let mut rng = rand::thread_rng();
    some_randoms(&mut rng, 1, 100);
    match guessing_game(&mut rng) {
        Err(e) => panic!("Guessing game panic! {:?}", e),
        _ => {}
    }

    Ok(())
}
