use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(Args)]
pub fn args(i: TokenStream) -> TokenStream {
    let pi = parse_macro_input!(i as DeriveInput);
    let fields = match &pi.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("Struct fields must be named."),
    };

    let ident = &pi.ident;
    let field_names = fields.iter().map(|f| &f.ident);

    let o = quote! {
        impl #ident for Args {
            fn parse(c: Ctx, t: &Vec<String>) -> Self {
                Self {
                    #(
                        #field_names: parse_arg(c, stringify!(#field_names), t),
                    )*
                }
            }
        }
    };
    o.into()
}
