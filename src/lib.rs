use std::num::NonZeroU32;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Block, ExprClosure, Ident, Result, Token,
};

mod kw {
    syn::custom_keyword!(STAGE);
    syn::custom_keyword!(INPUT);
    syn::custom_keyword!(OUTPUT);
    syn::custom_keyword!(REPLICATE);
}

struct ToStream {
    input: Punctuated<Ident, Token![,]>,
    output: Punctuated<Ident, Token![,]>,
    content: Block,
}

impl Parse for ToStream {
    fn parse(stream: ParseStream) -> Result<Self> {
        let (input, output, _, content) = parse_stage_args(&stream)?;

        Ok(Self {
            input,
            output,
            content,
        })
    }
}

struct Stage {
    input: Punctuated<Ident, Token![,]>,
    output: Punctuated<Ident, Token![,]>,

    //TODO: replicate can be a variable
    replicate: Option<NonZeroU32>,
    content: Block,
}

impl Parse for Stage {
    fn parse(stream: ParseStream) -> Result<Self> {
        stream.parse::<kw::STAGE>()?;
        let args;
        parenthesized!(args in stream);

        let (input, output, replicate, content) = parse_stage_args(&args)?;

        Ok(Self {
            input,
            output,
            replicate,
            content,
        })
    }
}

/// IMPORTANT: this assumes the parenthesis '()' have already been parsed by calling the
/// `parenthesized!` macro
fn parse_stage_args(
    args: ParseStream,
) -> Result<(
    Punctuated<Ident, Token![,]>,
    Punctuated<Ident, Token![,]>,
    Option<NonZeroU32>,
    Block,
)> {
    let mut input = Punctuated::new();
    let mut output = Punctuated::new();
    let mut replicate = None;
    let mut content: Option<Block> = None;

    while !args.is_empty() {
        if args.peek(kw::INPUT) {
            args.parse::<kw::INPUT>()?;
            if !input.is_empty() {
                return Err(
                    args.error("cannot have multiple 'INPUT' declarations in the same STAGE")
                );
            }
            let input_args;
            parenthesized!(input_args in args);
            input = input_args.parse_terminated(Ident::parse)?;
        } else if args.peek(kw::OUTPUT) {
            args.parse::<kw::OUTPUT>()?;
            if !output.is_empty() {
                return Err(
                    args.error("cannot have multiple 'OUTPUT' declarations in the same STAGE")
                );
            }
            let output_args;
            parenthesized!(output_args in args);
            output = output_args.parse_terminated(Ident::parse)?;
        } else if args.peek(kw::REPLICATE) {
            args.parse::<kw::REPLICATE>()?;
            if replicate.is_some() {
                return Err(
                    args.error("cannot have multiple 'REPLICATE' declarations in the same STAGE")
                );
            }
            args.parse::<Token![=]>()?;

            let integer = args.parse::<syn::LitInt>()?;
            let integer = integer.base10_parse::<u32>()?;
            if integer == 0 {
                return Err(args.error("'REPLICATE' cannot have an argument of '0'"));
            } else {
                replicate = Some(NonZeroU32::new(integer).unwrap());
            }
        } else {
            let block = args.parse::<Block>()?;
            if content.is_some() {
                return Err(args.error("cannot have multiple blocks of code"));
            } else {
                content = Some(block);
            }
        }

        if args.peek(Token![,]) {
            args.parse::<Token![,]>()?;
        }
    }
    if content.is_none() {
        return Err(args.error("missing block of code"));
    }
    Ok((input, output, replicate, content.unwrap()))
}

#[proc_macro]
pub fn to_stream(item: TokenStream) -> TokenStream {
    let ToStream {
        input,
        output,
        content: _,
    } = parse_macro_input!(item as ToStream);
    let input = input.into_iter().collect::<Vec<Ident>>();
    let output = output.into_iter().collect::<Vec<Ident>>();

    let codegen = quote! {
        #( println!("{}", #input));*
        ;
        #( println!("{}", #output));*
        ;
    };

    TokenStream::from(codegen)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    //use super::*;

    #[test]
    fn should_compile() {
        let t = trybuild::TestCases::new();
        t.pass("compile-tests/correct_syntax.rs");
    }

    #[test]
    fn should_not_compile() {
        let t = trybuild::TestCases::new();
        let files = Path::new("compile-tests/incorrect_syntax")
            .read_dir()
            .unwrap();
        for file in files {
            t.compile_fail(file.unwrap().path());
        }
    }
}
