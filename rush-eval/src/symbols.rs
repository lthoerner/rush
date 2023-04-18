
// Separators
pub const WHITESPACE: char = ' ';
pub const SEMI: char = ';';
pub const AMP: char = '&';
pub const GREAT: char = '>';
pub const LESS: char = '<';
pub const PIPE: char = '|';
pub const SINGLE_QUOTE: char = '\'';
pub const DOUBLE_QUOTE: char = '"';
pub const BACKSLASH: char = '\\';
pub const DOLLAR: char = '$';

//Operators
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
            AND_IF,
            OR_IF,
            DSEMI,
            DLESS,
            DGREAT,
            LESSAND,
            GREATAND,
            LESSGREAT,
            CLOBBER
        ];

        let separators = vec![
            WHITESPACE,
            AMP,
            SEMI
        ];

        Symbols {
            operators,
            separators
        }
    }
}


