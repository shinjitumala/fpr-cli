mod prelude {
    pub use convert_case::{Case, Casing};
    pub use darling::{ast, FromDeriveInput, FromField};
    pub use proc_macro::TokenStream;
    pub use quote::quote;
    pub use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Expr, Ident, LitStr, Type};
}
mod acts;
mod args;
use prelude::*;

#[proc_macro_derive(Args, attributes(args, arg))]
pub fn args(i: TokenStream) -> TokenStream {
    args::f(i)
}

#[proc_macro_derive(Acts, attributes(acts, act))]
pub fn acts(i: TokenStream) -> TokenStream {
    acts::f(i)
}
