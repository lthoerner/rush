use crate::Token;

type RawCommand = String;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Ctx {
    pub is_quote: bool,
    pub sequence: Vec<Sequence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sequence {
    Pipe {
        // command_left | command_right
        items: Vec<RawCommand>,
    },
    GreaterThanItem {
        // command_left > or >> command_right
        token: Token,
        command_left: RawCommand,
        command_right: RawCommand,
    },
}

impl Ctx {
    pub fn new() -> Self {
        Self {
            is_quote: false,
            sequence: Vec::new(),
        }
    }
}
