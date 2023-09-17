use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "tokenizer.pest"]
pub struct RushTokenizer;

struct Arg(String);
struct Args(Vec<Arg>);

struct Command(Arg, Args);
struct OperationRhs(Operator, Command);
struct Operation {
    left: Command,
    right: Vec<OperationRhs>,
}

enum Operator {
    And,
    Or,
    Pipe,
}

enum InputLineTree {

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenizer_test_1() {
        let cmd = "do the thing || ping pong | grep \"response and xyz\" | foo \'bar baz\'";
        let parse_result = RushTokenizer::parse(Rule::input_line, cmd);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn tokenizer_test_2() {
        let cmd = "ping (getip \'google website\') | grep response";
        let parse_result = RushTokenizer::parse(Rule::input_line, cmd);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn tokenizer_test_3() {
        let cmd = "test 1234";
        let parse_result = RushTokenizer::parse(Rule::input_line, cmd);
        println!("{:#?}", parse_result);
    }
}