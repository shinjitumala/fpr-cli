pub use cli_derive::*;
use itertools::Itertools;
use regex::Regex;
pub use smart_default::SmartDefault;
use std::{collections::HashMap, fmt::Debug, path::PathBuf, process, str::FromStr};

const PFX: &'static str = "--";
#[derive(Debug)]
pub struct Tkns {
    tkns: Vec<String>,
    consumed: bool,
}

impl Tkns {
    fn consume(&mut self, name: &'static str) -> ParseResult<()> {
        if self.consumed {
            Err(format!("Multiple consumers for '{}' (To developer)", name))
        } else {
            self.consumed = true;
            Ok(())
        }
    }
}

pub type ArgMap = HashMap<String, Tkns>;

fn add_prefix(name: &String) -> String {
    format!("{}{}", PFX, name)
}

fn to_argmap(args: &[String]) -> ParseResult<ArgMap> {
    let remove_prefix = Regex::new(&format!("^{}", PFX)).unwrap();

    let mut res = ArgMap::new();

    let mut idx_arg_names: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|&(_, a)| a.starts_with(PFX))
        .map(|(i, _)| i)
        .collect();
    idx_arg_names.push(args.len());

    if !args.is_empty() && idx_arg_names[0] != 0 {
        return Err(format!("Expected argument start {}", args[0]));
    }

    for (i, next) in idx_arg_names.iter().tuple_windows() {
        let x = &args[*i];

        let r = res.insert(
            remove_prefix.replace_all(x, "").to_string(),
            Tkns {
                tkns: args[*i + 1..*next].to_owned(),
                consumed: false,
            },
        );
        match r {
            Some(_) => return Err(format!("Duplicate argument: {}", x)),
            None => (),
        }
    }
    Ok(res)
}

pub type ParseResult<T> = Result<T, String>;

