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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

    Ok((
        SparAttrs::new(input, output, replicate),
        after,
        block_cursor,
    ))
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

#[cfg(test)]
mod tests {
    use proc_macro2::{Span, TokenStream};
    use quote::quote;

    use super::*;

    /// Returns the span of the first instance of an identifier with the same name
    fn get_ident_span(identifier: &'static str, tokens: TokenStream) -> Option<Span> {
        let buffer = TokenBuffer::new2(tokens);
        let cursor = buffer.begin();

        let mut groups = vec![cursor];
        while !groups.is_empty() {
            let mut rest = groups.pop().unwrap();
            while let Some((token_tree, next)) = rest.token_tree() {
                match &token_tree {
                    TokenTree::Ident(ident) if *ident == identifier => {
                        return Some(ident.span());
                    }

                    TokenTree::Group(group) => {
                        let (group_cursor, _, next) = rest.group(group.delimiter()).unwrap();
                        groups.push(next);
                        rest = group_cursor;
                    }

                    _ => rest = next,
                }
            }
        }

        None
    }

    fn get_spans(idents: &[&'static str], tokens: &TokenStream) -> Vec<Ident> {
        let mut vec = Vec::new();

        for i in idents {
            let span =
                get_ident_span(i, tokens.clone()).expect("Failed to find identifier in stream");
            vec.push(Ident::new(i, span));
        }

        vec
    }

    #[test]
    fn stage_no_attributes() {
        let tokens = quote! {
            STAGE({
                // Put some dummy code just to make sure nothing will break
                let mut a = 10;
                while true {
                    a += 1;
                }
            });
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens).begin()).unwrap();
        assert_eq!(spar_stages.len(), 1);

        let expected_attrs = SparAttrs::new(Vec::new(), Vec::new(), None);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }

    #[test]
    fn stage_with_input() {
        let tokens = quote! {
            STAGE(INPUT(a), {
                while true {
                    a += 1;
                }
            });
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens.clone()).begin()).unwrap();
        assert_eq!(spar_stages.len(), 1);

        let input = get_spans(&["a"], &tokens);
        let output = vec![];
        let replicate = None;
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }

    #[test]
    fn stage_with_multiple_inputs() {
        let tokens = quote! {
            STAGE(INPUT(a, b, c), {
                while true {
                    a += 1;
                    b += 2,
                    c += 3;
                }
            });
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens.clone()).begin()).unwrap();
        assert_eq!(spar_stages.len(), 1);

        let input = get_spans(&["a", "b", "c"], &tokens);
        let output = vec![];
        let replicate = None;
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }

    #[test]
    fn stage_with_output() {
        let tokens = quote! {
            STAGE(OUTPUT(a), {
                while true {
                    a += 1;
                }
            });
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens.clone()).begin()).unwrap();
        assert_eq!(spar_stages.len(), 1);

        let input = vec![];
        let output = get_spans(&["a"], &tokens);
        let replicate = None;
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }

    #[test]
    fn stage_with_multiple_outputs() {
        let tokens = quote! {
            STAGE(OUTPUT(a, b, c), {
                while true {
                    a += 1;
                    b += 2,
                    c += 3;
                }
            });
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens.clone()).begin()).unwrap();
        assert_eq!(spar_stages.len(), 1);

        let input = vec![];
        let output = get_spans(&["a", "b", "c"], &tokens);
        let replicate = None;
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }

    #[test]
    fn stage_with_replicate() {
        let tokens = quote! {
            STAGE(REPLICATE = 5, {
                // Put some dummy code just to make sure nothing will break
                let mut a = 10;
                while true {
                    a += 1;
                }
            });
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens).begin()).unwrap();
        assert_eq!(spar_stages.len(), 1);

        let expected_attrs = SparAttrs::new(Vec::new(), Vec::new(), NonZeroU32::new(5));
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }

    #[test]
    fn multiple_stages() {
        let tokens = quote! {
            STAGE({});
            STAGE(INPUT(a), OUTPUT(b), {});
            STAGE(INPUT(c, d), OUTPUT(e, f, g), {});
            STAGE(INPUT(h), OUTPUT(i), REPLICATE = 5, {});
        };

        let mut spar_stages = parse_spar_stages(TokenBuffer::new2(tokens.clone()).begin()).unwrap();
        assert_eq!(spar_stages.len(), 4);
        spar_stages.reverse();

        let expected_attrs = SparAttrs::new(Vec::new(), Vec::new(), None);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));

        let input = get_spans(&["a"], &tokens);
        let output = get_spans(&["b"], &tokens);
        let replicate = None;
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));

        let input = get_spans(&["c", "d"], &tokens);
        let output = get_spans(&["e", "f", "g"], &tokens);
        let replicate = None;
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));

        let input = get_spans(&["h"], &tokens);
        let output = get_spans(&["i"], &tokens);
        let replicate = NonZeroU32::new(5);
        let expected_attrs = SparAttrs::new(input, output, replicate);
        assert_eq!(spar_stages.pop().unwrap(), SparStage::new(expected_attrs));
    }
}
