use prelude::*;
use std::{env::args, process::exit};

pub mod cl;
pub mod cl2;

mod test;

pub fn run<Ctx, A: Acts<Ctx>>(c: &Ctx) {
    match parse::<Ctx, A>(c, &args().skip(1).collect()) {
        Ok(()) => (),
        Err(e) => {
            println!("Error: {e}");
            exit(1)
        }
    }
}

pub mod prelude {
    pub use super::cl::*;
    pub use super::cl2::*;
    pub use super::run;
    pub use std::process::Command;
}
