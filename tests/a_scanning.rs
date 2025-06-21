mod util;

use indoc::indoc;
use util::run_tokenize;
use util::SUCCESS;

#[test]
fn empty_file() {
    let input = "";
    let expected = indoc! {"
        EOF  null
    "};
    run_tokenize(input, expected, 0);
}

#[test]
fn parentheses() {
    let input1 = "(";
    let expected1 = indoc! {"
        LEFT_PAREN ( null
        EOF  null
    "};
    
    let input2 = "))";
    let expected2 = indoc! {"
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        EOF  null
    "};
    
    let input3 = "))())";
    let expected3 = indoc! {"
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        LEFT_PAREN ( null
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        EOF  null
    "};
    
    let input4 = ")((())(";
    let expected4 = indoc! {"
        RIGHT_PAREN ) null
        LEFT_PAREN ( null
        LEFT_PAREN ( null
        LEFT_PAREN ( null
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        LEFT_PAREN ( null
        EOF  null
    "};

    run_tokenize(input1, expected1, SUCCESS);
    run_tokenize(input2, expected2, SUCCESS);
    run_tokenize(input3, expected3, SUCCESS);
    run_tokenize(input4, expected4, SUCCESS);
}

#[test]
fn braces() {
    let input1 = "}";
    let expected1 = indoc! {"
        RIGHT_BRACE } null
        EOF  null
    "};

    let input2 = "{{}}";
    let expected2 = indoc! {"
        LEFT_BRACE { null
        LEFT_BRACE { null
        RIGHT_BRACE } null
        RIGHT_BRACE } null
        EOF  null
    "};

    let input3 = "{}}{}";
    let expected3 = indoc! {"
        LEFT_BRACE { null
        RIGHT_BRACE } null
        RIGHT_BRACE } null
        LEFT_BRACE { null
        RIGHT_BRACE } null
        EOF  null
    "};

    let input4 = "){)}}({";
    let expected4 = indoc! {"
        RIGHT_PAREN ) null
        LEFT_BRACE { null
        RIGHT_PAREN ) null
        RIGHT_BRACE } null
        RIGHT_BRACE } null
        LEFT_PAREN ( null
        LEFT_BRACE { null
        EOF  null
    "};

    run_tokenize(input1, expected1, SUCCESS);
    run_tokenize(input2, expected2, SUCCESS);
    run_tokenize(input3, expected3, SUCCESS);
    run_tokenize(input4, expected4, SUCCESS);
}