#[test]
fn test() {
    use crate::*;

    let choices: Element = char("a-z[") | (str("a").min(1).neg() | str("a")).neg().times(2) + str("a");
    println!("{}", choices);
}
