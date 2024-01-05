extern crate proc_macro;

use crate::syn::parse_macro_input;
pub(crate) use darling::export::syn;
use darling::export::syn::parse::{Parse, ParseStream};
use darling::export::syn::{Expr, LitStr, ReturnType};
use darling::export::syn::{ItemFn, Signature, Token};
use darling::export::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

#[derive(FromMeta)]
struct InstrumentParams {
    main: Option<bool>,
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn instrument(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let InstrumentParams { main } = InstrumentParams::from_list(&attr_args).unwrap();

    let ItemFn {
        vis, sig, block, ..
    } = parse_macro_input!(item as ItemFn);
    let Signature { ident, output, .. } = &sig;

    let func_name = ident.to_string();

    let mut modified_fn_body = quote!();
    let is_main = main == Some(true);

    if is_main {
        modified_fn_body.extend(quote!({
            instrument::profiler::GlobalProfilerWrapper::start();
        }));
    } else {
        modified_fn_body.extend(quote! {
            let mut __profiler_entry = instrument::profiler::ProfilerEntry::CodeBlock(instrument::profiler::ProfilerEntryData::init(#func_name));
            instrument::profiler::GlobalProfilerWrapper::push(&mut __profiler_entry);
        });
    }

    match output {
        ReturnType::Type(_, return_type) => {
            modified_fn_body.extend(quote! {
                let result: #return_type = #block;
            });
        }
        _ => {
            modified_fn_body.extend(quote! {
                let result = #block;
            });
        }
    };

    if is_main {
        modified_fn_body.extend(quote! {
            {
                instrument::profiler::GlobalProfilerWrapper::end();
            }

            return result;
        });
    } else {
        modified_fn_body.extend(quote! {
            {
                __profiler_entry.end();
            }

            return result;
        });
    }

    quote!(
        #vis #sig {
            #modified_fn_body
        }
    )
    .into()
}

#[derive(Debug)]
struct InstrumentBlock {
    identifier: LitStr,
    expression: Expr,
}

impl Parse for InstrumentBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let identifier: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let expression: Expr = input.parse()?;

        Ok(InstrumentBlock {
            identifier,
            expression,
        })
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn instrument_block(input: TokenStream) -> TokenStream {
    let InstrumentBlock {
        identifier,
        expression,
    } = parse_macro_input!(input as InstrumentBlock);

    quote!(
        if cfg!(feature = "profile") {
            let mut __profiler_entry = instrument::profiler::ProfilerEntry::CodeBlock(instrument::profiler::ProfilerEntryData::init(#identifier));
            instrument::profiler::GlobalProfilerWrapper::push(&mut __profiler_entry);

            let result = {
                #expression
            };

            __profiler_entry.end();

            result
        } else {
            #expression
        }
    ).into()
}
