use std::num::NonZeroU32;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Brace,
    Block, Ident, Result, Stmt, Token,
};

mod kw {
    syn::custom_keyword!(STAGE);
    syn::custom_keyword!(INPUT);
    syn::custom_keyword!(OUTPUT);
    syn::custom_keyword!(REPLICATE);
}

struct SparAttrs {
    input: Vec<Ident>,
    output: Vec<Ident>,
    replicate: Option<NonZeroU32>,
}

impl SparAttrs {
    pub fn new(input: Vec<Ident>, output: Vec<Ident>, replicate: Option<NonZeroU32>) -> Self {
        Self {
            input,
            output,
            replicate,
        }
    }
}

struct SparStage {
    attrs: SparAttrs,
    //code: TokenStream,
}

impl SparStage {
    pub fn new(attrs: SparAttrs) -> Self {
        Self { attrs }
    }
}

struct SparStream {
    attrs: SparAttrs,
    stages: Vec<SparStage>,
    code: TokenStream,
}

impl Parse for SparStream {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stages = Vec::new();
        let mut code = TokenStream::new();
        let (spar_input, spar_output, replicate) = parse_spar_args(&input)?;

        let attrs = SparAttrs::new(
            spar_input.into_iter().collect(),
            spar_output.into_iter().collect(),
            replicate,
        );

        let block;
        braced!(block in input);
        while !block.is_empty() {
            if !block.peek(kw::STAGE) {
                if block.peek(Brace) {
                   let b = block.parse::<Block>()?.into_token_stream();
                   code.extend::<TokenStream>(b.into());
                } else {
                    let s = block.parse::<Stmt>()?.into_token_stream();
                    code.extend::<TokenStream>(s.into());
                }
                continue;
            }
            block.parse::<kw::STAGE>()?;

            let stage_args;
            parenthesized!(stage_args in block);

            let (stage_input, stage_output, stage_replicate) = parse_spar_args(&stage_args)?;
            let stage_attrs = SparAttrs::new(
                stage_input.into_iter().collect(),
                stage_output.into_iter().collect(),
                stage_replicate,
            );

            let _b = stage_args.parse::<Block>()?; // This is necessary to empty the parser

            stages.push(SparStage::new(stage_attrs));
            block.parse::<Token![;]>()?;
        }

        if !input.is_empty() {
            return Err(input.error("unexpected trailing tokens"));
        }

        Ok(Self { attrs, code, stages })
    }
}

/// IMPORTANT: this assumes the parenthesis '()' have already been parsed by calling the
/// `parenthesized!` macro
/// Furthermore, after returning, 'args' should be at the code inside
fn parse_spar_args(
    args: ParseStream,
) -> Result<(
    Punctuated<Ident, Token![,]>,
    Punctuated<Ident, Token![,]>,
    Option<NonZeroU32>,
)> {
    let mut input = Punctuated::new();
    let mut output = Punctuated::new();
    let mut replicate = None;

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
        } else if args.peek(Brace) {
            return Ok((input, output, replicate));
        } else {
            return Err(args.error("unexpected token. Valid tokens are 'INPUT', 'OUTPUT', 'REPLICATE' and a code block"));
        }

        if args.peek(Token![,]) {
            args.parse::<Token![,]>()?;
        }
    }

    Err(args.error("missing block of code"))
}

#[proc_macro]
pub fn to_stream(item: TokenStream) -> TokenStream {
    let SparStream {
        attrs,
        code,
        stages,
    } = parse_macro_input!(item as SparStream);

    let input = &attrs.input;
    let output = &attrs.output;
    let replicate: u32 = attrs
        .replicate
        .unwrap_or(NonZeroU32::new(1).unwrap())
        .into();

    let mut codegen: TokenStream = quote! {
        println!("SparStream:");
        println!("\tInput:");
        #(println!("\t\t{}", #input));*;
        println!("\tOutput:");
        #(println!("\t\t{}", #output));*;
        println!("\tReplicate: {}", #replicate);
    }.into();

    codegen.extend(code);
    for (i, stage) in stages.iter().enumerate() {
        let input = &stage.attrs.input;
        let output = &stage.attrs.output;
        let replicate: u32 = stage
            .attrs
            .replicate
            .unwrap_or(NonZeroU32::new(1).unwrap())
            .into();

        codegen.extend::<TokenStream>(quote! {
            println!("\tStage[{}]:", #i);
            println!("\t\tInput:");
            #(println!("\t\t\t{}", #input));*;
            println!("\t\tOutput:");
            #(println!("\t\t\t{}", #output));*;
            println!("\t\tReplicate: {}", #replicate);
        }.into());
    }

    TokenStream::from(codegen)
}

#[cfg(test)]
mod tests {
    //use super::*;
}
