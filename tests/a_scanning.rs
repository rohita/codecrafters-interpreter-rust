mod util;

use indoc::indoc;
use util::run_tokenize;
use util::{SUCCESS, BUILD_ERROR, NO_ERROR};

#[test]
fn empty_file() {
    let input = "";
    let expected = indoc! {"
        EOF  null
    "};
    run_tokenize(input, expected, NO_ERROR, SUCCESS);
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

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, NO_ERROR, SUCCESS);
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

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, NO_ERROR, SUCCESS);
}

#[test]
fn single_chars() {
    let input1 = "+-";
    let expected1 = indoc! {"
        PLUS + null
        MINUS - null
        EOF  null
    "};

    let input2 = "++--**..,,;;";
    let expected2 = indoc! {"
        PLUS + null
        PLUS + null
        MINUS - null
        MINUS - null
        STAR * null
        STAR * null
        DOT . null
        DOT . null
        COMMA , null
        COMMA , null
        SEMICOLON ; null
        SEMICOLON ; null
        EOF  null
    "};

    let input3 = "+;*-.*;";
    let expected3 = indoc! {"
        PLUS + null
        SEMICOLON ; null
        STAR * null
        MINUS - null
        DOT . null
        STAR * null
        SEMICOLON ; null
        EOF  null
    "};

    let input4 = "({.;*,-})";
    let expected4 = indoc! {"
        LEFT_PAREN ( null
        LEFT_BRACE { null
        DOT . null
        SEMICOLON ; null
        STAR * null
        COMMA , null
        MINUS - null
        RIGHT_BRACE } null
        RIGHT_PAREN ) null
        EOF  null
    "};

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, NO_ERROR, SUCCESS);
}

#[test]
fn lexical_errors() {
    let input1 = "@";
    let expected1 = indoc! {"
        EOF  null
    "};
    let error1 = indoc! {"
        [line 1] Error: Unexpected character: @
    "};

    let input2 = ",.$(#";
    let expected2 = indoc! {"
        COMMA , null
        DOT . null
        LEFT_PAREN ( null
        EOF  null
    "};
    let error2 = indoc! {"
        [line 1] Error: Unexpected character: $
        [line 1] Error: Unexpected character: #
    "};

    let input3 = "@%%#$";
    let expected3 = indoc! {"
        EOF  null
    "};
    let error3 = indoc! {"
        [line 1] Error: Unexpected character: @
        [line 1] Error: Unexpected character: %
        [line 1] Error: Unexpected character: %
        [line 1] Error: Unexpected character: #
        [line 1] Error: Unexpected character: $
    "};

    let input4 = "{(;#+*-%@)}";
    let expected4 = indoc! {"
        LEFT_BRACE { null
        LEFT_PAREN ( null
        SEMICOLON ; null
        PLUS + null
        STAR * null
        MINUS - null
        RIGHT_PAREN ) null
        RIGHT_BRACE } null
        EOF  null
    "};
    let error4 = indoc! {"
        [line 1] Error: Unexpected character: #
        [line 1] Error: Unexpected character: %
        [line 1] Error: Unexpected character: @
    "};

    run_tokenize(input1, expected1, error1, BUILD_ERROR);
    run_tokenize(input2, expected2, error2, BUILD_ERROR);
    run_tokenize(input3, expected3, error3, BUILD_ERROR);
    run_tokenize(input4, expected4, error4, BUILD_ERROR);
}

#[test]
fn equal() {
    let input1 = "=";
    let expected1 = indoc! {"
        EQUAL = null
        EOF  null
    "};

    let input2 = "==";
    let expected2 = indoc! {"
        EQUAL_EQUAL == null
        EOF  null
    "};

    let input3 = "({=}){==}";
    let expected3 = indoc! {"
        LEFT_PAREN ( null
        LEFT_BRACE { null
        EQUAL = null
        RIGHT_BRACE } null
        RIGHT_PAREN ) null
        LEFT_BRACE { null
        EQUAL_EQUAL == null
        RIGHT_BRACE } null
        EOF  null
    "};

    let input4 = "((@$#%=))";
    let expected4 = indoc! {"
        LEFT_PAREN ( null
        LEFT_PAREN ( null
        EQUAL = null
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        EOF  null
    "};
    let error4 = indoc! {"
        [line 1] Error: Unexpected character: @
        [line 1] Error: Unexpected character: $
        [line 1] Error: Unexpected character: #
        [line 1] Error: Unexpected character: %
    "};

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, error4, BUILD_ERROR);
}

#[test]
fn not_equal() {
    let input1 = "!=";
    let expected1 = indoc! {"
        BANG_EQUAL != null
        EOF  null
    "};

    let input2 = "!!===";
    let expected2 = indoc! {"
        BANG ! null
        BANG_EQUAL != null
        EQUAL_EQUAL == null
        EOF  null
    "};

    let input3 = "!{!}(!===)=";
    let expected3 = indoc! {"
        BANG ! null
        LEFT_BRACE { null
        BANG ! null
        RIGHT_BRACE } null
        LEFT_PAREN ( null
        BANG_EQUAL != null
        EQUAL_EQUAL == null
        RIGHT_PAREN ) null
        EQUAL = null
        EOF  null
    "};

    let input4 = "{(#@==$=)}";
    let expected4 = indoc! {"
        LEFT_BRACE { null
        LEFT_PAREN ( null
        EQUAL_EQUAL == null
        EQUAL = null
        RIGHT_PAREN ) null
        RIGHT_BRACE } null
        EOF  null
    "};
    let error4 = indoc! {"
        [line 1] Error: Unexpected character: #
        [line 1] Error: Unexpected character: @
        [line 1] Error: Unexpected character: $
    "};

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, error4, BUILD_ERROR);
}

#[test]
fn relational() {
    let input1 = ">=";
    let expected1 = indoc! {"
        GREATER_EQUAL >= null
        EOF  null
    "};

    let input2 = "<<<=>>>=";
    let expected2 = indoc! {"
        LESS < null
        LESS < null
        LESS_EQUAL <= null
        GREATER > null
        GREATER > null
        GREATER_EQUAL >= null
        EOF  null
    "};

    let input3 = ">=>=><>";
    let expected3 = indoc! {"
        GREATER_EQUAL >= null
        GREATER_EQUAL >= null
        GREATER > null
        LESS < null
        GREATER > null
        EOF  null
    "};

    let input4 = "(){!=<=<}";
    let expected4 = indoc! {"
        LEFT_PAREN ( null
        RIGHT_PAREN ) null
        LEFT_BRACE { null
        BANG_EQUAL != null
        LESS_EQUAL <= null
        LESS < null
        RIGHT_BRACE } null
        EOF  null
    "};

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, NO_ERROR, SUCCESS);
}

#[test]
fn division() {
    let input1 = "//Comment";
    let expected1 = indoc! {"
        EOF  null
    "};

    let input2 = "(///Unicode:£§᯽☺♣)";
    let expected2 = indoc! {"
        LEFT_PAREN ( null
        EOF  null
    "};

    let input3 = "/";
    let expected3 = indoc! {"
        SLASH / null
        EOF  null
    "};

    let input4 = "({(,!=>)})//Comment";
    let expected4 = indoc! {"
        LEFT_PAREN ( null
        LEFT_BRACE { null
        LEFT_PAREN ( null
        COMMA , null
        BANG_EQUAL != null
        GREATER > null
        RIGHT_PAREN ) null
        RIGHT_BRACE } null
        RIGHT_PAREN ) null
        EOF  null
    "};

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, NO_ERROR, SUCCESS);
}

#[test]
fn whitespace() {
    let input1 = " ";
    let expected1 = indoc! {"
        EOF  null
    "};

    let input2 = " \t\n ";
    let expected2 = indoc! {"
        EOF  null
    "};

    let input3 = "{ \n}\n((\n,+; ))";
    let expected3 = indoc! {"
        LEFT_BRACE { null
        RIGHT_BRACE } null
        LEFT_PAREN ( null
        LEFT_PAREN ( null
        COMMA , null
        PLUS + null
        SEMICOLON ; null
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        EOF  null
    "};

    let input4 = "{\t\t\n  }\n((, \t<*))";
    let expected4 = indoc! {"
        LEFT_BRACE { null
        RIGHT_BRACE } null
        LEFT_PAREN ( null
        LEFT_PAREN ( null
        COMMA , null
        LESS < null
        STAR * null
        RIGHT_PAREN ) null
        RIGHT_PAREN ) null
        EOF  null
    "};

    run_tokenize(input1, expected1, NO_ERROR, SUCCESS);
    run_tokenize(input2, expected2, NO_ERROR, SUCCESS);
    run_tokenize(input3, expected3, NO_ERROR, SUCCESS);
    run_tokenize(input4, expected4, NO_ERROR, SUCCESS);
}

#[test]
fn multiline_errors() {
    let input1 = indoc! {"
        ()
        \t@
    "};
    let expected1 = indoc! {"
        LEFT_PAREN ( null
        RIGHT_PAREN ) null
        EOF  null
    "};
    let error1 = indoc! {"
        [line 2] Error: Unexpected character: @
    "};

    let input2 = indoc! {"
        $\t
         
    "};
    let expected2 = indoc! {"
        EOF  null
    "};
    let error2 = indoc! {"
        [line 1] Error: Unexpected character: $
    "};

    let input3 = indoc! {"
        ()  #\t{}
        @
        $
        +++
        // Let's Go!
        +++
        #
    "};
    let expected3 = indoc! {"
        LEFT_PAREN ( null
        RIGHT_PAREN ) null
        LEFT_BRACE { null
        RIGHT_BRACE } null
        PLUS + null
        PLUS + null
        PLUS + null
        PLUS + null
        PLUS + null
        PLUS + null
        EOF  null
    "};
    let error3 = indoc! {"
        [line 1] Error: Unexpected character: #
        [line 2] Error: Unexpected character: @
        [line 3] Error: Unexpected character: $
        [line 7] Error: Unexpected character: #
    "};

    let input4 = indoc! {"
        ({- #})
    "};
    let expected4 = indoc! {"
        LEFT_PAREN ( null
        LEFT_BRACE { null
        MINUS - null
        RIGHT_BRACE } null
        RIGHT_PAREN ) null
        EOF  null
    "};
    let error4 = indoc! {"
        [line 1] Error: Unexpected character: #
    "};

    run_tokenize(input1, expected1, error1, BUILD_ERROR);
    run_tokenize(input2, expected2, error2, BUILD_ERROR);
    run_tokenize(input3, expected3, error3, BUILD_ERROR);
    run_tokenize(input4, expected4, error4, BUILD_ERROR);
}