pub trait Parse
where
    Self: Sized + Clone + Debug,
{
    fn parse(name: &'static str, tkn: &String) -> ParseResult<Self>;
}

impl<T: FromStr + Clone + Debug> Parse for T {
    fn parse(name: &'static str, tkn: &String) -> ParseResult<Self> {
        match Self::from_str(tkn) {
            Ok(v) => Ok(v),
            Err(_) => Err(format!("Failed to parse '{}': {}", name, tkn)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathExist {
    pub p: PathBuf,
}

impl Parse for PathExist {
    fn parse(name: &'static str, tkn: &String) -> ParseResult<Self> {
        match PathBuf::from_str(tkn) {
            Ok(v) => {
                if v.exists() {
                    Ok(PathExist { p: v })
                } else {
                    Err(format!("Path does not exist: {:?}", v))
                }
            }
            Err(_) => Err(format!("Failed to parse '{}': {}", name, tkn)),
        }
    }
}

pub struct One<T: Parse> {
    _v: T,
}

pub trait Parse2
where
    Self: Sized,
    Self::R: Clone + Debug,
{
    type R;
    fn parse(name: &'static str, tkns: &[String]) -> ParseResult<Self::R>;
    fn desc() -> &'static str;
}

impl<T: Parse> Parse2 for One<T> {
    type R = T;
    fn parse(name: &'static str, tkns: &[String]) -> ParseResult<Self::R> {
        if tkns.len() == 1 {
            Ok(T::parse(name, &tkns[0])?)
        } else {
            Err(format!("Expected single value for '{}': {:?}", name, tkns))
        }
    }

    fn desc() -> &'static str {
        "   "
    }
}

impl<T: Parse> Parse2 for Vec<T> {
    type R = Vec<T>;
    fn parse(name: &'static str, tkns: &[String]) -> ParseResult<Self::R> {
        if tkns.len() == 0 {
            Err(format!("Expected at least one value for '{}'", name))
        } else {
            Ok(tkns.iter().map(|t| T::parse(name, t)).try_collect()?)
        }
    }
    fn desc() -> &'static str {
        "Arr"
    }
}

pub struct Req<T: Parse2> {
    _v: T,
}

pub struct Opt<T: Parse2> {
    _v: Option<T>,
}

pub enum Init<Ctx, T: Parse2> {
    None,
    Const(T::R),
    Dyn(fn(&Ctx) -> T::R),
}

impl<Ctx> From<&'static str> for Init<Ctx, One<String>> {
    fn from(value: &'static str) -> Self {
        Init::Const(value.to_string())
    }
}

impl<Ctx> From<i32> for Init<Ctx, One<i32>> {
    fn from(value: i32) -> Self {
        Init::Const(value)
    }
}

impl<Ctx, T: Parse2> From<fn(&Ctx) -> T::R> for Init<Ctx, T> {
    fn from(value: fn(&Ctx) -> T::R) -> Self {
        Init::Dyn(value)
    }
}

fn get_init<Ctx, T: Parse2>(c: &Ctx, init: &Init<Ctx, T>) -> Option<T::R> {
    match init {
        Init::None => None,
        Init::Const(ref t) => Some(t.to_owned()),
        Init::Dyn(f) => Some(f(c)),
    }
}

pub trait Parse3<Ctx>
where
    Self::Init: Parse2,
{
    type R;
    type Init;
    fn parse(
        c: &Ctx,
        name: &'static str,
        init: &Init<Ctx, Self::Init>,
        am: &mut ArgMap,
    ) -> ParseResult<Self::R>;
    fn desc() -> String;
}

impl<Ctx, T: Parse2> Parse3<Ctx> for Req<T> {
    type R = T::R;
    type Init = T;

    fn parse(
        c: &Ctx,
        name: &'static str,
        init: &Init<Ctx, Self::Init>,
        am: &mut ArgMap,
    ) -> ParseResult<Self::R> {
        match am.get_mut(name) {
            Some(v) => {
                v.consume(name)?;
                T::parse(name, &v.tkns)
            }
            None => match get_init(c, init) {
                Some(e) => Ok(e),
                None => Err(format!("Argument '{}' is required.", add_prefix(&name.to_string()))),
            },
        }
    }
    fn desc() -> String {
        format!("Req {}", T::desc())
    }
}
impl<Ctx, T: Parse2> Parse3<Ctx> for Opt<T> {
    type Init = T;
    type R = Option<T::R>;

    fn parse(
        c: &Ctx,
        name: &'static str,
        init: &Init<Ctx, Self::Init>,
        am: &mut ArgMap,
    ) -> ParseResult<Self::R> {
        match am.get_mut(name) {
            Some(v) => {
                v.consume(name)?;
                Ok(Some(T::parse(name, &v.tkns)?))
            }
            None => match get_init(c, init) {
                Some(e) => Ok(Some(e)),
                None => Ok(None),
            },
        }
    }
    fn desc() -> String {
        format!("Opt {}", T::desc())
    }
}

trait ArgT<Ctx, T: Parse3<Ctx>> {
    fn desc(&self, c: &Ctx) -> String;
    fn parse(&self, c: &Ctx, name: &'static str, am: &mut ArgMap) -> ParseResult<T::R>;
}

pub enum Desc<Ctx> {
    Const(&'static str),
    Dyn(fn(&Ctx) -> String),
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

pub struct Arg<Ctx, T: Parse3<Ctx>>
where
    T::Init: Parse2,
{
    init: Init<Ctx, T::Init>,
    desc: Desc<Ctx>,
}

impl<Ctx, T: Parse3<Ctx>> ArgT<Ctx, T> for Arg<Ctx, T>
where
    T::Init: Parse2,
{
    fn desc(&self, c: &Ctx) -> String {
        let x = format!(
            "{} {}",
            T::desc(),
            match self.desc {
                Desc::Const(c) => c.to_string(),
                Desc::Dyn(f) => f(c),
            }
        );
        match get_init(c, &self.init) {
            Some(ref i) => format!("{} (default: {:?})", x, i),
            None => x,
        }
    }

    fn parse(&self, c: &Ctx, name: &'static str, am: &mut ArgMap) -> ParseResult<T::R> {
        T::parse(c, name, &self.init, am)
    }
}

impl<Ctx, T: Parse3<Ctx>> Arg<Ctx, T>
where
    T::Init: Parse2,
{
    pub fn new(desc: Desc<Ctx>, init: Init<Ctx, T::Init>) -> Self {
        Self { desc, init }
    }
    pub fn s(desc: &'static str) -> Self {
        Self::new(Desc::<Ctx>::Const(desc), Init::None)
    }
    pub fn d(f: fn(&Ctx) -> String) -> Self {
        Self::new(Desc::<Ctx>::Dyn(f), Init::None)
    }
}

pub trait Args<Ctx> {
    type R;
    fn desc(&self, name: &'static str, c: &Ctx) -> Vec<(String, String)>;
    fn parse(&self, name: &'static str, c: &Ctx, am: &mut ArgMap) -> ParseResult<Self::R>;
}

impl<Ctx, T: Parse3<Ctx>> Args<Ctx> for Arg<Ctx, T>
where
    T::Init: Parse2,
{
    type R = T::R;

    fn desc(&self, name: &'static str, c: &Ctx) -> Vec<(String, String)> {
        vec![(name.to_string(), ArgT::desc(self, c))]
    }

    fn parse(&self, name: &'static str, c: &Ctx, am: &mut ArgMap) -> ParseResult<Self::R> {
        ArgT::parse(self, c, name, am)
    }
}

fn print_table(d: &Vec<(String, String)>) -> String {
    let r0 = d
        .iter()
        .map(|v| v.0.len() + PFX.len())
        .max()
        .expect("Data should not be empty.");
    let r1 = d
        .iter()
        .map(|v| v.1.len())
        .max()
        .expect("Data should not be empty.");
    d.iter()
        .map(|v| format!("{0:1$} {2:3$}", add_prefix(&v.0), r0, v.1, r1))
        .join("\n")
}

pub fn parse2<Ctx, A: Args<Ctx> + Default>(
    ctx: &Ctx,
    args: &[String],
) -> ParseResult<<A as Args<Ctx>>::R> {
    let y = || -> ParseResult<<A as Args<Ctx>>::R> {
        let mut am = to_argmap(&args)?;
        let a = A::default();

        match am.get("help") {
            Some(_) => {
                println!("Usage:\n{}", desc::<Ctx, A>(ctx));
                process::exit(0);
            }
            None => (),
        }

        let r = a.parse("", ctx, &mut am)?;

        let errs: Vec<String> = am
            .iter()
            .filter(|a| a.1.consumed == false)
            .map(|a| format!("Unknown argument: {}", add_prefix(a.0)))
            .collect();

        if !errs.is_empty() {
            Err(format!("{}", errs.join("\n")))
        } else {
            Ok(r)
        }
    };

    match y() {
        Ok(a) => Ok(a),
        Err(e) => Err(format!("{e}\nUsage:\n{}", desc::<Ctx, A>(ctx))),
    }
}

pub fn desc<Ctx, A: Args<Ctx> + Default>(c: &Ctx) -> String {
    let a = A::default();
    let mut v = a.desc("", c);
    v.push((format!("help"), format!("Show this.")));
    print_table(&v)
}

pub type Ret<Ctx, A> = <A as Args<Ctx>>::R;
