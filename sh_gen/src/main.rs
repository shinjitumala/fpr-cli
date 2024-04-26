use std::{env::args, process};

use cli::{cl::*, cl2::*};

struct C {}

type V<V> = Arg<C, V>;

#[derive(Args)]
#[args(ctx = C)]
struct Main {
    #[arg(desc = ("Files and directoreis to be parsed."), i = Init::None)]
    paths: V<Req<Vec<PathExist>>>,
}

fn main() {
    let c = C {};
    let a = match cli::cl2::parse::<_, Main>(&c, &args().skip(1).collect::<Vec<_>>()) {
        Ok(a) => a,
        Err(e) => {
            println!("Parse error: {}\nUsage:\n{}", e, desc::<_, Main>(&c));
            process::exit(1);
        }
    };
}
