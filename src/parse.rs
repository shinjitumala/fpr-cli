use crate::com::*;

pub const PFX: &'static str = "--";
#[derive(Debug)]
pub struct Key<'a> {
    pub k: Option<&'a str>,
    pub v: Vec<&'a str>,
    pub used: bool,
}
#[derive(Debug)]
pub struct ParsedArgs<'c> {
    pub k: Vec<Key<'c>>,
}

impl<'c> ParsedArgs<'c> {
    pub fn consume(&mut self, name: &str) -> Option<&Vec<&'c str>> {
        let k = self
            .k
            .iter_mut()
            .find(|k| k.k.map(|k| k == name).unwrap_or(false))?;
        assert!(
            !k.used,
            "A token should not ever be consumed twice. Probably a duplicate argument: {name}"
        );
        k.used = true;
        Some(&k.v)
    }
}

impl<'c> ParsedArgs<'c> {
    pub fn new(args: &[&'c str]) -> Self {
        let mut last: Option<&str> = None;
        let r = ParsedArgs {
            k: args
                .iter()
                .map(|a| {
                    if a.starts_with(PFX) {
                        last = Some(a);
                    }
                    (last, *a)
                })
                .into_group_map()
                .into_iter()
                .map(|(k, v)| Key {
                    k,
                    v: if k.is_some() { v[1..].into() } else { v },
                    used: false,
                })
                .collect(),
        };

        r
    }
}

pub enum Init<C, T: Display> {
    None,
    Const(T),
    Dyn(fn(&C) -> T),
}

impl<C, T: Display> Init<C, T> {
    pub fn get(self, c: &C) -> Option<T> {
        match self {
            Init::None => None,
            Init::Const(v) => Some(v),
            Init::Dyn(f) => Some(f(&c)),
        }
    }
    fn to_string(self, c: &C) -> String {
        match self.get(c) {
            Some(s) => format!(" (default: {s})"),
            None => format!(""),
        }
    }
}

#[derive(Error, Debug)]
pub enum ErrArg {
    #[error("Failed while parsing values for '{arg}': {e}")]
    Val { arg: String, e: ErrVal },
    #[error("'{arg}' must be one value.")]
    OnlyOne { arg: String },
    #[error("'{arg}' is required.")]
    Required { arg: String },
    #[error("'{arg}' requires at least one value.")]
    AtLeastOne { arg: String },
}

pub trait Parse2<'a, 'b, C>
where
    Self: Sized,
    Self::I: Display,
{
    type I;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs<'b>,
    ) -> Result<Self, ErrArg>;
    fn desc2(i: Init<C, Self::I>, d: &'static str, k: &'static str, c: &C) -> [String; 4];
    fn default2(c: &C, i: Init<C, Self::I>) -> Self;
}

impl<'a, 'b, C, T: Parse<'a> + Default> Parse2<'b, 'a, C> for T {
    type I = T;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ErrArg> {
        match p.consume(k) {
            Some(args) => {
                if args.len() != 1 {
                    Err(ErrArg::OnlyOne { arg: k.into() })
                } else {
                    Ok(T::parse(&args[0]).map_err(|e| ErrArg::Val { arg: k.into(), e })?)
                }
            }
            None => Ok(i.get(c).ok_or(ErrArg::Required { arg: k.into() })?),
        }
    }

    fn desc2(i: Init<C, Self>, d: &'static str, k: &'static str, c: &C) -> [String; 4] {
        [
            k.into(),
            format!("Req<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
    fn default2(c: &C, i: Init<C, Self::I>) -> Self {
        i.get(c).unwrap_or(Self::default())
    }
}

impl<'a, 'b, Ctx, T: Parse<'a>> Parse2<'b, 'a, Ctx> for Option<T> {
    type I = T;
    fn parse2(
        i: Init<Ctx, T>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ErrArg> {
        match p.consume(k) {
            Some(args) => {
                if args.len() != 1 {
                    Err(ErrArg::OnlyOne { arg: k.into() })
                } else {
                    Ok(Some(
                        T::parse(&args[0]).map_err(|e| ErrArg::Val { arg: k.into(), e })?,
                    ))
                }
            }
            None => Ok(match i.get(c) {
                Some(e) => Some(e),
                None => None,
            }),
        }
    }
    fn desc2(i: Init<Ctx, T>, d: &'static str, k: &'static str, c: &Ctx) -> [String; 4] {
        [
            k.into(),
            format!("Opt<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
    fn default2(c: &Ctx, i: Init<Ctx, Self::I>) -> Self {
        match i.get(c) {
            Some(i) => Some(i),
            None => None,
        }
    }
}

impl<'a, 'b, Ctx, T: Parse<'a> + Display> Parse2<'b, 'a, Ctx> for Vec<T> {
    type I = DisplayVec<T>;
    fn parse2(
        i: Init<Ctx, Self::I>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ErrArg> {
        match p.consume(k) {
            Some(args) => {
                let args = args
                    .iter()
                    .map(|a| T::parse(a).map_err(|e| ErrArg::Val { arg: k.into(), e }))
                    .process_results(|i| i.collect_vec())?;
                if args.is_empty() {
                    Err(ErrArg::OnlyOne { arg: k.into() })?;
                }
                Ok(args)
            }
            None => Ok(match i.get(c) {
                Some(e) => e.into(),
                None => vec![],
            }),
        }
    }

    fn desc2(i: Init<Ctx, Self::I>, d: &'static str, k: &'static str, c: &Ctx) -> [String; 4] {
        [
            k.into(),
            format!("Vec<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
    fn default2(c: &Ctx, i: Init<Ctx, Self::I>) -> Self {
        match i.get(c) {
            Some(v) => v.0,
            None => Self::default(),
        }
    }
}

#[derive(Debug)]
pub struct OptVec<T: Display>(pub Vec<T>);
impl<T: Display> From<Vec<T>> for OptVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}
impl<T: Display> From<DisplayVec<T>> for OptVec<T> {
    fn from(v: DisplayVec<T>) -> Self {
        Self(v.0)
    }
}
impl<'a, 'b, Ctx, T: Parse<'a>> Parse2<'b, 'a, Ctx> for OptVec<T> {
    type I = DisplayVec<T>;
    fn parse2(
        i: Init<Ctx, Self::I>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ErrArg> {
        match p.consume(k) {
            Some(args) => {
                let args = args
                    .iter()
                    .map(|a| T::parse(a).map_err(|e| ErrArg::Val { arg: k.into(), e }))
                    .process_results(|i| i.collect_vec())?;
                Ok(args.into())
            }
            None => Ok(match i.get(c) {
                Some(e) => e.into(),
                None => vec![].into(),
            }),
        }
    }

    fn desc2(i: Init<Ctx, Self::I>, d: &'static str, k: &'static str, c: &Ctx) -> [String; 4] {
        [
            k.into(),
            format!("Vec<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
    fn default2(c: &Ctx, i: Init<Ctx, Self::I>) -> Self {
        Self::from(match i.get(c) {
            Some(v) => v.0,
            None => <Vec<T>>::default(),
        })
    }
}
impl Display for DirExist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}
impl Display for FileExist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}
