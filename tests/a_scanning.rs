mod util;
use predicates::str::contains;
use util::run_tokenize;

#[test]
fn empty_file() {
    run_tokenize("")
        .stdout(contains("EOF  null"));
}
