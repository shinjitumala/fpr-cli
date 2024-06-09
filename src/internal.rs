use super::*;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

pub enum EActsKind<'arg> {
    Run(String),
    ExpectedAnAction,
    NotAnAction(&'arg str),
    ParseActs(EParseActs<'arg>),
    ParseArgs(EParseArgs<'arg>),
}
impl<'arg> From<EParseActs<'arg>> for EActsKind<'arg> {
    fn from(v: EParseActs<'arg>) -> Self {
        Self::ParseActs(v)
    }
}
impl<'arg> From<EParseArgs<'arg>> for EActs<'arg> {
    fn from(v: EParseArgs<'arg>) -> Self {
        EActs {
            kind: EActsKind::ParseArgs(v),
            stack: vec![],
        }
    }
}
impl<'arg> From<String> for EActs<'arg> {
    fn from(v: String) -> Self {
        EActs {
            kind: EActsKind::Run(v),
            stack: vec![],
        }
    }
}

pub struct EActs<'arg> {
    pub kind: EActsKind<'arg>,
    pub stack: Vec<&'arg str>,
    // pub usage: String,
}
enum EParseActs<'arg> {
    ParseArgs(EParseArgs<'arg>),
}
pub enum EParseArgs<'arg> {
    ExpectedArgStart(&'arg str),
    ParseError(EParseArg<'arg>),
    UnknownArgs(Vec<&'arg str>),
    Help,
}
impl<'arg> Display for EParseArgs<'arg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use EParseArgs::*;
        match self {
            ExpectedArgStart(a) => write!(f, "Expected arg start at '{a}'"),
            ParseError(e) => write!(f, "{e}"),
            UnknownArgs(args) => write!(f, "Unknown args '{args:?}'"),
            Help => panic!("Cannot display help."),
        }
    }
}
pub struct EParseArg<'arg> {
    pub i: &'arg str,
    pub ty: &'static str,
    pub err: String,
}
impl<'arg> Display for EParseArg<'arg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse '{}' as '{}' because '{}'",
            self.i, self.ty, self.err
        )
    }
}

impl<'arg> From<EParseArg<'arg>> for EParseArgs<'arg> {
    fn from(v: EParseArg<'arg>) -> Self {
        Self::ParseError(v)
    }
}

impl Parse for i32 {
    fn parse_str(i: &str) -> Result<Self, EParseArg> {
        i32::from_str(i).map_err(|e| EParseArg {
            i,
            ty: Self::tyname(),
            err: format!("{e}"),
        })
    }

    fn tyname() -> &'static str {
        stringify!(i32)
    }
}

impl Parse for String {
    fn parse_str(i: &str) -> Result<Self, EParseArg> {
        Ok(i.to_owned())
    }

    fn tyname() -> &'static str {
        stringify!(String)
    }
}

impl Parse for FileExist {
    fn parse_str(i: &str) -> Result<Self, EParseArg> {
        let r = (|| -> Result<PathBuf, String> {
            let p = PathBuf::from_str(i).map_err(|e| e.to_string())?;
            if !p.exists() {
                return Err(format!("Path does not exist '{i}'"));
            };
            if !p.is_file() {
                return Err(format!("Not a file '{i}'"));
            };
            Ok(p)
        })();

        match r {
            Ok(p) => Ok(FileExist { p, s: i.to_owned() }),
            Err(err) => Err(EParseArg {
                i,
                ty: Self::tyname(),
                err,
            }),
        }
    }

    fn tyname() -> &'static str {
        stringify!(FileExist)
    }
}

