macro_rules! call_foo {
    () => { foo(); };
}
fn foo() {}
fn main() {
    call_foo!();
}
