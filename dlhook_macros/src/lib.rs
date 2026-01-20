use proc_macro::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use std::{collections::VecDeque, ffi::CString};
use syn::{
    Abi, BareFnArg, BareVariadic, FnArg, Ident, Lit, LitCStr, LitStr, MetaNameValue, Pat, Token,
    Type, TypeBareFn, parse,
    punctuated::{Pair, Punctuated},
    token::Comma,
};

fn parse_attr(tokens: TokenStream) -> Option<String> {
    let Ok(attr): Result<MetaNameValue, _> = parse(tokens) else {
        return None;
    };
    if attr.path.get_ident().is_none_or(|ident| ident != "origin") {
        return None;
    }
    match attr.value {
        syn::Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) => Some(s.value()),
        _ => None,
    }
}

#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn dlhook(attr: TokenStream, content: TokenStream) -> TokenStream {
    let macro_span = Span::call_site().into();
    let Some(target) = parse_attr(attr) else {
        return TokenStream::from(quote_spanned! {macro_span =>
            compile_error!("please provide origin function name to dlhook macro, like '#[dlhook(origin = \"getuid\")'");
        });
    };

    let Ok(mut f): Result<syn::ItemFn, _> = syn::parse(content) else {
        return TokenStream::from(quote_spanned! { macro_span =>
            compile_error!("dlhook can only apply on functions");
        });
    };
    if f.sig.inputs.empty_or_trailing() {
        let span = f.sig.ident.span();
        return TokenStream::from(quote_spanned! { span =>
            compile_error!("dlhook functions must has at least one argument(for receive the original function pointer)");
        });
    }
    match f.sig.inputs.first().unwrap() {
        FnArg::Typed(typed) if matches!(&*typed.ty, Type::Infer(_)) => (),
        _ => {
            let span = f.sig.ident.span();
            return TokenStream::from(quote_spanned! { span =>
                compile_error!("dlhook function's first argument must have InferType(\"arg: _\")");
            });
        }
    }

    let mut fn_args = Punctuated::new();
    for mut item in &mut f.sig.inputs.pairs_mut().skip(1) {
        let typed = match item.value_mut() {
            FnArg::Typed(item) => item,
            FnArg::Receiver(item) => {
                let span = item.self_token.span;
                return TokenStream::from(quote_spanned! {span =>
                    compile_error!("dlhook can not apply on non-static functions");
                });
            }
        };
        fn_args.push_value(BareFnArg {
            attrs: Vec::new(),
            name: match &*typed.pat {
                Pat::Ident(ident) => Some((ident.ident.clone(), Token![:](ident.ident.span()))),
                _ => None,
            },
            ty: *typed.ty.clone(),
        });
        if let Some(p) = item.punct() {
            fn_args.push_punct(**p);
        }
    }

    let fn_type = TypeBareFn {
        lifetimes: None,
        unsafety: Some(Token![unsafe](macro_span)),
        abi: Some(Abi {
            extern_token: Token![extern](macro_span),
            name: Some(LitStr::new("C", macro_span)),
        }),
        fn_token: Token![fn](macro_span),
        paren_token: f.sig.paren_token,
        output: f.sig.output.clone(),
        inputs: fn_args.clone(),
        variadic: f.sig.variadic.as_ref().map(|item| BareVariadic {
            attrs: Vec::new(),
            name: None,
            dots: item.dots,
            comma: item.comma,
        }),
    };

    if let FnArg::Typed(item) = f.sig.inputs.pairs_mut().next().unwrap().into_value() {
        *item.ty = Type::BareFn(fn_type.clone());
    }

    let hook_name = Ident::new(&format!("__dlhook_{}", f.sig.ident), macro_span);
    let link_name = LitStr::new(&target, macro_span);
    let c_link_name = LitCStr::new(CString::new(target).unwrap().as_c_str(), macro_span);
    let hook_ident = f.sig.ident.clone();
    let ret = f.sig.output.clone();
    let mut arg_names = VecDeque::new();
    let i_args: Punctuated<BareFnArg, Comma> = fn_args
        .pairs()
        .map(|pair| {
            let (item, punct) = pair.into_tuple();
            let mut item = item.clone();
            if item.name.is_none() {
                item.name = Some((
                    Ident::new(&format!("arg{}", arg_names.len()), macro_span),
                    Token![:](macro_span),
                ));
            }
            arg_names.push_back(item.name.clone().unwrap().0);
            Pair::new(item, punct.copied())
        })
        .collect();
    let mut args: Punctuated<Ident, Comma> =
        arg_names
            .into_iter()
            .fold(Punctuated::new(), |mut acc, item| {
                acc.push_value(item);
                acc.push_punct(Token![,](macro_span));
                acc
            });
    args.pop_punct();
    let hook_fn = quote_spanned! {macro_span =>
        #[unsafe(export_name = #link_name)]
        pub extern "C" fn #hook_name(#i_args) #ret {
            unsafe {
                #hook_ident(
                    ::core::mem::transmute::<*mut ::core::ffi::c_void, #fn_type>(
                        ::dlhook::__hidden::dlsym(::dlhook::__hidden::RTLD_NEXT, ::core::ffi::CStr::as_ptr(#c_link_name))
                    ),
                #args)
            }
        }
    };

    TokenStream::from(
        quote! {
            #f

            #hook_fn
        }
        .into_token_stream(),
    )
}
