use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    process,
};

use cli::{cl::*, cl2::*};
use regex::Regex;

struct C {}

type V<V> = Arg<C, V>;

#[derive(Args)]
#[args(ctx = C)]
struct Main {
    #[arg(desc = ("Files and directoreis to be parsed."), i = Init::None)]
    paths: V<Req<Vec<PathExist>>>,
}

struct Pats {
    start: Regex,
    ty: Regex,
    arg: Regex,
    end: Regex,
}

fn gen(f: &PathExist, p: &Pats) -> String {
    println!("{:?}", &f.p);
    let f = BufReader::new(File::open(&f.p).expect("Failed to open file"));

    for l in f.lines() {
        let l = l.unwrap();
        if p.start.find(&l).is_some() {
            println!("start {}", l);
        }
        println!("{}", l);
    }

    format!("")
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

    let p = Pats {
        start: Regex::new("^# start metadata$").unwrap(),
        end: Regex::new("^# end metadata$").unwrap(),
        ty: Regex::new("^# type text$").unwrap(),
        arg: Regex::new(r"^([^=])+=\$(\d+) # (.*)$").unwrap(),
    };

    println!("{:?}", a);

    a.paths.iter().for_each(|f| {
        gen(f, &p);
    });
}
