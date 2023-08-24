use super::symbols::{
    Symbols, AMPERSAND, BACKSLASH, DOLLAR, DOUBLE_QUOTE, GREATER_THAN, LESS_THAN, PIPE, SEMICOLON,
    SINGLE_QUOTE, WHITESPACE,
};

pub fn tokenize(input: &str) -> Vec<String> {
    let symbols = Symbols::new();

    let mut curr_token = String::new();
    let mut tokens: Vec<String> = Vec::new();

    let mut characters = input.trim().chars().peekable();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    loop {
        let character = characters.next();

        match character {
            Some(v) => {
                match v {
                    WHITESPACE | AMPERSAND | SEMICOLON => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if (peeked_char == &SEMICOLON && v == SEMICOLON)
                                    || (peeked_char == &AMPERSAND && v == AMPERSAND)
                                {
                                    // clear token, push the operator into the token and advance to the next character
                                    delimit_token(&mut tokens, &mut curr_token);

                                    curr_token.push(v);
                                    curr_token.push(*peeked_char);
                                    characters.next();

                                    delimit_token(&mut tokens, &mut curr_token);
                                } else {
                                    delimit_token(&mut tokens, &mut curr_token);
                                }
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                    PIPE | LESS_THAN | GREATER_THAN => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if symbols
                                    .operators
                                    .iter()
                                    .any(|&i| i == format!("{v}{peeked_char}"))
                                {
                                    // clear token, push the operator into the token and advance to the next character
                                    delimit_token(&mut tokens, &mut curr_token);

                                    curr_token.push(v);
                                    curr_token.push(*peeked_char);
                                    characters.next();

                                    delimit_token(&mut tokens, &mut curr_token);
                                }
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                    SINGLE_QUOTE => {
                        if !in_double_quotes {
                            in_single_quotes = !in_single_quotes;
                        } else {
                            curr_token.push(v);
                        }
                    }
                    DOUBLE_QUOTE => {
                        if !in_single_quotes {
                            in_double_quotes = !in_double_quotes;
                        } else {
                            curr_token.push(v);
                        }
                    }
                    BACKSLASH => {
                        if in_single_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        'dont_skip: {
                            if let Some(peeked_char) = characters.peek() {
                                match *peeked_char {
                                    'n' => curr_token.push('\n'),
                                    't' => curr_token.push('\t'),
                                    'r' => curr_token.push('\r'),
                                    '0' => curr_token.push('\0'),
                                    'a' => curr_token.push('\x07'),
                                    'b' => curr_token.push('\x08'),
                                    'v' => curr_token.push('\x0b'),
                                    'f' => curr_token.push('\x0c'),
                                    'e' => curr_token.push('\x1b'),
                                    SINGLE_QUOTE | DOUBLE_QUOTE | DOLLAR | BACKSLASH => {
                                        curr_token.push(*peeked_char)
                                    }
                                    _ => {
                                        curr_token.push(v);
                                        break 'dont_skip;
                                    }
                                }
                            }

                            characters.next();
                        }
                    }
                    _ => curr_token.push(v),
                }
            }
            None => {
                delimit_token(&mut tokens, &mut curr_token);
                break;
            }
        }
    }

    tokens
}

fn delimit_token(tokens: &mut Vec<String>, curr_token: &mut String) {
    if !curr_token.is_empty() {
        tokens.push(curr_token.clone());
        curr_token.clear();
    }
}
