use darling::{ast, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Ident, LitStr, Path, Type};

#[derive(Debug, FromField)]
#[darling(attributes(arg))]
struct Arg {
    desc: Option<Expr>,
    i: Option<Expr>,
    ident: Option<Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(args), supports(struct_named))]
struct Args {
    ctx: Path,
    ident: Ident,
    data: ast::Data<(), Arg>,
}

fn is_arg(a: &Arg) -> bool {
    !(a.desc.is_none() && a.i.is_none())
}

#[proc_macro_derive(Args, attributes(args, arg))]
pub fn args(i: TokenStream) -> TokenStream {
    let p = match Args::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &p.ident;
    let fields = p.data.as_ref().take_struct().expect("").fields;
    let ctx = &p.ctx;

    let parsers = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_arg(a) {
                return quote! {
                    #ident: Args::parse(c, p)?
                };
            };

            let desc = a.desc.as_ref().unwrap();
            let init = a.i.as_ref().unwrap();

            quote! {
                #ident: Parse2::parse2(&Arg::<#ctx, #ty>::new((#init).into(), (#desc).into()), concat!(PFX, stringify!(#ident)), c, p)?
            }
        })
        .collect::<Vec<_>>();

    let descs = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_arg(a) {
                return quote! {
                    r.extend(#ty::desc(c))
                };
            };

            let desc = a.desc.as_ref().unwrap();
            let init = a.i.as_ref().unwrap();

            quote! {
                r.push(Parse2::desc2(&Arg::<#ctx, #ty>::new((#init).into(), (#desc).into()), concat!(PFX, stringify!(#ident)), c))
            }
        })
        .collect::<Vec<_>>();

    let o = quote! {
        impl Args<#ctx> for #ident {
            fn parse(c: &#ctx, p: &mut ParsedArgs) -> Res<Self> {
                use constcat::concat;
                Ok(Self {
                    #(
                        #parsers,
                    )*
                })
            }
            fn desc(c: &#ctx) -> Vec<Vec<String>> {
                use constcat::concat;
                let mut r = Vec::<Vec<String>>::new();
                #(
                    #descs;
                )*
                r.sort_by(|l, r| { l[0].cmp(&r[0]) });
                if r.iter().find(|a| a[0] == "--help").is_none() {
                    r.push(vec!["--help".to_string(), format!(""), format!("Display this help."), format!("")]);
                }

                // Check unique
                let mut u = std::collections::HashSet::new();
                let all_unique = r.iter().all(|x| u.insert(x));
                assert!(all_unique, "There should not be duplicate args: {:?}", r);

                r
            }
        }
    };
    o.into()
}

#[derive(Debug, FromField)]
#[darling(attributes(act))]
struct Act {
    desc: Option<LitStr>,
    act: Option<Expr>,
    ident: Option<Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(acts), supports(struct_named))]
struct Acts {
    ident: Ident,
    ctx: Path,
    desc: LitStr,
    data: ast::Data<(), Act>,
}

fn is_act(a: &Act) -> bool {
    !(a.desc.is_none() && a.act.is_none())
}

#[proc_macro_derive(Acts, attributes(acts, act))]
pub fn argmap(i: TokenStream) -> TokenStream {
    let p = match Acts::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &p.ident;
    let fields = p.data.as_ref().take_struct().expect("").fields;
    let ctx = &p.ctx;
    let desc = &p.desc;

    let parsers = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_act(a) {
                return quote! {
                    stringify!(#ident) => #ty::parse(c, next_pfx, next_args),
                };
            }

            let act = a.act.as_ref().unwrap();

            quote! {
                stringify!(#ident) => ((#act)(c, parse2::<#ctx, #ty>(c, next_args)?)).map_err(|e| format!("Error: {e}\nUsage ({}):\n{}", stringify!(#ident), print_table(&#ty::desc(c)))),
            }
        })
        .collect::<Vec<_>>();
    let descs = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_act(a) {
                return quote! {
                    r.push(vec![stringify!(#ident).to_string(), #ty::act_desc().to_string()]);
                };
            };

            let desc = a.desc.as_ref().unwrap();

            quote! {
                r.push(vec![stringify!(#ident).to_string(), #desc.to_string()]);
            }
        })
        .collect::<Vec<_>>();

    let lists = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_act(a) {
                return quote! {
                    r.extend(#ty::list(&(||{
                        let mut a = pfx.clone();
                        a.push(format!(stringify!(#ident)));
                        a
                    })()));
                };
            };

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
        impl Acts<#ctx> for #ident {
            fn parse(c: &#ctx, pfx: Option<String>, args: &[String]) -> Res<()> {
                let desc = || {
                    print_table(&Self::desc())
                };
                if args.len() == 0 {
                    return Err(format!("{}Available Actions:\n{}", match pfx {
                        Some(e) => format!("Expected an action in '{e}'\n"),
                        None => format!(""),
                    }, desc()))
                };
                let next = &args[0];
                let next_args = &args[1..];
                let next_pfx = Some(match pfx {
                    Some(ref e) => {
                        let mut c = e.clone();
                        c.push_str(next);
                        c
                    },
                    None => format!("{next}"),
                });
                match next.as_str() {
                    #(
                        #parsers
                    )*
                    ref a => Err(format!("Not an action{}: {a}\nAvailable Actions:\n{}", match pfx {
                        Some(e) => format!(" in '{e}'"),
                        None => format!(""),
                    }, desc())),
                }?;
                Ok(())
            }
            fn act_desc() -> &'static str {
                #desc
            }
            fn desc() -> Vec<Vec<String>>{
                let mut r = Vec::<Vec<String>>::new();
                #(
                    #descs
                )*
                r
            }
            fn list(pfx: &Vec<String>) -> Vec<Vec<String>>{
                let mut r = Vec::<Vec<String>>::new();
                #(
                    #lists
                )*
                r
            }
        }
    };
    o.into()
}
