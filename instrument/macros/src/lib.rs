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
    data_expression: Option<Expr>,
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

    let InstrumentParams {
        main,
        data_expression,
    } = InstrumentParams::from_list(&attr_args).unwrap();

    let ItemFn {
        vis, sig, block, ..
    } = parse_macro_input!(item as ItemFn);
    let Signature { ident, output, .. } = &sig;

    let func_name = ident.to_string();

    let mut modified_fn_body = quote!();
    let is_main = main == Some(true);

    let init_expression = if let Some(data_expression) = data_expression {
        quote! {
            let __profiler_data_processed = #data_expression;
            let mut __profiler_entry = instrument::profiler::ProfilerEntry::CodeBlock(instrument::profiler::ProfilerEntryData::init_with_throughput(#func_name, __profiler_data_processed));
        }
    } else {
        quote! {
            let mut __profiler_entry = instrument::profiler::ProfilerEntry::CodeBlock(instrument::profiler::ProfilerEntryData::init(#func_name));
        }
    };

    if is_main {
        modified_fn_body.extend(quote!({
            instrument::profiler::GlobalProfilerWrapper::start();
        }));
    } else {
        modified_fn_body.extend(quote! {
            #init_expression
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
    data_expression: Option<Expr>,
}

impl Parse for InstrumentBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let identifier: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let expression: Expr = input.parse()?;

        let data_expression = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            input.parse::<Expr>().ok()
        } else {
            None
        };

        Ok(InstrumentBlock {
            identifier,
            expression,
            data_expression,
        })
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn instrument_block(input: TokenStream) -> TokenStream {
    let InstrumentBlock {
        identifier,
        expression,
        data_expression,
    } = parse_macro_input!(input as InstrumentBlock);

    let mut init_block = quote!();

    if let Some(data_expression) = data_expression {
        init_block.extend(quote! {
            let __profiler_data_processed = #data_expression;
            let mut __profiler_entry = instrument::profiler::ProfilerEntry::CodeBlock(instrument::profiler::ProfilerEntryData::init_with_throughput(#identifier, __profiler_data_processed));
        });
    } else {
        init_block.extend(quote! {
            let mut __profiler_entry = instrument::profiler::ProfilerEntry::CodeBlock(instrument::profiler::ProfilerEntryData::init(#identifier));
        });
    }

    quote!(
        if cfg!(feature = "profile") {
            #init_block
            instrument::profiler::GlobalProfilerWrapper::push(&mut __profiler_entry);

            let result = {
                #expression
            };

            __profiler_entry.end();

            result
        } else {
            #expression
        }
    )
    .into()
}
