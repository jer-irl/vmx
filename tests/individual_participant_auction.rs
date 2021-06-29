mod common_programs;

use vmx::Price;

#[test]
fn program_quotes_applied() {
    let program = common_programs::replace_quotes(Price(100), 100, Price(200), 100);
    todo!();
}

#[test]
fn parameters_updated() {
    todo!();
}

#[test]
fn prevent_self_crossing() {
    todo!();
}
