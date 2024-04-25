pub use cli_derive::*;
pub use smart_default::SmartDefault;

type Tkns = Vec<String>;

#[derive(Clone)]
pub struct Act<Ctx> {
    desc: &'static str,
    act: fn(&Ctx, &Tkns),
}

impl<Ctx> Act<Ctx> {
    fn new(desc: &'static str, act: fn(&Ctx, &Tkns)) -> Self {
        Self { desc, act }
    }
}

trait ActPath<Ctx> {
    fn nested(&self) -> Vec<(&'static str, Act<Ctx>)>;
}

trait Acts<Ctx> {
    fn parse(c: &Ctx, args: &Tkns);
}

// test
struct TestCtx {}

#[derive(Acts, SmartDefault)]
#[ctx(TestCtx)]
struct TestActMap2 {
    #[default(_code = "Act::new(\"1\",|_,_|{})")]
    act1: Act<TestCtx>,
    #[default(_code = "Act::new(\"2\",|_,_|{})")]
    act2: Act<TestCtx>,
}

#[derive(Acts)]
#[ctx(TestCtx)]
struct TestActMap {}
