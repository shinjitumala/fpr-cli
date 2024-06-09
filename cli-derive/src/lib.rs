use darling::{ast, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Ident, LitStr, Path, Type};

#[derive(Debug, FromField)]
#[darling(attributes(arg))]
struct Arg {
    desc: Option<LitStr>,
    i: Option<Expr>,
    ident: Option<Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(args), supports(struct_named))]
struct Args {
    desc: LitStr,
    ident: Ident,
    data: ast::Data<(), Arg>,
}

enum ArgKind {
    Arg(Ident, Type, LitStr, Expr),
    Args(Ident, Type),
    KindErr,
}

fn arg_kind(a: Arg) -> ArgKind {
    use ArgKind::*;

    let ident = {
        if let Some(i) = a.ident {
            i
        } else {
            println!("Field does not have an identifier");
            return KindErr;
        }
    };

    if a.desc.is_none() && a.i.is_none() {
        Args(ident, a.ty)
    } else if let (Some(desc), Some(i)) = (a.desc, a.i) {
        Arg(ident, a.ty, desc, i)
    } else {
        println!(
            "You must defind both 'desc' and 'i' for an Arg. Otherwise, do not define anything."
        );
        KindErr
    }
}

#[proc_macro_derive(Args, attributes(args, arg))]
pub fn args(i: TokenStream) -> TokenStream {
    let p = match Args::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &p.ident;
    let fields: Vec<_> = p
        .data
        .take_struct()
        .expect("")
        .fields
        .into_iter()
        .map(arg_kind)
        .collect();
    let act_desc = &p.desc;

    let parsers = match fields
        .iter()
        .map(|a| -> Result<_, ()> {
            use ArgKind::*;
            match a {
                Arg(ident, _, _, i) => {
                    Ok(quote! { #ident: parse_arg(c, stringify!(#ident), #i, a)? })
                }
                Args(ident, ty) => Ok(quote! { #ident: #ty::parse2(c, a)? }),
                KindErr => Err(()),
            }
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) => v,
        Err(()) => return quote! {}.into(),
    };

    let descs = match fields
        .iter()
        .map(|a| {
            use ArgKind::*;
            match a {
                Arg(ident, ty, desc, i) => {
                    Ok(quote! { r.push(<#ty>::desc2(c, stringify!(#ident), #desc, #i)) })
                }
                Args(_, ty) => Ok(quote! { <#ty>::desc(c, r) }),
                KindErr => Err(()),
            }
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(o) => o,
        Err(()) => return quote! {}.into(),
    };

    let o = quote! {
        impl Args for #ident {
            fn parse2<'args, 'arg, C>(c: &C, a: &mut ParsedArgs<'args,'arg>) -> Result<Self, EParseArgs<'arg>> where Self: Run<C> {
                Ok(Self {
                    #(
                        #parsers,
                    )*
                })
            }
            fn desc<C>(c: &C, r: &mut Vec<[String ;4]>) {
                #(
                    #descs;
                )*
            }
            fn act_desc() -> &'static str {
                #act_desc
            }
        }
    };
    o.into()
}

#[derive(Debug, FromField)]
#[darling(attributes(act))]
struct Act {
    ident: Option<Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(acts), supports(struct_named))]
struct Acts {
    ident: Ident,
    desc: LitStr,
    data: ast::Data<(), Act>,
}

#[proc_macro_derive(Acts, attributes(acts, act))]
pub fn acts(i: TokenStream) -> TokenStream {
    let p = match Acts::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &p.ident;
    let fields = p.data.as_ref().take_struct().expect("").fields;
    let desc = &p.desc;

    let parsers = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            quote! {
                stringify!(#ident) => <#ty>::parse(c, next_args)?,
            }
        })
        .collect::<Vec<_>>();
    let descs = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            quote! {
                [stringify!(#ident), #ty::act_desc()]
            }
        })
        .collect::<Vec<_>>();

    let lists = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            // if !is_act(a) {
            //     return quote! {
            //         r.extend(#ty::list(&(||{
            //             let mut a = pfx.clone();
            //             a.push(format!(stringify!(#ident)));
            //             a
            //         })()));
            //     };
            // };

            quote! {
                r.push((||{
                        let mut a = pfx.clone();
                        a.push(format!(stringify!(#ident)));
                        a
                    })());
            }
        })
        .collect::<Vec<_>>();

    let o = quote! {
        impl Acts for #ident {
            fn next<'args, 'arg, Ctx>(
                c: &Ctx,
                next: &'arg str,
                next_args: &'args [&'arg str],
            ) -> Result<(), EActs<'arg>> {
                Ok(match next {
                    #(
                        #parsers
                    )*
                    ref a => Err(EActs{
                        kind: EActsKind::NotAnAction(a),
                        stack: vec![],
                    })?,
                })
            }
            fn desc() -> Vec<[&'static str; 2]>{
                vec![#(
                    #descs,
                )*]
            }
            fn act_desc() -> &'static str{
                #desc
            }
        //     fn list(pfx: &Vec<String>) -> Vec<Vec<String>>{
        //         let mut r = Vec::<Vec<String>>::new();
        //         #(
        //             #lists
        //         )*
        //         r
        //     }
        }
    };
    o.into()
}
