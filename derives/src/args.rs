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

    let f = p
        .data
        .take_struct()
        .ok_or(format!("Expected a struct."))?
        .fields;
    let desc = &p.desc;

    let parse = f
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_arg(a) {
                return quote! {
                    #ident: Args::new(c, args)?
                };
            };

            let init = a.i.as_ref().unwrap();
            let k = LitStr::new( format!("--{}",ident.to_string()).as_str(),ident.span());

            quote! {
                #ident: <#ty>::parse2(#init, #k, c, args).map_err(|e| ArgsParseErr::Arg(#k.to_string(), e))?
            }
        })
        .collect::<Vec<_>>();

    let descs = f
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if !is_arg(a) {
                return quote! {
                    #ty::add_usage(c, r)
                };
            };

            let desc = a.desc.as_ref().unwrap();
            let init = a.i.as_ref().unwrap();
            let k = LitStr::new(format!("--{}", ident.to_string()).as_str(), ident.span());

            quote! {
                r.push(<#ty>::desc2(#init, #desc, #k, c))
            }
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
