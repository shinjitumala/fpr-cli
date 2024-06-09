mod internal;
mod test;

pub use cli_derive::*;

use internal::*;
use itertools::Itertools;
use std::{
    env::args,
    fmt::{self, Debug, Display, Formatter},
    path::PathBuf,
};
use unicode_width::UnicodeWidthStr;

pub fn run<Ctx, A: IActs>(c: &Ctx) -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let args = args.iter().map(|a| a.as_str()).collect::<Vec<_>>();
    let prog = &args[0];
    let r = A::parse(c, &args[1..]);

    match r {
        Err(e) => Err(format!("Usage: {prog}")),
        Ok(r) => Ok(r),
    }
}

pub trait Acts {
    fn next<'args, 'arg, Ctx>(
        c: &Ctx,
        next: &'arg str,
        next_args: &'args [&'arg str],
    ) -> Result<(), EActs<'arg>>;
    fn desc() -> Vec<[&'static str; 2]>;
    fn act_desc() -> &'static str;
    // fn list(pfx: &Vec<String>) -> Vec<Vec<String>>;
}

pub trait Args
where
    Self: Sized,
{
    fn parse2<'args, 'arg, C>(
        c: &C,
        args: &mut ParsedArgs<'args, 'arg>,
    ) -> Result<Self, EParseArgs<'arg>>
    where
        Self: Run<C>;
    fn desc<Ctx>(c: &Ctx, r: &mut Vec<[String; 4]>);
    fn act_desc() -> &'static str;
}

pub trait Parse
where
    Self: Sized + Display + Clone,
{
    fn parse_str<'arg>(i: &'arg str) -> Result<Self, EParseArg>;
    fn tyname() -> &'static str;
}

#[derive(Clone, Debug)]
pub struct FileExist {
    pub p: PathBuf,
    pub s: String,
}
impl Display for FileExist {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

#[derive(Clone, Debug)]
pub struct DirExist {
    pub p: PathBuf,
    pub s: String,
}
impl Display for DirExist {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

pub enum Init<Ctx, T: Debug + Clone> {
    None,
    Const(T),
    Dyn(fn(&Ctx) -> T),
}

pub enum Desc<Ctx> {
    Const(&'static str),
    Dyn(fn(&Ctx) -> String),
}

pub fn print_table<const W: usize>(d: &Vec<[String; W]>) -> String {
    if d.is_empty() {
        return format!("");
    };
    let w: Vec<_> = (1..W)
        .map(|i| {
            d.iter()
                .map(|l| l[i].width())
                .max()
                .expect("Data should not be empty")
        })
        .collect();
    d.iter()
        .map(|v| {
            (1..W)
                .map(|i| format!("{}{: <2$}", v[i], "", w[i] - v[i].width()))
                .join(" ")
        })
        .join("\n")
}

pub trait Run<Ctx> {
    fn run(c: &Ctx, a: Self) -> Result<(), String>;
}

// pub fn list<Ctx, A: IActs>() -> Vec<Vec<String>> {
//     A::list(&vec![])
// }

pub fn parse_arg<'args, 'arg, Ctx, T: Parse2 + Debug + Clone>(
    c: &Ctx,
    k: &'static str,
    i: Init<Ctx, T::Init>,
    a: &mut ParsedArgs<'args, 'arg>,
) -> Result<T, EParseArg<'arg>> {
    Ok(T::parse_strs(i, k, c, a)?)
}
