mod ctx;

use ctx::{Ctx, Sequence};
use logos::{Lexer, Logos};

#[derive(Logos, Clone, Default, Debug, Eq, PartialEq)]
pub enum Token {
    #[token("#")]
    Comment,
    #[token("|")]
    Pipe,
    #[token("||")]
    Or,
    #[token("&&")]
    And,
    #[token("\\")]
    Escape,
    #[token("&")]
    Ampersand,
    #[token(">")]
    GreaterThan,
    #[token(r#"'"#)]
    SingleQuote,
    #[token(r#"""#)]
    DoubleQuote,
    #[token(">>")]
    DoubleGreaterThan,
    #[regex("[a-zA-z0-9]+", item)]
    Item(String),
    #[error]
    #[default]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub fn lex(raw_command: &str) -> Vec<ctx::Sequence> {
    let lexer = Token::lexer(raw_command);
    let mut lexed = lexer.clone().spanned();
    let mut ctx = Ctx::new();

    while let Some((token, span)) = lexed.next() {
        if let Token::Item(command) = token {
            if let Some((token, _)) = lexed.next() {
                if Token::GreaterThan == token || Token::DoubleGreaterThan == token {
                    if let Some((_, next_span)) = lexed.next() {
                        let next_command = raw_command[next_span].to_owned();

                        ctx.sequence.push(Sequence::GreaterThanItem {
                            token: token.clone(),
                            command_left: command,
                            command_right: next_command,
                        });
                    }
                }

                if Token::Pipe == token {
                    let next_command = raw_command[span].to_owned();

                    if let Some(Sequence::Pipe { items }) = ctx.sequence.last_mut() {
                        items.push(next_command);
                    } else {
                        ctx.sequence.push(Sequence::Pipe {
                            items: vec![next_command],
                        })
                    }
                }
            }
        }
    }

    ctx.sequence
}

fn item(lex: &mut Lexer<Token>) -> Option<String> {
    Some(lex.slice().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_lexing() {
        let lexed = lex(r#"test | test1 | test2 | test3 | test4 >> not_a_test"#);

        assert_eq!(
            lexed,
            vec![
                Sequence::Pipe {
                    items: vec![
                        String::from("test"),
                        String::from("test1"),
                        String::from("test2"),
                        String::from("test3")
                    ]
                },
                Sequence::GreaterThanItem {
                    command_left: String::from("test4"),
                    token: Token::DoubleGreaterThan,
                    command_right: String::from("not_a_test")
                }
            ]
        )
    }
}