impl Parse for DirExist {
    fn parse_str(i: &str) -> Result<Self, EParseArg> {
        let r = {
            match PathBuf::from_str(i) {
                Ok(p) => {
                    if !p.exists() {
                        Err(format!("Path does not exist '{i}'"))
                    } else if !p.is_dir() {
                        Err(format!("Not a directory '{i}'"))
                    } else {
                        Ok(p)
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        };
        match r {
            Ok(p) => Ok(DirExist { p, s: i.to_owned() }),
            Err(err) => Err(EParseArg {
                i,
                ty: Self::tyname(),
                err,
            }),
        }
    }

    fn tyname() -> &'static str {
        stringify!(DirExist)
    }
}

const PFX: &'static str = "--";
struct Key {
    pub i: usize,
    pub used: bool,
}
pub struct ParsedArgs<'args, 'arg> {
    pub args: &'args [&'arg str],
    pub keys: Vec<Key>,
}

impl<'args, 'arg> ParsedArgs<'args, 'arg> {
    pub fn consume(&mut self, name: &'static str) -> Option<&'args [&'arg str]> {
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

pub fn parse_args<'args, 'arg>(
    args: &'args [&'arg str],
) -> Result<ParsedArgs<'args, 'arg>, EParseArgs<'arg>> {
    let mut r = ParsedArgs {
        args,
        keys: args
            .iter()
            .enumerate()
            .filter(|&(_, a)| a.starts_with(PFX))
            .map(|(i, _)| Key { i, used: false })
            .collect(),
    };

    if r.consume("--help").is_some() {
        return Err(EParseArgs::Help);
    }

    if !r.keys.is_empty() && r.keys[0].i != 0 {
        Err(EParseArgs::ExpectedArgStart(args[0]))
    } else {
        Ok(r)
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

impl<Ctx, T: Debug + Clone> From<T> for Init<Ctx, T> {
    fn from(value: T) -> Self {
        Init::Const(value)
    }
}

impl<Ctx> From<&'static str> for Init<Ctx, String> {
    fn from(value: &'static str) -> Self {
        Init::Const(value.to_string())
    }
}

// fn parse2<'args, 'arg, Ctx, A: Args + IArgs + Run<Ctx>>(
//     c: &Ctx,
//     args: &'args [&'arg str],
// ) -> Result<A, EParseArgs<'arg>>{
//     let mut p = parse_args(args)?;
//
//     if p.consume("--help").is_some() {
//         return Err(EParseArgs::Help);
//     }
//
//     let a = A::parse2(c, &mut p)?;
//
//     let u = p
//         .keys
//         .iter()
//         .filter(|k| !k.used)
//         .map(|k| p.args[k.i])
//         .collect::<Vec<_>>();
//     if !u.is_empty() {
//         Err(EParseArgs::UnknownArgs(u))
//     } else {
//         Ok(a)
//     }
// }

impl<T: Parse + Debug + Clone> Parse2 for Vec<T> {
    type Init = Vec<T>;
    fn parse_strs<'args, 'arg, Ctx>(
        i: Init<Ctx, Self::Init>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'args, 'arg>,
    ) -> Result<Self, EParseArg<'arg>> {
        match p.consume(k) {
            Some(args) => args.iter().map(|a| T::parse_str(a)).collect(),
            None => Ok(match get_init::<Ctx, Self::Init>(c, &i) {
                Some(e) => e,
                None => vec![],
            }),
        }
    }

    fn desc2<Ctx>(
        c: &Ctx,
        k: &'static str,
        d: &'static str,
        i: Init<Ctx, Self::Init>,
    ) -> [String; 4] {
        [
            format!("{PFX}{k}"),
            format!("Vec<{}>", T::tyname()),
            d.to_owned(),
            get_init_desc(c, &i),
        ]
    }
}

impl<T: Parse + Debug + Clone> Parse2 for T {
    type Init = T;
    fn parse_strs<'args, 'arg, Ctx>(
        i: Init<Ctx, Self::Init>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'args, 'arg>,
    ) -> Result<Self, EParseArg<'arg>> {
        let r = (|| -> Result<Result<Self, EParseArg>, String> {
            match p.consume(k) {
                Some(args) => {
                    if args.len() != 1 {
                        Err(format!("Expected only one value for '{:?}'.", args))
                    } else {
                        Ok(T::parse_str(&args[0]))
                    }
                }
                None => Ok(Ok(get_init(c, &i).ok_or(format!("'{k}' is required."))?)),
            }
        })();
        match r {
            Err(err) => Err(EParseArg {
                i: k,
                ty: T::tyname(),
                err,
            }),
            Ok(o) => o,
        }
    }

