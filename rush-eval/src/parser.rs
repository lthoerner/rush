use crate::symbols::{WHITESPACE, AND, SEMI, Symbols};

fn tokenizer(input: &String) -> Vec<String> {
    let mut curr_token = String::new();
    let mut tokens: Vec<String> = Vec::new();

    let mut characters = input.trim().chars().peekable();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    let symbols = Symbols::new();

    loop {
        let character = characters.next();

        match character {
            Some(v) => {
                match v {
                    WHITESPACE | AND | SEMI => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if (peeked_char == &';' && v == SEMI) || (peeked_char == &'&' && v == AND)  {
                                    // clear token, push the operator into the token and advance to the next character
                                    curr_token.push(v);
                                    curr_token.push(*peeked_char);
                                    characters.next();

                                    delimit_token(&mut tokens, &mut curr_token);
                                } else {
                                    if !curr_token.is_empty() {
                                        delimit_token(&mut tokens, &mut curr_token);
                                    }
                                }
                            },
                            None => {
                                continue;
                            }
                        }
                    },
                    '|' | '<' | '>' => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if symbols.operators.iter().any(|&i| i == format!("{v}{peeked_char}")) {
                                    delimit_token(&mut tokens, &mut curr_token);

                                    curr_token.push(v);
                                    curr_token.push(*peeked_char);
                                    characters.next();

                                    delimit_token(&mut tokens, &mut curr_token);
                                }
                            },
                            None => {
                                continue;
                            }
                        }
                    }
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
                                        delimit_token(&mut tokens, &mut curr_token);

                                        curr_token.push(v);
                                        curr_token.push(*peeked_char);
                                        characters.next();

                                        delimit_token(&mut tokens, &mut curr_token);
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
                    _ => curr_token.push(v)
                }
            },
            None => {
                delimit_token(&mut tokens, &mut curr_token);
                break;
            }
        }
    }

    println!("{:?}", tokens);
    return tokens
}

fn delimit_token(tokens: &mut Vec<String>, curr_token: &mut String) {
    tokens.push(curr_token.clone());
    curr_token.clear();
}

pub fn tokenize(input: &String) -> (String, Vec<String>) {
    tokenizer(input);
    return (String::from("test"), vec![String::from("testArg")]);
}

