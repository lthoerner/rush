use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "tokenizer.pest"]
pub struct RushTokenizer;

#[derive(PartialEq, Debug)]
enum Expression {
    Logic(Box<LogicOperation>),
    Redirect(Box<RedirectOperation>),
    Command(Box<Command>),
}

// Redirect operations and single commands are grouped with a higher precedence than
// logical operations, so they are grouped together and called `SubOperation`s
#[derive(PartialEq, Debug)]
enum SubOperation {
    Command(Box<Command>),
    Redirect(Box<RedirectOperation>),
}

#[derive(PartialEq, Debug)]
struct LogicOperationRhs {
    operator: LogicOperator,
    right_subop: SubOperation,
}

#[derive(PartialEq, Debug)]
struct LogicOperation {
    left_subop: SubOperation,
    right: LogicOperationRhs,
    continued: Vec<LogicOperationRhs>,
}

#[derive(PartialEq, Debug)]
struct RedirectOperationRhs {
    operator: RedirectOperator,
    right_command: Command,
}

#[derive(PartialEq, Debug)]
struct RedirectOperation {
    left_command: Command,
    right: RedirectOperationRhs,
    continued: Vec<RedirectOperationRhs>,
}

#[derive(PartialEq, Debug)]
struct Command {
    name: StringArg,
    args: Args,
}

#[derive(PartialEq, Debug)]
struct Args(Vec<Arg>);
#[derive(PartialEq, Debug)]
enum Arg {
    Substitution(Box<SubstitutionArg>),
    String(StringArg),
}

#[derive(PartialEq, Debug)]
struct SubstitutionArg(SubOperation);
#[derive(PartialEq, Debug)]
struct StringArg(String);

#[derive(PartialEq, Debug)]
enum LogicOperator {
    And,
    Or,
}

#[derive(PartialEq, Debug)]
enum RedirectOperator {
    DoubleRedirect,
    LeftRedirect,
    RightRedirect,
    Pipe,
}

fn parse_expression(line: &str) -> Expression {
    let mut pairs = RushTokenizer::parse(Rule::expression, line).unwrap();
    let expr_content = pairs.next().unwrap().into_inner().next().unwrap();
    assert!(matches!(
        expr_content.as_rule(),
        Rule::command | Rule::redirect_operation | Rule::logic_operation
    ));

    match expr_content.as_rule() {
        Rule::command => Expression::Command(Box::new(parse_command(expr_content))),
        Rule::redirect_operation => {
            Expression::Redirect(Box::new(parse_redirect_operation(expr_content)))
        }
        Rule::logic_operation => Expression::Logic(Box::new(parse_logic_operation(expr_content))),
        _ => unreachable!(),
    }
}

fn parse_suboperation(subop: Pair<Rule>) -> SubOperation {
    assert_eq!(subop.as_rule(), Rule::suboperation);
    let mut subop_content = subop.into_inner().next().unwrap();
    assert!(matches!(
        subop_content.as_rule(),
        Rule::command | Rule::redirect_operation
    ));

    match subop_content.as_rule() {
        Rule::command => SubOperation::Command(Box::new(parse_command(subop_content))),
        Rule::redirect_operation => {
            SubOperation::Redirect(Box::new(parse_redirect_operation(subop_content)))
        }
        _ => unreachable!(),
    }
}

fn parse_command(command: Pair<Rule>) -> Command {
    assert_eq!(command.as_rule(), Rule::command);
    let mut command_content = command.into_inner();
    let name = parse_string_arg(command_content.next().unwrap());
    let mut args = Vec::new();
    for arg in command_content {
        match arg.as_rule() {
            Rule::string_argument => args.push(Arg::String(parse_string_arg(arg))),
            Rule::substitution_argument => {
                let subop = parse_substitution_arg(arg);
                args.push(Arg::Substitution(Box::new(subop)));
            }
            _ => unreachable!(),
        }
    }

    Command {
        name,
        args: Args(args),
    }
}

fn parse_logic_operation(operation: Pair<Rule>) -> LogicOperation {
    fn parse_rhs(rhs: Pair<Rule>) -> LogicOperationRhs {
        assert_eq!(rhs.as_rule(), Rule::logic_operation_rhs);
        let mut rhs_content = rhs.into_inner();
        let operator = parse_operator(rhs_content.next().unwrap());
        let right_subop = parse_suboperation(rhs_content.next().unwrap());

        LogicOperationRhs {
            operator,
            right_subop,
        }
    }

    fn parse_operator(operator: Pair<Rule>) -> LogicOperator {
        assert_eq!(operator.as_rule(), Rule::logic_operator);
        match operator.as_str() {
            "&&" => LogicOperator::And,
            "||" => LogicOperator::Or,
            _ => unreachable!(),
        }
    }

    assert_eq!(operation.as_rule(), Rule::logic_operation);
    let mut operation_content = operation.into_inner();
    let left_subop = parse_suboperation(operation_content.next().unwrap());
    let mut continued = Vec::new();
    let mut right = parse_rhs(operation_content.next().unwrap());
    for rhs in operation_content {
        continued.push(parse_rhs(rhs));
    }

    LogicOperation {
        left_subop,
        right,
        continued,
    }
}

