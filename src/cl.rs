pub use cli_derive::*;
pub use smart_default::SmartDefault;

type Tkns = Vec<String>;

pub struct Act<Ctx> {
    desc: &'static str,
    act: fn(&Ctx, &Tkns),
}

// It's a bug?
// [rust - How to clone a function pointer - Stack Overflow](https://stackoverflow.com/questions/33454425/how-to-clone-a-function-pointer)
impl<Ctx> Clone for Act<Ctx> {
    fn clone(&self) -> Self {
        Act {
            desc: self.desc,
            act: self.act,
        }
    }
}

impl<Ctx> Act<Ctx> {
    pub fn new(desc: &'static str, act: fn(&Ctx, &Tkns)) -> Self {
        Self { desc, act }
    }
}

pub trait ActPath<Ctx> {
    fn next(&self, c: &Ctx, pfx: String, rest: Vec<String>) -> Result<(), String>;
}

impl<Ctx> ActPath<Ctx> for Act<Ctx> {
    fn next(&self, c: &Ctx, _: String, rest: Vec<String>) -> Result<(), String> {
        Ok((self.act)(c, &rest))
    }
}

pub trait Acts<Ctx> {
    fn parse(c: &Ctx, args: &Tkns);
}
