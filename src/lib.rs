mod test;

pub mod prelude {
    pub use super::run;
}

pub use cli_derive::*;
use itertools::Itertools;
use std::env::args;
use std::fmt::Debug;
use std::str::FromStr;

type Res<T> = Result<T, String>;

pub fn run<Ctx, A: Acts<Ctx>>(c: &Ctx) -> Res<()> {
    parse::<_, A>(c, &args().collect::<Vec<_>>())
}

trait Acts<Ctx> {
    fn parse(c: &Ctx, pfx: Option<String>, args: &[String]) -> Res<()>;
    fn desc() -> Vec<Vec<String>>;
    fn act_desc() -> &'static str;
    fn list(pfx: &Vec<String>) -> Vec<Vec<String>>;
}

trait Args<Ctx>
where
    Self: Sized,
{
    fn parse(c: &Ctx, args: &mut ParsedArgs) -> Res<Self>;
    fn desc(c: &Ctx) -> Vec<Vec<String>>;
}

trait Parse
where
    Self: Sized,
{
    fn parse(i: &String) -> Res<Self>;
    fn desc() -> &'static str;
}

impl Parse for i32 {
    fn parse(i: &String) -> Res<Self> {
        i32::from_str(i).or(Err(format!("Failed to parse {}: {i}", Self::desc())))
    }

    fn desc() -> &'static str {
        stringify!(i32)
    }
}

impl Parse for String {
    fn parse(i: &String) -> Res<Self> {
        Ok(i.to_owned())
    }

    fn desc() -> &'static str {
        stringify!(String)
    }
}

const PFX: &'static str = "--";
struct Key {
    i: usize,
    used: bool,
}
struct ParsedArgs<'a, 'b> {
    prog: &'a String,
    args: &'b [String],
    keys: Vec<Key>,
}

impl<'a, 'b> ParsedArgs<'a, 'b> {
    fn consume(&mut self, name: &'static str) -> Option<&[String]> {
        let (i, k) = self
            .keys
            .iter_mut()
            .enumerate()
            .find(|(_, k)| self.args[k.i] == name)?;
        assert!(
            !k.used,
            "A token should not ever be consumed twice. Probably a duplicate argument: {name}"
        );
        k.used = true;
        let idx = k.i + 1;
        Some(if i == self.keys.len() - 1 {
            &self.args[idx..]
        } else {
            &self.args[idx..self.keys[i + 1].i]
        })
    }
}

fn parse_args(args: &[String]) -> Res<ParsedArgs> {
    let prog = &args[0];
    let args = &args[1..];

    let r = ParsedArgs {
        prog,
        args,
        keys: args
            .iter()
            .enumerate()
            .filter(|&(_, a)| a.starts_with(PFX))
            .map(|(i, _)| Key { i, used: false })
            .collect(),
    };

    if !r.keys.is_empty() && r.keys[0].i != 0 {
        Err(format!("Expected argument start: {}", args[0]))
    } else {
        Ok(r)
    }
}

pub enum Init<Ctx, T: Parse2<Ctx>> {
    None,
    Const(T),
    Dyn(fn(&Ctx) -> T),
}

fn get_init<Ctx, T: Parse2<Ctx>>(c: &Ctx, i: &Init<Ctx, T>) -> Option<T> {
    match i {
        Init::None => None,
        Init::Const(v) => Some(v.to_owned()),
        Init::Dyn(f) => Some(f(&c)),
    }
}

fn get_init_desc<Ctx, T: Parse2<Ctx>>(c: &Ctx, i: &Init<Ctx, T>) -> String {
    match get_init(c, i) {
        Some(s) => format!(" (default: {:?})", s),
        None => format!(""),
    }
}

pub enum Desc<Ctx> {
    Const(&'static str),
    Dyn(fn(&Ctx) -> String),
}

struct Arg<Ctx, T: Parse2<Ctx>> {
    init: Init<Ctx, T>,
    desc: Desc<Ctx>,
}

impl<Ctx, T: Parse2<Ctx>> Arg<Ctx, T> {
    fn new(init: Init<Ctx, T>, desc: Desc<Ctx>) -> Self {
        Self { init, desc }
    }
}

fn get_desc<Ctx>(c: &Ctx, d: &Desc<Ctx>) -> String {
    match d {
        Desc::Const(e) => e.to_string(),
        Desc::Dyn(f) => f(c),
    }
}

trait Parse2<Ctx>
where
    Self: Sized + Clone + Debug,
{
    fn parse2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx, p: &mut ParsedArgs) -> Res<Self>;
    fn desc2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx) -> Vec<String>;
}

