//! This module implements the parsing of SparStream

use std::num::NonZeroU32;

use proc_macro2::{Delimiter, Group, TokenTree};
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
}

impl SparStage {
    pub fn new(attrs: SparAttrs) -> Self {
        Self { attrs }
    }
}

pub struct SparStream {
    pub attrs: SparAttrs,
    pub stages: Vec<SparStage>,
}

impl TryFrom<&proc_macro::TokenStream> for SparStream {
    type Error = syn::Error;

    fn try_from(value: &proc_macro::TokenStream) -> std::result::Result<Self, Self::Error> {
        let input = TokenBuffer::new(
            TokenTree::Group(Group::new(Delimiter::Parenthesis, value.clone().into()))
                .into_token_stream()
                .into(),
        );
        let (attrs, _, block) = parse_spar_args(input.begin())?;
        let stages = parse_spar_stages(block)?;

        Ok(Self { attrs, stages })
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
                let msg = std::format!("unexpected token '{token_tree}'");
                return Err(syn::Error::new(rest.span(), msg));
            }
        }
    }
    Ok((idents, after))
}

fn parse_replicate(cursor: Cursor) -> Result<(NonZeroU32, Cursor)> {
    if let Some((TokenTree::Punct(punct), next)) = cursor.token_tree() {
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

    Err(syn::Error::new(
        cursor.span(),
        "failed to parse REPLICATE attribute.
                        Correct syntax is: `REPLICATE = N`, where is in either a
                        number or an identifier",
    ))
}

fn parse_spar_args(cursor: Cursor) -> Result<(SparAttrs, Cursor, Cursor)> {
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
    let mut block_cursor = cursor;
    let mut has_code = false;

    let mut rest = args;
    while let Some((token_tree, next)) = rest.token_tree() {
        match &token_tree {
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                "INPUT" => {
                    if !input.is_empty() {
                        return Err(syn::Error::new(
                            rest.span(),
                            "multiple INPUTs aren't allowed",
                        ));
                    }
                    let (i, next) = get_identifiers(next)?;
                    input = i;
                    rest = next;
                }
                "OUTPUT" => {
                    if !output.is_empty() {
                        return Err(syn::Error::new(
                            rest.span(),
                            "multiple OUTPUTs aren't allowed",
                        ));
                    }
                    let (o, next) = get_identifiers(next)?;
                    output = o;
                    rest = next;
                }
                "REPLICATE" => {
                    if replicate.is_some() {
                        return Err(syn::Error::new(
                            rest.span(),
                            "multiple REPLICATEs aren't allowed",
                        ));
                    }
                    let (r, next) = parse_replicate(next)?;
                    replicate = Some(r);
                    rest = next;
                }

                _ => {
                    let msg = std::format!( "unexpected token '{token_tree}'. Valid tokens are 'INPUT(args)', 'OUTPUT(args)', 'REPLICATE = N' and a code block");
                    return Err(syn::Error::new(rest.span(), msg));
                }
            },

            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                rest = next;
            }

            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                let (group_cursor, _, next) = rest.group(group.delimiter()).unwrap();
                if has_code {
                    return Err(syn::Error::new(
                        rest.span(),
                        "SPAR arguments cannot contain multiple code blocks",
                    ));
                }
                block_cursor = group_cursor;
                has_code = true;
                rest = next;
            }

            _ => {
                let msg = std::format!( "unexpected token '{token_tree}'. Valid tokens are 'INPUT(args)', 'OUTPUT(args)', 'REPLICATE = N' and a code block");
                return Err(syn::Error::new(rest.span(), msg));
            }
        }
    }

    if !has_code {
        return Err(syn::Error::new(
            rest.span(),
            "there must be block of code `{...}` in SPAR's arguments",
        ));
    }

    Ok((SparAttrs::new(input, output, replicate), after, block_cursor))
}

fn parse_spar_stages(cursor: Cursor) -> Result<Vec<SparStage>> {
    let mut stages = Vec::new();

    let mut groups = vec![cursor];
    while !groups.is_empty() {
        let mut rest = groups.pop().unwrap();
        while let Some((token_tree, next)) = rest.token_tree() {
            match &token_tree {
                TokenTree::Ident(ident) if *ident == "STAGE" => {
                    let (attrs, semicolon, _) = parse_spar_args(next)?;
                    stages.push(SparStage::new(attrs));

                    match semicolon.token_tree() {
                        Some((token, next)) => match token {
                            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                                rest = next;
                            }
                            _ => return Err(syn::Error::new(next.span(), "expected ';'")),
                        },
                        None => return Err(syn::Error::new(next.span(), "expected ';'")),
                    }
                }

                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                    let (group_cursor, _, next) = rest.group(group.delimiter()).unwrap();
                    groups.push(next);
                    rest = group_cursor;
                }

                _ => rest = next,
            }
        }
    }

    Ok(stages)
}
