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
    fn simple_command_no_quotes() {
        let command = r#"echo Hello world!"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn simple_command_double_quotes() {
        let command = r#"echo "Hello world!""#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn simple_command_single_quotes() {
        let command = r#"echo 'Hello world!'"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn simple_command_escapes() {
        let command = r#"echo Hello\ world!"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn simple_command_substitution() {
        let command = r#"echo (date)"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn single_pipe() {
        let command = r#"ls | sort"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn multi_pipe() {
        let command = r#"ls | sort | grep rwx"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn single_logic_operation() {
        let command = r#"mkdir testdir && cd testdir"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn multi_logic_operation() {
        let command = r#"mkdir testdir && cd testdir && touch testfile"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn single_redirect() {
        let command = r#"sort < file.txt"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn multi_redirect() {
        let command = r#"sort < unsorted.txt >> sorted.txt"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn special_chars() {
        // TODO: Figure out whether EOF should be part of syntax
        let command = r#"echo \nHello\ world!\nEOF"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn nested_substitution() {
        let command = r#"echo (echo (date))"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn nested_substitution_with_pipe() {
        let command = r#"echo (ls | wc -l)"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn logic_operation_with_substitution() {
        let command = r#"test -d testdir && echo "(date): Directory exists"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn logic_operation_with_redirect() {
        let command = r#"ls fake_dir 2>/dev/null || echo "(date): Directory does not exist"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn double_sided_redirect() {
        let command = r#"command <> file.txt"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn pattern_matching() {
        let command = r#"ls *.txt"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn multi_argument() {
        let command = r#"command --option argument"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn double_quotes_with_escapes() {
        let command = r#"echo "This is a \"test\""#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }

    #[test]
    fn double_quotes_with_special_chars() {
        let command = r#"echo -e "First line\nSecond line"#;
        let parse_result = RushTokenizer::parse(Rule::input_line, command);
    }
}