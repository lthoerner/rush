
// lexical scanner
// tokenize input
// parse tokens and create AST


// use std::str::Chars;
fn tokenizer(input: &String) -> Vec<String> {
    let mut curr_token = String::new();
    let mut tokens: Vec<String> = Vec::new();

    let mut characters = input.trim().chars().peekable();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    loop {
        let character = characters.next();
        // handle when a single quote is encountered
        // handle when a double quote is encountered
        // handle when a \ is encountered
        // handle when a \0 a \e or a \x is encountered (most likely the start of a ANSII sequence which is also between double qoutes
        // handle \r, \t and \n as separate tokens
        // ANSI sequences can be handled as separate tokens

        match character {
            Some(v) => {
                match v {
                    '\'' => {
                        in_single_quotes = !in_single_quotes;
                    }
                    '"' => {
                        in_double_quotes = !in_double_quotes;
                    }
                    '\\' => {
                        match characters.peek() {
                            Some(peeked_char) => {
                                if peeked_char == &'n' || peeked_char == &'0' {
                                    if in_single_quotes || in_double_quotes {
                                        // clear token and push the newline and backslash into the token
                                        tokens.push(curr_token.clone());
                                        curr_token.clear();

                                        curr_token.push(v);
                                        curr_token.push(*peeked_char);
                                        characters.next();

                                        tokens.push(curr_token.clone());
                                        curr_token.clear();
                                    } else {
                                        continue;
                                    }
                                }
                            },
                            None => {
                                continue;
                            }
                        }
                    },
                    ' ' => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v)
                        } else if !curr_token.is_empty() {
                            tokens.push(curr_token.clone());
                            curr_token.clear();
                        }
                    },
                    _ => curr_token.push(v)
                }
            },
            None => {
                tokens.push(curr_token.clone());
                curr_token.clear();
                break;
            }
        }
    }

    println!("{:?}", tokens);
    return tokens
}

pub fn tokenize(input: &String) -> (String, Vec<String>) {
    tokenizer(input);
    return (String::from("test"), vec![String::from("testArg")]);
}

