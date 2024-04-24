use super::*;

struct FakeCtx {}

#[derive(SArg)]
struct Test2 {
    next: Arg<FakeCtx, Option<String>>,
}

#[derive(SArg)]
struct Test {
    name: Arg<FakeCtx, Require<i32>>,
    id: Arg<FakeCtx, Option<A>>,
    test: Test2,
}

#[derive(Debug)]
struct A {
    name: String,
}
impl Parse for A {
    fn parse(name: &'static str, tkn: &String) -> Self {
        todo!()
    }
}

#[test]
fn test() {
    let args = vec![
        // format!("-aaa"),
        // format!("a"),
        // format!("b"),
        // format!("c"),
        // format!("-foo"),
        // format!("bar"),
        // format!("-baz"),
        // format!("-nyom"),
        // format!("-baa"),
        // format!("a"),
        // format!("b"),
        // format!("c"),
        format!("-name"),
        // format!("10"),
    ];
    let x = parse::<Test>(&args);
    println!("{:?}", x);
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
