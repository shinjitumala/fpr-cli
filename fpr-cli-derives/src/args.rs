use super::prelude::*;

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

fn is_arg(a: &Arg) -> bool {
    !(a.desc.is_none() && a.i.is_none())
}

fn a(p: Args) -> Result<TokenStream, String> {
    let i = &p.ident;
    let k = LitStr::new(i.to_string().to_case(Case::Kebab).as_str(), i.span());

    enum F {
        Args(Ident, Type),
        Arg(Ident, Type, LitStr, Expr, LitStr),
    }

    let f: Vec<_> = p
        .data
        .take_struct()
        .ok_or(format!("Expected a struct."))?
        .fields
        .into_iter()
        .map(|f| {
            let b = is_arg(&f);
            let i = f.ident.unwrap();
            let ty = f.ty;
            if !b {
                F::Args(i, ty)
            } else {
                let desc = f.desc.unwrap();
                let init = f.i.unwrap();
                let k = LitStr::new(
                    format!("--{}", i.to_string().to_case(Case::Kebab)).as_str(),
                    i.span(),
                );
                F::Arg(i, ty, desc, init, k)
            }
        })
        .collect();
    let desc = &p.desc;

    let parse = f
        .iter()
        .map(|a| {
            match a {
                F::Args(ident, _) => 
                    quote! { #ident: Args::new(c, args)? },
                F::Arg(ident, ty, _, init, k) => 
                    quote! { #ident: <#ty>::parse2(#init, #k, c, args).map_err(|e| ArgsParseErr::Arg(#k.to_string(), e, Self::usage(c)))? },
            }
        })
        .collect::<Vec<_>>();

    let descs = f
        .iter()
        .map(|a| match a {
            F::Args(_, ty) => quote! {
                #ty::add_usage(c, r)
            },
            F::Arg(_, ty, desc, init, k) => quote! {
                r.push(<#ty>::desc2(#init, #desc, #k, c))
            },
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl Args<C> for #i{
            fn new(c: &C, args: &mut ParsedArgs) -> Result<Self, ArgsParseErr>{
                Ok(Self{
                   #( #parse, )*
                })
            }
            fn desc_act() -> &'static str { #desc }
            fn add_usage(c: &C, r: &mut Vec<[String; 4]>) {
                #( #descs; )*
            }
            fn add_paths(pfx: &mut Vec<String>, p: &mut Vec<Vec<String>>) {
                pfx.push(#k.to_string());
                p.push(pfx.to_owned());
            }
        }
    }
    .into())
}

pub fn f(i: TokenStream) -> TokenStream {
    let p = match Args::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    match a(p) {
        Ok(p) => p,
        Err(e) => {
            println!("{e}");
            quote! {}.into()
        }
    }
}
