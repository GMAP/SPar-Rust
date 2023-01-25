//! This module implements the parsing of SparStream

use std::num::NonZeroU32;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Brace,
    Block, Ident, Result, Token,
};

mod kw {
    syn::custom_keyword!(STAGE);
    syn::custom_keyword!(INPUT);
    syn::custom_keyword!(OUTPUT);
    syn::custom_keyword!(REPLICATE);
}

pub struct SparAttrs {
    pub input: Vec<Ident>,
    pub output: Vec<Ident>,
    pub replicate: Option<NonZeroU32>,
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

pub struct SparStage {
    pub attrs: SparAttrs,
    pub code: TokenStream,
}

impl SparStage {
    pub fn new(attrs: SparAttrs, code: TokenStream) -> Self {
        Self { attrs, code }
    }
}

pub struct SparStream {
    pub attrs: SparAttrs,
    pub stages: Vec<SparStage>,
    pub code: TokenStream,
}

impl Parse for SparStream {
    fn parse(input: ParseStream) -> Result<Self> {
        let (spar_input, spar_output, replicate) = parse_spar_args(&input)?;

        let attrs = SparAttrs::new(
            spar_input.into_iter().collect(),
            spar_output.into_iter().collect(),
            replicate,
        );

        let block;
        braced!(block in input);

        let mut stages = Vec::new();
        let mut code = TokenStream::new();
        while !block.is_empty() {
            skip_until_stage(&block, &mut code)?;

            let stage_args;
            parenthesized!(stage_args in block);

            let (stage_input, stage_output, stage_replicate) = parse_spar_args(&stage_args)?;
            let stage_attrs = SparAttrs::new(
                stage_input.into_iter().collect(),
                stage_output.into_iter().collect(),
                stage_replicate,
            );
            let b = stage_args.parse::<Block>()?; // This is necessary to empty the parser

            stages.push(SparStage::new(stage_attrs, b.into_token_stream()));
            block.parse::<Token![;]>()?;
        }

        if !input.is_empty() {
            return Err(input.error("unexpected trailing tokens"));
        }

        Ok(Self {
            attrs,
            code,
            stages,
        })
    }
}

/// Skips the ParseStream up until the next STAGE token, putting everything it skiped inside @tokens
fn skip_until_stage(stream: ParseStream, tokens: &mut TokenStream) -> Result<()> {
    stream.step(|cursor| {
        let mut rest = *cursor;
        while let Some((token_tree, next)) = rest.token_tree() {
            match &token_tree {
                TokenTree::Ident(ident) if ident.to_string() == "STAGE" => {
                    return Ok(((), next));
                }
                _ => {
                    token_tree.to_tokens(tokens);
                    rest = next;
                }
            }
        }
        Ok(((), rest))
    })?;
    Ok(())
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
