extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn make_dyn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let wrapper_name = if _attr.is_empty() {
        format_ident!("dyn_{}", fn_name)
    } else {
        let name_lit = parse_macro_input!(_attr as LitStr);
        format_ident!("{}", name_lit.value())
    };
    let expected_arg_count = input.sig.inputs.len();

    let mut arg_extractions = Vec::new();
    let mut arg_names = Vec::new();

    for (i, arg) in input.sig.inputs.iter().enumerate() {
        if let syn::FnArg::Typed(pat_type) = arg {
            let arg_type = &pat_type.ty;
            let arg_name = format_ident!("arg_{}", i);

            arg_extractions.push(quote! {
                let #arg_name: #arg_type =
                    args.get(#i)
                        .ok_or_else(|| format!("Missing argument {}", #i))?
                        .cast()?;
            });

            arg_names.push(arg_name);
        }
    }

    let expanded = quote! {
        #input

        // Fix: Use the public path for the slice arguments and return types
        pub fn #wrapper_name(args: &[crate::models::Value]) -> ::std::result::Result<crate::models::Value, ::std::string::String> {
            if args.len() != #expected_arg_count {
                return ::std::result::Result::Err(format!("Expected {} args, found {}", #expected_arg_count, args.len()));
            }

            #(#arg_extractions)*

            let result = #fn_name(#(#arg_names),*);

            ::std::result::Result::Ok(result.into())
        }
    };

    TokenStream::from(expanded)
}