    fn desc2<Ctx>(
        c: &Ctx,
        k: &'static str,
        d: &'static str,
        i: Init<Ctx, Self::Init>,
    ) -> [String; 4] {
        [
            k.to_string(),
            format!("Req<{}>", T::tyname()),
            d.to_owned(),
            get_init_desc(c, &i),
        ]
    }
}

impl<T: Parse + Debug + Clone> Parse2 for Option<T> {
    type Init = T;
    fn parse_strs<'args, 'arg, Ctx>(
        i: Init<Ctx, Self::Init>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'args, 'arg>,
    ) -> Result<Self, EParseArg<'arg>> {
        let r = (|| -> Result<Result<Self, EParseArg>, String> {
            match p.consume(k) {
                Some(args) => {
                    if args.len() != 1 {
                        Err(format!("Expected one value for '{k}'"))
                    } else {
                        Ok(match T::parse_str(&args[0]) {
                            Err(e) => Err(e),
                            Ok(t) => Ok(Some(t)),
                        })
                    }
                }
                None => Ok(Ok(get_init::<Ctx, Self::Init>(c, &i))),
            }
        })();
        match r {
            Err(err) => Err(EParseArg {
                i: k,
                ty: T::tyname(),
                err,
            }),
            Ok(o) => o,
        }
    }

    fn desc2<Ctx>(
        c: &Ctx,
        k: &'static str,
        d: &'static str,
        i: Init<Ctx, Self::Init>,
    ) -> [String; 4] {
        [
            k.to_string(),
            format!("Opt<{}>", T::tyname()),
            d.to_owned(),
            get_init_desc(c, &i),
        ]
    }
}

fn get_init<Ctx, T: Debug + Clone>(c: &Ctx, i: &Init<Ctx, T>) -> Option<T> {
    match i {
        Init::None => None,
        Init::Const(v) => Some(v.to_owned()),
        Init::Dyn(f) => Some(f(&c)),
    }
}

fn get_init_desc<Ctx, T: Debug + Clone>(c: &Ctx, i: &Init<Ctx, T>) -> String {
    match get_init(c, i) {
        Some(s) => format!(" (default: {:?})", s),
        None => format!(""),
    }
}

fn get_desc<Ctx>(c: &Ctx, d: &Desc<Ctx>) -> String {
    match d {
        Desc::Const(e) => e.to_string(),
        Desc::Dyn(f) => f(c),
    }
}

const LIST_SEP: &'static str = "_";

pub trait IActs
where
    Self: Acts,
{
    fn parse<'args, 'arg, Ctx>(c: &Ctx, args: &'args [&'arg str]) -> Result<(), EActs<'arg>> {
        if args.is_empty() {
            return Err(EActs {
                kind: EActsKind::ExpectedAnAction,
                stack: vec![],
                // usage: todo!(), //print_table(&Self::desc()),
            });
        };

        let next = args[0];
        let next_args = &args[1..];
        Self::next(c, next, next_args).map_err(|e| EActs {
            kind: e.kind,
            stack: e.stack.into_iter().chain([next]).collect(),
            // usage: todo!(), //print_table(&Self::desc()),
        })
    }
}
impl<T: Acts> IActs for T {}

pub trait IArgs
where
    Self: Args,
{
    fn parse<'args, 'arg, Ctx>(c: &Ctx, args: &'args [&'arg str]) -> Result<(), EActs<'arg>>
    where
        Self: Run<Ctx>,
    {
        let mut a = parse_args(args)?;

        let u = a
            .keys
            .iter()
            .filter(|k| !k.used)
            .map(|k| a.args[k.i])
            .collect::<Vec<_>>();

        if !u.is_empty() {
            return Err(EParseArgs::UnknownArgs(u))?;
        }

        Ok(Self::run(c, Self::parse2(c, &mut a)?)?)
    }
}
impl<T: Args> IArgs for T {}

pub trait Parse2
where
    Self: Sized,
{
    type Init: Debug + Clone;
    fn parse_strs<'args, 'arg, Ctx>(
        i: Init<Ctx, Self::Init>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'args, 'arg>,
    ) -> Result<Self, EParseArg<'arg>>;
    fn desc2<Ctx>(
        c: &Ctx,
        k: &'static str,
        d: &'static str,
        i: Init<Ctx, Self::Init>,
    ) -> [String; 4];
}
