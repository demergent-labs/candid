use super::{candid_path, get_lit_str};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use std::sync::Mutex;
use syn::{Attribute, Error, ItemFn, Meta, Result, ReturnType, Signature, Type};

type RawArgs = Vec<String>;
type RawRets = Vec<String>;
type ParsedArgs = Vec<Type>;
type ParsedRets = Vec<Type>;

struct Method {
    args: RawArgs,
    rets: RawRets,
    modes: String,
    docs: Vec<String>,
}

// There is no official way to communicate information across proc macro invocations.
// lazy_static works for now, but may get incomplete info with incremental compilation.
// See https://github.com/rust-lang/rust/issues/44034
// Hopefully, we can have an attribute on impl, then we don't need global state.
lazy_static! {
    static ref METHODS: Mutex<Option<BTreeMap<String, Method>>> =
        Mutex::new(Some(BTreeMap::default()));
    static ref INIT: Mutex<Option<Option<RawArgs>>> = Mutex::new(Some(Option::default()));
}

pub(crate) fn candid_method(attrs: Vec<Meta>, fun: ItemFn) -> Result<TokenStream> {
    let attrs = get_candid_attribute(attrs)?;
    let sig = &fun.sig;
    if !sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &sig.generics,
            "candid_method doesn't support generic parameters",
        ));
    }
    let ident = sig.ident.to_string();
    let name = attrs.rename.as_ref().unwrap_or(&ident).clone();
    let modes = attrs.method_type.unwrap_or_else(|| "update".to_string());
    let (args, rets) = get_args(sig)?;
    let args: RawArgs = args
        .iter()
        .map(|t| format!("{}", t.to_token_stream()))
        .collect();
    let rets: RawRets = rets
        .iter()
        .map(|t| format!("{}", t.to_token_stream()))
        .collect();
    if modes == "oneway" && !rets.is_empty() {
        return Err(Error::new_spanned(
            &sig.output,
            "oneway function should have no return value",
        ));
    }
    if attrs.is_init {
        match (rets.len(), rets.first().map(|x| x.as_str())) {
            (0, None) | (1, Some("Self")) => {
                if let Some(init) = INIT.lock().unwrap().as_mut() {
                    *init = Some(args);
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    &sig.output,
                    "init method should have no return value or return Self",
                ))
            }
        }
    } else if let Some(map) = METHODS.lock().unwrap().as_mut() {
        let docs = extract_doc_comments(&fun.attrs);
        map.insert(
            name.clone(),
            Method {
                args,
                rets,
                modes,
                docs,
            },
        );
    }
    Ok(quote! { #fun })
}

pub(crate) fn export_service(path: Option<TokenStream>) -> TokenStream {
    if let Some(meths) = METHODS.lock().unwrap().as_mut() {
        let candid = candid_path(&path);
        let init = if let Some(opt_args) = INIT.lock().unwrap().as_mut() {
            let res = opt_args.as_ref().map(|args| {
                let args = args
                    .iter()
                    .map(|t| generate_arg(quote! { init_args }, t))
                    .collect::<Vec<_>>();
                quote! {
                    let mut init_args: Vec<Type> = Vec::new();
                    #(#args)*
                }
            });
            *opt_args = None;
            res
        } else {
            unreachable!();
        };
        let gen_tys = meths.iter().map(
            |(
                name,
                Method {
                    args,
                    rets,
                    modes,
                    docs: method_docs,
                },
            )| {
                let args = args
                    .iter()
                    .map(|t| generate_arg(quote! { args }, t))
                    .collect::<Vec<_>>();
                let rets = rets
                    .iter()
                    .map(|t| generate_ret(quote! { rets }, t))
                    .collect::<Vec<_>>();
                let modes = match modes.as_ref() {
                    "query" => quote! { vec![#candid::types::FuncMode::Query] },
                    "composite_query" => quote! { vec![#candid::types::FuncMode::CompositeQuery] },
                    "oneway" => quote! { vec![#candid::types::FuncMode::Oneway] },
                    "update" => quote! { vec![] },
                    _ => unreachable!(),
                };
                let doc_storage = if !method_docs.is_empty() {
                    quote! {
                        docs.add_service_method(#name.to_string(), vec![#(#method_docs.to_string(),)*]);
                    }
                } else {
                    quote! {}
                };
                quote! {
                    {
                        #doc_storage
                        let mut args: Vec<Type> = Vec::new();
                        #(#args)*
                        let mut rets: Vec<Type> = Vec::new();
                        #(#rets)*
                        let func = Function { args, rets, modes: #modes };
                        service.push((#name.to_string(), TypeInner::Func(func).into()));
                    }
                }
            },
        );
        let service = quote! {
            use #candid::types::{CandidType, Function, Type, TypeInner};
            let mut service = Vec::<(String, Type)>::new();
            let mut env = #candid::types::internal::TypeContainer::new();
            let mut docs = #candid::pretty::candid::DocComments::empty();
            #(#gen_tys)*
            service.sort_unstable_by_key(|(name, _)| name.clone());
            let ty = TypeInner::Service(service).into();
        };
        let actor = if let Some(init) = init {
            quote! {
                #init
                let actor = Some(TypeInner::Class(init_args, ty).into());
            }
        } else {
            quote! { let actor = Some(ty); }
        };
        let res = quote! {
            fn __export_service() -> String {
                #service
                #actor
                let result = #candid::pretty::candid::compile_with_docs(&env.env, &actor, &docs);
                format!("{}", result)
            }
        };
        meths.clear();
        //panic!(res.to_string());
        res
    } else {
        unreachable!()
    }
}

fn generate_arg(name: TokenStream, ty: &str) -> TokenStream {
    let ty = syn::parse_str::<Type>(ty).unwrap();
    quote! {
        #name.push(env.add::<#ty>());
    }
}

fn generate_ret(name: TokenStream, ty: &str) -> TokenStream {
    generate_arg(name, ty)
}

fn get_args(sig: &Signature) -> Result<(ParsedArgs, ParsedRets)> {
    let mut args = Vec::new();
    for arg in &sig.inputs {
        match arg {
            syn::FnArg::Receiver(r) => {
                if r.reference.is_none() {
                    return Err(Error::new_spanned(arg, "only works for borrowed self"));
                }
            }
            syn::FnArg::Typed(syn::PatType { ty, .. }) => args.push(ty.as_ref().clone()),
        }
    }
    let rets = match &sig.output {
        ReturnType::Default => Vec::new(),
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.iter().cloned().collect(),
            _ => vec![ty.as_ref().clone()],
        },
    };
    Ok((args, rets))
}

struct CandidAttribute {
    rename: Option<String>,
    method_type: Option<String>,
    is_init: bool,
}

fn get_candid_attribute(attrs: Vec<Meta>) -> Result<CandidAttribute> {
    let mut res = CandidAttribute {
        rename: None,
        method_type: None,
        is_init: false,
    };
    for attr in attrs {
        match &attr {
            Meta::NameValue(m) if m.path.is_ident("rename") && res.rename.is_none() => {
                if let Ok(lit) = get_lit_str(&m.value) {
                    res.rename = Some(lit.value());
                }
            }
            Meta::Path(p) if res.method_type.is_none() => {
                let mode = p.get_ident().unwrap().to_string();
                match mode.as_ref() {
                    "query" | "composite_query" | "update" | "oneway" => {
                        res.method_type = Some(mode);
                    }
                    "init" => res.is_init = true,
                    _ => return Err(Error::new_spanned(p, "unknown mode")),
                }
            }
            _ => return Err(Error::new_spanned(attr, "unknown or conflicting attribute")),
        }
    }
    Ok(res)
}

fn extract_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    let mut docs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let Ok(lit) = get_lit_str(&meta.value) {
                    let doc_content = lit.value();
                    if !doc_content.is_empty() {
                        for line in doc_content.lines() {
                            let trimmed = line.trim().to_string();
                            docs.push(trimmed);
                        }
                    } else {
                        docs.push("".to_string());
                    }
                }
            }
        }
    }
    docs
}
