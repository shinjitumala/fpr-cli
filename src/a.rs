use crate::i::*;

struct X {}

struct Args2 {
    a: String,
}
struct Args1 {
    a: Args2,
    b: String,
}
impl Run for Args1 {
    fn run<X>(c: &X, a: Self) {
        todo!()
    }
}

struct Main {
    a: Args1,
    m: Main2,
}
impl Acts for Main {
    fn next<C>(c: &C) -> Result<(), ()> {
        todo!()
    }
}

struct Main2 {
    b: Args2,
}

impl Run for i32 {
    fn run<X = X>(c: &X, a: Self) {}
}

struct Y {}

pub fn test() {
    let x = Y {};
    Main::run(&x);
}

#[derive(Acts2)]
#[act2()]
enum Main3 {
    Foo(Args1),
    // Bar,
}