impl<Ctx, T: Parse + Clone + Debug> Parse2<Ctx> for T {
    fn parse2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx, p: &mut ParsedArgs) -> Res<Self> {
        match p.consume(k) {
            Some(args) => {
                if args.len() != 1 {
                    Err(format!("Expected one value for '{k}': {:?}", args))
                } else {
                    T::parse(&args[0])
                }
            }
            None => Ok(get_init(c, &a.init).ok_or(format!("'{k}' is required."))?),
        }
    }

    fn desc2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx) -> Vec<String> {
        vec![
            k.to_string(),
            format!("Req<{}>", T::desc()),
            get_desc(c, &a.desc),
            get_init_desc(c, &a.init),
        ]
    }
}

impl<Ctx, T: Parse + Clone + Debug> Parse2<Ctx> for Option<T> {
    fn parse2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx, p: &mut ParsedArgs) -> Res<Self> {
        match p.consume(k) {
            Some(args) => {
                if args.len() != 1 {
                    Err(format!("Expected one value for '{k}'"))
                } else {
                    Ok(Some(T::parse(&args[0])?))
                }
            }
            None => Ok(match get_init::<Ctx, Self>(c, &a.init) {
                Some(e) => e,
                None => None,
            }),
        }
    }

    fn desc2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx) -> Vec<String> {
        vec![
            k.to_string(),
            format!("Opt<{}>", T::desc()),
            get_desc(c, &a.desc),
            get_init_desc(c, &a.init),
        ]
    }
}

impl<Ctx, T: Parse + Clone + Debug> Parse2<Ctx> for Vec<T> {
    fn parse2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx, p: &mut ParsedArgs) -> Res<Self> {
        match p.consume(k) {
            Some(args) => args.iter().map(|a| T::parse(a)).collect(),
            None => Ok(match get_init::<Ctx, Self>(c, &a.init) {
                Some(e) => e,
                None => vec![],
            }),
        }
    }

    fn desc2(a: &Arg<Ctx, Self>, k: &'static str, c: &Ctx) -> Vec<String> {
        vec![
            k.to_string(),
            format!("Vec<{}>", T::desc()),
            get_desc(c, &a.desc),
            get_init_desc(c, &a.init),
        ]
    }
}

impl<Ctx> From<&'static str> for Desc<Ctx> {
    fn from(value: &'static str) -> Self {
        Desc::Const(value)
    }
}

impl<Ctx> From<fn(&Ctx) -> String> for Desc<Ctx> {
    fn from(value: fn(&Ctx) -> String) -> Self {
        Desc::Dyn(value)
    }
}

impl<Ctx, T: Parse2<Ctx>> From<T> for Init<Ctx, T> {
    fn from(value: T) -> Self {
        Init::Const(value)
    }
}

impl<Ctx> From<&'static str> for Init<Ctx, String> {
    fn from(value: &'static str) -> Self {
        Init::Const(value.to_string())
    }
}

fn parse2<Ctx, A: Args<Ctx>>(c: &Ctx, args: &[String]) -> Res<A> {
    let usage = || print_table(&A::desc(c));
    let parse_err = |e| -> String { format!("Parse error: {e}\nUsage:\n{}", usage()) };
    let mut p = parse_args(args).map_err(parse_err)?;

    if p.consume("--help").is_some() {
        return Err(format!("{}", usage()));
    }

    let a = A::parse(c, &mut p).map_err(parse_err)?;

    let u = p
        .keys
        .iter()
        .filter(|k| !k.used)
        .map(|k| p.args[k.i].to_owned())
        .collect::<Vec<_>>();
    if !u.is_empty() {
        Err(parse_err(format!("Unknown arguments '{:?}'", u)))
    } else {
        Ok(a)
    }
}

fn print_table(d: &Vec<Vec<String>>) -> String {
    if d.is_empty() {
        return format!("");
    };
    let w = d[0]
        .iter()
        .enumerate()
        .map(|(i, _)| {
            d.iter()
                .map(|l| l[i].len())
                .max()
                .expect("Data should not be empty")
        })
        .collect::<Vec<_>>();

    d.iter()
        .map(|v| {
            v.iter()
                .enumerate()
                .map(|(i, s)| format!("{0:1$}", s, w[i]))
                .join(" ")
        })
        .join("\n")
}

pub const LIST_SEP: &'static str = "_";

struct Act<Ctx, A: Args<A>> {
    act: fn(c: &Ctx, a: A) -> Res<()>,
    desc: &'static str,
}

fn parse<Ctx, A: Acts<Ctx>>(c: &Ctx, args: &[String]) -> Res<()> {
    A::parse(c, Option::None, args)
}
