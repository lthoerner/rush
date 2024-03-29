// Separator tokens
pub const WHITESPACE: char = ' ';
pub const SEMICOLON: char = ';';
pub const AMPERSAND: char = '&';
pub const GREATER_THAN: char = '>';
pub const LESS_THAN: char = '<';
pub const PIPE: char = '|';
pub const SINGLE_QUOTE: char = '\'';
pub const DOUBLE_QUOTE: char = '"';
pub const BACKSLASH: char = '\\';
pub const DOLLAR: char = '$';

// Operator tokens
pub const AND_IF: &str = "&&";
pub const OR_IF: &str = "||";
pub const DSEMI: &str = ";;";
pub const DLESS: &str = "<<";
pub const DGREAT: &str = ">>";
pub const LESSAND: &str = "<&";
pub const GREATAND: &str = ">&";
pub const LESSGREAT: &str = "<>";
pub const CLOBBER: &str = ">|";

pub struct Symbols<'a> {
    pub operators: Vec<&'a str>,
    pub separators: Vec<char>,
}

impl Symbols<'_> {
    pub fn new() -> Self {
        let operators = vec![
            AND_IF, OR_IF, DSEMI, DLESS, DGREAT, LESSAND, GREATAND, LESSGREAT, CLOBBER,
        ];

        let separators = vec![WHITESPACE, AMPERSAND, SEMICOLON];

        Symbols {
            operators,
            separators,
        }
    }
}