fn parse_redirect_operation(operation: Pair<Rule>) -> RedirectOperation {
    fn parse_rhs(rhs: Pair<Rule>) -> RedirectOperationRhs {
        assert_eq!(rhs.as_rule(), Rule::redirect_operation_rhs);
        let mut rhs_content = rhs.into_inner();
        let operator = parse_operator(rhs_content.next().unwrap());
        let right_command = parse_command(rhs_content.next().unwrap());

        RedirectOperationRhs {
            operator,
            right_command,
        }
    }

    fn parse_operator(operator: Pair<Rule>) -> RedirectOperator {
        assert_eq!(operator.as_rule(), Rule::redirect_operator);
        match operator.as_str() {
            ">>" => RedirectOperator::DoubleRedirect,
            "<" => RedirectOperator::LeftRedirect,
            ">" => RedirectOperator::RightRedirect,
            "|" => RedirectOperator::Pipe,
            _ => unreachable!(),
        }
    }

    assert_eq!(operation.as_rule(), Rule::redirect_operation);
    let mut operation_content = operation.into_inner();
    let left_command = parse_command(operation_content.next().unwrap());
    let mut continued = Vec::new();
    let mut right = parse_rhs(operation_content.next().unwrap());
    for rhs in operation_content {
        continued.push(parse_rhs(rhs));
    }

    RedirectOperation {
        left_command,
        right,
        continued,
    }
}

fn parse_substitution_arg(arg: Pair<Rule>) -> SubstitutionArg {
    assert_eq!(arg.as_rule(), Rule::substitution_argument);
    let mut subop = arg.into_inner().next().unwrap();
    assert_eq!(subop.as_rule(), Rule::suboperation);

    SubstitutionArg(parse_suboperation(subop))
}

fn parse_string_arg(arg: Pair<Rule>) -> StringArg {
    assert_eq!(arg.as_rule(), Rule::string_argument);
    let arg_content = arg.into_inner().next().unwrap();
    assert!(matches!(
        arg_content.as_rule(),
        Rule::nonliteral_argument | Rule::nonliteral_arguments | Rule::literal_arguments
    ));

    StringArg(arg_content.as_str().to_owned())
}

fn debug_expression(line: &str) {
    let parse_result = RushTokenizer::parse(Rule::expression, line);
    println!("{:#?}", parse_result);
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! arg {
        ($arg:literal) => {
            Arg::String(StringArg($arg.to_owned()))
        };
    }

    #[test]
    fn simple_command_no_quotes() {
        let command = r#"echo Hello world!"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);

        let expected = Expression::Command(Box::new(Command {
            name: StringArg("echo".to_owned()),
            args: Args(vec![arg!("Hello"), arg!("world!")]),
        }));

        assert_eq!(parse_result, expected);
    }

    #[test]
    fn simple_command_double_quotes() {
        let command = r#"echo "Hello world!""#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);

        let expected = Expression::Command(Box::new(Command {
            name: StringArg("echo".to_owned()),
            args: Args(vec![arg!("Hello world!")]),
        }));

        assert_eq!(parse_result, expected);
    }

    #[test]
    fn simple_command_single_quotes() {
        let command = r#"echo 'Hello world!'"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);

        let expected = Expression::Command(Box::new(Command {
            name: StringArg("echo".to_owned()),
            args: Args(vec![arg!("Hello world!")]),
        }));

        assert_eq!(parse_result, expected);
    }

    #[test]
    fn simple_command_escapes() {
        let command = r#"echo Hello\ world!"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn simple_command_substitution() {
        let command = r#"echo *(date)"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn single_pipe() {
        let command = r#"ls | sort"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn multi_pipe() {
        let command = r#"ls | sort | grep rwx"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn single_logic_operation() {
        let command = r#"mkdir testdir && cd testdir"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn multi_logic_operation() {
        let command = r#"mkdir testdir && cd testdir && touch testfile"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn single_redirect() {
        let command = r#"sort < file.txt"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn multi_redirect() {
        let command = r#"sort < unsorted.txt >> sorted.txt"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn special_chars() {
        let command = r#"echo \nHello\ world!\n"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn nested_substitution() {
        let command = r#"echo *(echo *(date))"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn nested_substitution_with_pipe() {
        let command = r#"echo *(ls | wc -l)"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn logic_operation_with_substitution() {
        // ? Should substitutions be supported inside double quotes?
        let command = r#"test -d testdir && echo "*(date): Directory exists""#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn logic_operation_with_redirect() {
        let command = r#"ls fake_dir 2>/dev/null || echo "*(date): Directory does not exist""#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn double_sided_redirect() {
        let command = r#"command <> file.txt"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn pattern_matching() {
        let command = r#"ls *.txt"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn multi_argument() {
        let command = r#"command --option argument"#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn double_quotes_with_escapes() {
        let command = r#"echo "This is a \"test\""#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }

    #[test]
    fn double_quotes_with_special_chars() {
        let command = r#"echo -e "First line\nSecond line""#;
        let parse_result = parse_expression(command);
        println!("{:#?}", parse_result);
    }
}
