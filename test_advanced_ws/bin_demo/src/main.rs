use lib_derive::{super_base_fn, super_util_fn, SuperOp};
use lib_trait::{Add, Mul};

fn foo(x: i32) -> i32 {
    bar(x + 1)
}
fn bar(y: i32) -> i32 {
    y * 2
}
fn run_trait(op: &dyn SuperOp, x: i32) -> i32 {
    op.apply(x)
}
fn main() {
    let add = Add;
    let mul = Mul;
    println!("{}", run_trait(&add, 1));
    println!("{}", run_trait(&mul, 2));
    let res = foo(3);
    println!("{}", res);
    super_base_fn(123);
    super_util_fn();
}
