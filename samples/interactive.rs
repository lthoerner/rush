fn main() {
    println!("Please enter a word: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    println!("You entered: {}", input);
}
