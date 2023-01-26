//! This module implements the parsing of SparStream

use std::num::NonZeroU32;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    buffer::{Cursor, TokenBuffer},
    Ident, Result,
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

impl TryFrom<proc_macro::TokenStream> for SparStream {
    type Error = syn::Error;

    fn try_from(value: proc_macro::TokenStream) -> std::result::Result<Self, Self::Error> {
        let input = TokenBuffer::new(
            TokenTree::Group(Group::new(Delimiter::Parenthesis, value.into()))
                .into_token_stream()
                .into(),
        );
        let (attrs, block, _) = parse_spar_args(input.begin())?;
        let mut code = TokenStream::new();
        let mut stages = Vec::new();
        parse_spar_stages(TokenBuffer::new2(block).begin(), &mut code, &mut stages)?;

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

fn parse_spar_args(cursor: Cursor) -> Result<(SparAttrs, TokenStream, Cursor)> {
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

    Ok((SparAttrs::new(input, output, replicate), code, after))
}

fn parse_spar_stages<'a>(
    cursor: Cursor<'a>,
    tokens: &mut TokenStream,
    stages: &mut Vec<SparStage>,
) -> Result<((), Cursor<'a>)> {
    let mut rest = cursor;
    let mut after_groups = vec![cursor];

    while !after_groups.is_empty() {
        rest = after_groups.pop().unwrap();
        while let Some((token_tree, next)) = rest.token_tree() {
            match &token_tree {
                TokenTree::Ident(ident) if ident.to_string() == "STAGE" => {
                    let (attrs, code, next) = parse_spar_args(next)?;
                    stages.push(SparStage::new(attrs, code));
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
