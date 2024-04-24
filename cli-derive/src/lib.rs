use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};

#[proc_macro_derive(Args, attributes(ctx))]
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
    let field_names2 = field_names.clone();
    let field_names3 = field_names.clone();
    let field_types = fields.iter().map(|f| &f.ty);

    let result_ty_name = format!("Result{}", ident);
    let result_ty = Ident::new(&result_ty_name, ident.span());
    let result_ty2 = result_ty.clone();

    let f = || -> Ident {
        for attr in &pi.attrs {
            if attr.path().is_ident("ctx") {
                let r: Ident = attr.parse_args().unwrap();
                return r;
            }
        }
        panic!("Attribute 'ctx' is missing.")
    };
    let ctx = f();

    let o = quote! {
        #[derive(Debug)]
        struct #result_ty {
            #(
                #field_names: <#field_types as Args<#ctx>>::R,
            )*
        }

        impl Args<#ctx> for #ident {
            type R = #result_ty2;

            fn parse(&self, name: &'static str, c: &#ctx, am: &mut ArgMap) -> ParseResult<Self::R> {
                Ok(Self::R {
                    #(
                        #field_names2: Args::parse(&self.#field_names2, stringify!(#field_names2), c, am)?,
                    )*
                })
            }

            fn desc(&self, name: &'static str, c: &#ctx) -> Vec<(String, String)> {
                let mut r = Vec::<(String,String)>::new();
                #(
                    r.extend(Args::desc(&self.#field_names3, stringify!(#field_names3), c));
                )*
                r
            }
        }
    };
    o.into()
}
