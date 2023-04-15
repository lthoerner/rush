
// Separators
pub const WHITESPACE: char = ' ';
pub const SEMI: char = ';';
pub const AND: char = '&';

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
            AND,
            SEMI
        ];

        Symbols {
            operators,
            separators
        }
    }
}


