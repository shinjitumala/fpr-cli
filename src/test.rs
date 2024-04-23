use super::*;

mod ctx {
    pub struct MyCtx {}
}

#[test]
fn test() {
    let args = vec![
        format!("-aaa"),
        format!("a"),
        format!("b"),
        format!("c"),
        format!("-foo"),
        format!("bar"),
        format!("-baz"),
        format!("-nyom"),
        format!("-baa"),
        format!("a"),
        format!("b"),
        format!("c"),
        format!("-name"),
        format!("c"),
    ];
    let mut r = to_argmap(&args);
    <Test as SArg>::parse("", & mut r);
    println!("{:?}", r);
}

// pub trait A {
//     type I;
//
//     fn bar(&self) -> Self::I;
// }
//
// struct B {
//     i: i32 = 1,
// }
// impl Default for
//
// impl A for B {
//     type I = i32;
//     fn bar(&self) -> Self::I {
//         1
//     }
// }
//
// fn foo() {
//     let b = B{};
//
//     {
//         foo: b.bar(),
//     }
// }
