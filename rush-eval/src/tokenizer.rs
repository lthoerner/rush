use crate::symbols::{WHITESPACE, AMP, SEMI, PIPE, LESS, GREAT, SINGLE_QUOTE, DOUBLE_QUOTE, BACKSLASH, Symbols, DOLLAR};

pub fn tokenize(input: &String) -> Vec<String> {
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
                    WHITESPACE | AMP | SEMI => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if (peeked_char == &SEMI && v == SEMI) || (peeked_char == &AMP && v == AMP)  {
                                    // clear token, push the operator into the token and advance to the next character
                                    delimit_token(&mut tokens, &mut curr_token);

                                    curr_token.push(v);
                                    curr_token.push(*peeked_char);
                                    characters.next();

                                    delimit_token(&mut tokens, &mut curr_token);
                                } else {
                                    delimit_token(&mut tokens, &mut curr_token);
                                }
                            },
                            None => {
                                continue;
                            }
                        }
                    },
                    PIPE | LESS | GREAT => {
                        if in_single_quotes || in_double_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if symbols.operators.iter().any(|&i| i == format!("{v}{peeked_char}")) {
                                    // clear token, push the operator into the token and advance to the next character
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
                    },
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
                        if !in_single_quotes && !in_double_quotes {
                            continue;
                        }

                        if in_single_quotes {
                            curr_token.push(v);
                            continue;
                        }

                        match characters.peek() {
                            Some(peeked_char) => {
                                if peeked_char == &'n' {
                                    curr_token.push(v);
                                } else if peeked_char == &SINGLE_QUOTE || peeked_char == &DOLLAR || peeked_char == &DOUBLE_QUOTE || peeked_char == &BACKSLASH {
                                    curr_token.push(*peeked_char);
                                    characters.next();
                                    continue;
                                } else {
                                    curr_token.push(v);
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

    return tokens
}

fn delimit_token(tokens: &mut Vec<String>, curr_token: &mut String) {
    if !curr_token.is_empty() {
        tokens.push(curr_token.clone());
        curr_token.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_correct_token_with_quoted_backslash_special_chars() {
        //given
        let input = String::from("print \"test\\$test\"");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("print"),
            String::from("test$test"),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_token_with_non_quoted_backslash() {
        //given
        let input = String::from("\\0");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("0"),
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_token_with_and_operator() {
        //given
        let input = String::from("ls && ls -a");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("ls"),
            String::from("&&"),
            String::from("ls"),
            String::from("-a")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_token_with_or_operator() {
        //given
        let input = String::from("ls || ls -a");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("ls"),
            String::from("||"),
            String::from("ls"),
            String::from("-a")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_token_with_semi_operator() {
        //given
        let input = String::from("ls ;; ls -a");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("ls"),
            String::from(";;"),
            String::from("ls"),
            String::from("-a")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_token_with_greater_and_less_operator() {
        //given
        let input = String::from("ls << ls -a >> mkdir");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("ls"),
            String::from("<<"),
            String::from("ls"),
            String::from("-a"),
            String::from(">>"),
            String::from("mkdir")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_tokens_with_double_quotes() {
        //given
        let input = String::from("print \"print my text\"");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("print"),
            String::from("print my text")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_tokens_with_double_quotes_and_special_operators() {
        //given
        let input = String::from("print \"print&&  my text;; with< operators|<\"");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("print"),
            String::from("print&&  my text;; with< operators|<")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_tokens_with_double_quotes_and_backslash() {
        //given
        let input = String::from("print \"print\n  my\r text\"");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("print"),
            String::from("print\n  my\r text")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_tokens_with_double_quotes_and_single_quote() {
        //given
        let input = String::from("print \"print' my text\"");

        //when
        let tokens = tokenize(&input);

        //then
        let expected = vec![
            String::from("print"),
            String::from("print' my text")
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn return_correct_tokens_with_unquoted_backslash() {
        //given
        let input = String::from("print\\nls");

        //when
        let tokens = tokenize(&input);
        println!("tokens: {:?}", tokens);

        //then
        let expected = vec![
            String::from("printnls")
        ];
        assert_eq!(tokens, expected);
    }
}