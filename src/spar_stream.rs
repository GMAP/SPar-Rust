//! This module implements the parsing of SparStream

use std::num::NonZeroU32;

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    braced,
    buffer::Cursor,
    parenthesized,
    parse::{Parse, ParseStream, StepCursor},
    punctuated::Punctuated,
    token::Brace,
    Ident, Result, Token,
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

        let mut code = TokenStream::new();
        let stages = skip_until_stage(&block, &mut code)?;

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

fn get_identifiers(cursor: Cursor) -> Result<(Vec<Ident>, Cursor)> {
    let args;
    let after;
    match cursor.group(Delimiter::Parenthesis) {
        Some((a, _, r)) => {
            args = a;
            after = r;
        }
        None => {
            let msg = match cursor.token_tree() {
                Some((tt, _)) => {
                    std::format!("expected arguments, in parenthesis `()`, found: {tt}")
                }

                None => "expected arguments, in parenthesis `()`, found: nothing".to_owned(),
            };
            return Err(syn::Error::new(cursor.span(), msg));
        }
    };
    let mut rest = args;
    let mut idents = Vec::new();
    while let Some((token_tree, next)) = rest.token_tree() {
        match &token_tree {
            TokenTree::Ident(ident) => {
                idents.push(ident.to_owned());
                rest = next;
            }

            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                rest = next;
            }

            _ => {
                let msg = std::format!("unexpected token '{}", token_tree);
                return Err(syn::Error::new(rest.span(), msg));
            }
        }
    }
    Ok((idents, after))
}

fn parse_replicate(cursor: Cursor) -> Result<(NonZeroU32, Cursor)> {
    if let Some((tt, next)) = cursor.token_tree() {
        if let TokenTree::Punct(punct) = tt {
            if punct.as_char() == '=' {
                if let Some((tt, next)) = next.token_tree() {
                    match tt {
                        TokenTree::Ident(_i) => todo!(),
                        TokenTree::Literal(lit) => {
                            if let Ok(i) = lit.to_string().parse::<u32>() {
                                if i > 0 {
                                    return Ok((NonZeroU32::new(i).unwrap(), next));
                                } else {
                                    return Err(syn::Error::new(
                                        cursor.span(),
                                        "'REPLICATE' cannot have an argument of '0'",
                                    ));
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    Err(syn::Error::new(
        cursor.span(),
        "failed to parse REPLICATE attribute.
                        Correct syntax is: `REPLICATE = N`, where is in either a
                        number or an identifier",
    ))
}

fn parse_spar_stage(cursor: Cursor) -> Result<(SparStage, Cursor)> {
    let args;
    let after;
    match cursor.group(Delimiter::Parenthesis) {
        Some((a, _, r)) => {
            args = a;
            after = r;
        }
        None => {
            let msg = match cursor.token_tree() {
                Some((tt, _)) => {
                    std::format!("expected SPAR arguments, in parenthesis `()`, found: {tt}")
                }

                None => "expected SPAR arguments, in parenthesis `()`, found: nothing".to_owned(),
            };
            return Err(syn::Error::new(cursor.span(), msg));
        }
    };

    let mut input: Vec<Ident> = Vec::new();
    let mut output: Vec<Ident> = Vec::new();
    let mut replicate: Option<NonZeroU32> = None;
    let mut code = TokenStream::new();

    let mut rest = args;
    while let Some((token_tree, next)) = rest.token_tree() {
        match &token_tree {
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                "INPUT" => {
                    let (i, next) = get_identifiers(next)?;
                    input = i;
                    rest = next;
                }
                "OUTPUT" => {
                    let (o, next) = get_identifiers(next)?;
                    output = o;
                    rest = next;
                }
                "REPLICATE" => {
                    let (r, next) = parse_replicate(next)?;
                    replicate = Some(r);
                    rest = next;
                }

                _ => {
                    let msg = std::format!( "unexpected token '{}'. Valid tokens are 'INPUT(args)', 'OUTPUT(args)', 'REPLICATE = N' and a code block", token_tree);
                    return Err(syn::Error::new(rest.span(), msg));
                }
            },

            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                rest = next;
            }

            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                let (group, _, next) = rest.group(group.delimiter()).unwrap();
                if !code.is_empty() {
                    return Err(syn::Error::new(
                        rest.span(),
                        "SPAR arguments cannot contain multiple code blocks",
                    ));
                }
                code.extend(group.token_stream());
                rest = next;
            }

            _ => {
                let msg = std::format!( "unexpected token '{}'. Valid tokens are 'INPUT(args)', 'OUTPUT(args)', 'REPLICATE = N' and a code block", token_tree);
                return Err(syn::Error::new(rest.span(), msg));
            }
        }
    }

    let attrs = SparAttrs::new(input, output, replicate);

    Ok((SparStage::new(attrs, code), after))
}

fn cursor_advance_until_stage<'a, 'b>(
    cursor: StepCursor<'a, 'b>,
    tokens: &mut TokenStream,
    stages: &mut Vec<SparStage>,
) -> Result<((), Cursor<'a>)> {
    let mut rest = *cursor;
    let mut after_groups = vec![*cursor];

    while !after_groups.is_empty() {
        rest = after_groups.pop().unwrap();
        while let Some((token_tree, next)) = rest.token_tree() {
            match &token_tree {
                TokenTree::Ident(ident) if ident.to_string() == "STAGE" => {
                    let (stage, next) = parse_spar_stage(next)?;
                    stages.push(stage);
                    rest = next;
                }

                TokenTree::Group(group) => {
                    let (group_cursor, _, next) = rest.group(group.delimiter()).unwrap();
                    after_groups.push(next);
                    rest = group_cursor;
                }

                _ => {
                    token_tree.to_tokens(tokens);
                    rest = next;
                }
            }
        }
    }

    Ok(((), rest))
}

/// Skips the ParseStream up until the next STAGE token, putting everything it skiped inside @tokens
fn skip_until_stage(stream: ParseStream, tokens: &mut TokenStream) -> Result<Vec<SparStage>> {
    let mut stages = Vec::new();
    stream.step(|cursor| Ok(cursor_advance_until_stage(cursor, tokens, &mut stages)?))?;
    Ok(stages)
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
