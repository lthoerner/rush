fn main() {
    use std::time::Duration;
    use std::thread::sleep;
    use std::env::args;

    let args: Vec<String> = args().collect();
    if args.len() < 2 { exit(); }

    let millis = match args[1].parse::<u64>() {
        Ok(millis) => millis,
        Err(_) => exit(),
    };

    let sleep_duration = Duration::from_millis(millis);

    println!("1");
    sleep(sleep_duration);
    eprintln!("2");
    sleep(sleep_duration);
    println!("3");
    sleep(sleep_duration);
    eprintln!("4");
    sleep(sleep_duration);
    println!("5");
    sleep(sleep_duration);
    eprintln!("6");
    sleep(sleep_duration);
    println!("7");
    sleep(sleep_duration);
    eprintln!("8");
    sleep(sleep_duration);
    println!("9");
    sleep(sleep_duration);
    eprintln!("10");
}

fn exit() -> ! {
    eprintln!("Usage: printdelay <millis>");
    std::process::exit(1);
}
