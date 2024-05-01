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

#[proc_macro_derive(Args, attributes(args, arg))]
pub fn args(i: TokenStream) -> TokenStream {
    let p = match Args::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &p.ident;
    let fields = p.data.as_ref().take_struct().expect("").fields;
    let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let result_ty_name = format!("Result{}", ident);
    let result_ty = Ident::new(&result_ty_name, ident.span());
    let ctx = &p.ctx;

    let attr_fields = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if a.desc.is_none() && a.i.is_none() {
                return quote! {
                    #ident: #ty::default()
                };
            };

            let desc = a.desc.as_ref().unwrap();
            let i = a.i.as_ref().unwrap();

            quote! {
                #ident: Arg::new((#desc).into(),(#i).into())
            }
        })
        .collect::<Vec<_>>();

    let o = quote! {
        #[derive(Debug)]
        pub struct #result_ty {
            #(
                pub #field_names: <#field_types as Args<#ctx>>::R,
            )*
        }

        impl Args<#ctx> for #ident {
            type R = #result_ty;

            fn parse(&self, name: &'static str, c: &#ctx, am: &mut ArgMap) -> ParseResult<Self::R> {
                Ok(Self::R {
                    #(
                        #field_names: Args::parse(&self.#field_names, stringify!(#field_names), c, am)?,
                    )*
                })
            }

            fn desc(&self, name: &'static str, c: &#ctx) -> Vec<(String, String)> {
                let mut r = Vec::<(String,String)>::new();
                #(
                    r.extend(Args::desc(&self.#field_names, stringify!(#field_names), c));
                )*
                r
            }
        }

        impl Default for #ident {
            fn default() -> Self {
                Self {
                    #(
                        #attr_fields,
                    )*
                }
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

#[proc_macro_derive(Acts, attributes(acts, act))]
pub fn argmap(i: TokenStream) -> TokenStream {
    let p = match Acts::from_derive_input(&parse_macro_input!(i as DeriveInput)) {
        Ok(p) => p,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &p.ident;
    let fields = p.data.as_ref().take_struct().expect("").fields;
    let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let ctx = &p.ctx;
    let desc = &p.desc;

    let attr_fields = fields
        .iter()
        .map(|a| {
            let ident = a.ident.as_ref().unwrap();
            let ty = &a.ty;
            if a.desc.is_none() && a.act.is_none() {
                return quote! {
                    #ident: #ty::default()
                };
            };

            let desc = a.desc.as_ref().unwrap();
            let i = a.act.as_ref().unwrap();

            quote! {
                #ident: Act::new(#desc, #i)
            }
        })
        .collect::<Vec<_>>();

    let o = quote! {
        impl ActPath<#ctx> for #ident {
            fn next(&self, c: &#ctx, pfx: String, rest: Vec<String>) -> Result<(), String>{
                let usage = (||{
                    let desc = vec![
                        #(
                            (format!(stringify!(#field_names)), format!("{}", self.#field_names.desc())),
                        )*
                    ];
                    let v = print_table(&desc);
                    format!("Available actions:\n{}", v)
                })();
                if(rest.len() == 0){
                    return Err(format!("Expected an action in '{}'\n{}", pfx, &usage))
                }
                let next_rest = rest.clone().drain(1..).collect::<Vec<_>>();
                let next = rest[0].to_owned();
                let next_pfx = match pfx.as_str(){
                    "" => next.to_owned(),
                    _ => format!("{} {}", pfx, next),
                };
                match next.as_str() {
                    #(
                        stringify!(#field_names) => self.#field_names.next(c, next_pfx, next_rest),
                    )*
                    _ => Err(format!("'{}' is not an action in '{}'\n{}", next, pfx, &usage)),
                }
            }
            fn desc(&self) -> &'static str {
                #desc
            }
            fn list(pfx: Vec<String>, name: &'static str) -> Vec<Vec<String>> {
                let mut r = Vec::<Vec<String>>::new();
                let mut next_pfx =  pfx.to_owned();
                next_pfx.push(name.to_string());
                #(
                    r.extend(<#field_types as ActPath<#ctx>>::list(next_pfx.to_owned(), stringify!(#field_names)));
                )*
                r
            }
        }

        impl Acts<#ctx> for #ident where Self: ActPath<#ctx> {
            fn parse(c: &#ctx, args: &Vec<String>){
                let d = Self::default();
                let r = ActPath::<#ctx>::next(&d, c, format!(""), args.to_owned());
                match r {
                    Ok(_) => (),
                    Err(ref e) => {
                        println!("Parse error: {}", e);
                    },
                }
            }
            fn list() -> Vec<Vec<String>> {
                let d = Self::default();
                <Self as ActPath<#ctx>>::list(vec![], <Self as ActPath<#ctx>>::desc(&d))
            }
        }

        impl Default for #ident {
            fn default() -> Self {
                Self {
                    #(
                        #attr_fields,
                    )*
                }
            }
        }
    };
    o.into()
}
