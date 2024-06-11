pub use cli_derive::*;
use std::env::args;

pub trait Acts {
    fn run<C>(c: &C) -> Result<(), ()>
    where
        Self: IActs,
    {
        let a: Vec<_> = args().collect();
        let p = &a[0];

        match Self::parse_args(c, &a[1..]) {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("Usage: {p} {e}");
                Err(())
            }
        }
    }

    fn next<C>(c: &C) -> Result<(), ()>;
}

trait IActs {
    fn parse_args<C>(c: &C, a: &[String]) -> Result<(), String>
    where
        Self: Acts,
    {
        Self::next(c);

        i32::run(c, 1);
        todo!()
    }
}
impl<T: Acts> IActs for T {}

pub trait Args {}

pub trait Run {
    fn run<C>(c: &C, a: Self);
}

pub trait Acts2 {}

trait ActsParse {
    fn foo() {}
}
