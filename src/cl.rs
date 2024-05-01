use crate::cl2;
pub use cli_derive::*;
use itertools::Itertools;
pub use smart_default::SmartDefault;

type Tkns = Vec<String>;

pub struct Act<Ctx, A: cl2::Args<Ctx>> {
    desc: &'static str,
    act: fn(&Ctx, A::R) -> Result<(), String>,
}

pub const LIST_SEP: &'static str = "_";

// It's a bug?
// [rust - How to clone a function pointer - Stack Overflow](https://stackoverflow.com/questions/33454425/how-to-clone-a-function-pointer)
impl<Ctx, A: cl2::Args<Ctx> + Default> Clone for Act<Ctx, A> {
    fn clone(&self) -> Self {
        Act {
            desc: self.desc,
            act: self.act,
        }
    }
}

impl<Ctx, A: cl2::Args<Ctx> + Default> Act<Ctx, A> {
    pub fn new(desc: &'static str, act: fn(&Ctx, A::R) -> Result<(), String>) -> Self {
        Self { desc, act }
    }
}

pub trait ActPath<Ctx> {
    fn next(&self, c: &Ctx, pfx: String, rest: Vec<String>) -> Result<(), String>;
    fn desc(&self) -> &'static str;
    fn list(pfx: Vec<String>, name: &'static str) -> Vec<Vec<String>>;
}

impl<Ctx, A: cl2::Args<Ctx> + Default> ActPath<Ctx> for Act<Ctx, A> {
    fn next(&self, c: &Ctx, _: String, rest: Vec<String>) -> Result<(), String> {
        let a = cl2::parse2::<Ctx, A>(c, &rest)?;
        Ok((self.act)(c, a)?)
    }
    fn desc(&self) -> &'static str {
        self.desc
    }
    fn list(pfx: Vec<String>, name: &'static str) -> Vec<Vec<String>> {
        let mut r = pfx.to_owned();
        r.push(name.to_string());
        vec![r]
    }
}

pub fn print_table(d: &Vec<(String, String)>) -> String {
    let r0 = d
        .iter()
        .map(|v| v.0.len())
        .max()
        .expect("Data should not be empty.");
    let r1 = d
        .iter()
        .map(|v| v.1.len())
        .max()
        .expect("Data should not be empty.");
    d.iter()
        .map(|v| format!("{0:1$} {2:3$}", v.0, r0, v.1, r1))
        .join("\n")
}

pub trait Acts<Ctx> {
    fn parse(c: &Ctx, args: &Tkns) -> Result<(), String>;
    fn list() -> Vec<Vec<String>>;
}

pub fn parse<Ctx, A: Acts<Ctx>>(c: &Ctx, args: &Tkns) -> Result<(), String> {
    A::parse(c, args)
}
pub fn list<Ctx, A: Acts<Ctx>>() -> Vec<Vec<String>> {
    A::list()
}
