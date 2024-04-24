use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident};

#[proc_macro_derive(SArg)]
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
    let field_types2 = field_types.clone();

    let result_ty_name = format!("Result{}", ident);
    let result_ty = Ident::new(&result_ty_name, ident.span());
    let result_ty2 = result_ty.clone();

    let o = quote! {
        #[derive(Debug)]
        struct #result_ty {
            #(
                #field_names: <#field_types as SArg>::R,
            )*
        }

        impl SArg for #ident {
            type R = #result_ty2;

            fn parse(name: &'static str, am: & mut ArgMap) -> Self::R {
                Self::R {
                    #(
                        #field_names2: <#field_types2 as SArg>::parse(stringify!(#field_names2), am),
                    )*
                }
            }
        }
    };
    o.into()
}
