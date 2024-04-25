use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Expr, Fields,
    Ident, LitStr, Stmt, Type,
};

#[derive(Debug, FromField)]
#[darling(attributes(arg))]
struct Arg {
    desc: Option<Expr>,
    act: Option<Expr>,
    i: Option<Expr>,
    ident: Option<Ident>,
    ty: Type,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(args), supports(struct_named))]
struct Args {
    ctx: syn::Path,
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
            if a.act.is_none() {
                return quote! {
                    #ident: #ty::default()
                };
            };

            let mut o = String::from("Arg::new(");

            let desc = a.desc.as_ref().unwrap();
            let i = a.i.as_ref().unwrap();
            let act = a.act.as_ref().unwrap();

            quote! {
                #ident: Arg::new((#desc).into(),(#i).into())
            }
        })
        .collect::<Vec<_>>();
    // let attr_fields_descs = attr_fields
    //     .iter()
    //     .map(|f| f.desc.as_ref().unwrap())
    //     .collect::<Vec<_>>();
    // let attr_fields_acts = attr_fields
    //     .iter()
    //     .map(|f| f.act.as_ref().unwrap())
    //     .collect::<Vec<_>>();

    let o = quote! {
        #[derive(Debug)]
        struct #result_ty {
            #(
                #field_names: <#field_types as Args<#ctx>>::R,
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

        impl Default for #ident{
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

#[proc_macro_derive(Acts, attributes(ctx, desc))]
pub fn argmap(i: TokenStream) -> TokenStream {
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

    let f2 = || -> LitStr {
        for attr in &pi.attrs {
            if attr.path().is_ident("desc") {
                let r: LitStr = attr.parse_args().unwrap();
                return r;
            }
        }
        panic!("Attribute 'desc' is missing.")
    };
    let desc = f2();

    let o = quote! {
        impl ActPath<#ctx> for #ident {
            fn next(&self, c: &#ctx, pfx: String, rest: Vec<String>) -> Result<(), String>{
                let usage = (||{
                    let desc = vec![
                        #(
                            (format!(stringify!(#field_names2)), format!("{}", self.#field_names2.desc())),
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
        }
    };
    o.into()
}
