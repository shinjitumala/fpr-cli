use super::prelude::*;

#[derive(Debug, FromField)]
#[darling(attributes(act))]
struct Act {
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(acts), supports(struct_tuple))]
struct Acts {
    ident: Ident,
    desc: Option<LitStr>,
    data: ast::Data<(), Act>,
}

fn a(p: Acts) -> Result<TokenStream, String> {
    let i = &p.ident;
    let k = LitStr::new(i.to_string().to_case(Case::Kebab).as_str(), i.span());
    let f: Result<Vec<_>, _> = p
        .data
        .take_struct()
        .ok_or(format!("Expected a struct."))?
        .into_iter()
        .map(|f| -> Result<_, String> {
            Ok((
                match &f.ty {
                    Type::Path(ref p) => {
                        let i = p
                            .path
                            .get_ident()
                            .ok_or(format!("Expected a Path, not '{p:?}'"))?;
                        LitStr::new(i.to_string().to_case(Case::Kebab).as_str(), f.ty.span())
                    }
                    e => Err(format!("Expected a Path, not '{e:?}'"))?,
                },
                f.ty,
            ))
        })
        .collect();
    let (ks, tys): (Vec<_>, Vec<_>) = f?.into_iter().unzip();
    let desc = &p.desc;
    Ok(quote! {
        impl Acts<C> for #i {
            fn next_impl(c: &C, s: &mut ParseCtx, a: &Arg, args: &[Arg]) -> Result<(), ActsErr> {
                match a.as_str() {
                    #(  #ks => { s.pfx.push(#ks.into()); #tys::next(c, s, args) } , )*
                    o => return Err(ActsErr::UnknownAct(s.to_owned(), o.to_owned())),
                }
            }
            fn opts() -> Vec<&'static str> {
                vec![ #( #ks, )*  ]
            }
            fn desc_act() -> &'static str { #desc }
            fn usage_v() -> Vec<[&'static str; 2]> {
                vec![
                    #( [ #ks, #tys::desc_act() ], )*
                ]
            }
            fn add_paths(pfx: &mut Vec<String>, p: &mut Vec<Vec<String>>) {
                pfx.push(#k.to_string());
                #( #tys::add_paths(pfx, p); )*
            }
        }
    }
    .into())
}

pub fn f(i: TokenStream) -> TokenStream {
    let p = match Acts::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
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
