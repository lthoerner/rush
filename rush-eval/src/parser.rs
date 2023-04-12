// Splits arguments by spaces, taking quotes into account
// $ This is a temporary solution, and will be replaced by a proper parser
pub fn tokenize(line: &String) -> (String, Vec<String>) {
    let line = line.trim();
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => {
                if in_quotes {
                    args.push(current_arg);
                    current_arg = String::new();
                }

                in_quotes = !in_quotes;
            }
            ' ' => {
                if in_quotes {
                    current_arg.push(c);
                } else {
                    args.push(current_arg);
                    current_arg = String::new();
                }
            }
            _ => current_arg.push(c),
        }
    }

    if args.is_empty() {
        return (current_arg, Vec::new());
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    (args.remove(0), args)
}
