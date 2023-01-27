use std::num::NonZeroU32;

use crate::spar_stream::SparStream;
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::buffer::TokenBuffer;

fn gen_replicate(replicate: &Option<NonZeroU32>) -> TokenStream {
    match replicate {
        Some(n) => {
            let n: u32 = (*n).into();
            quote!(#n)
        }
        None => quote!(std::env::var("SPAR_NUM_WORKERS")
            .unwrap_or("1".to_string())
            .parse()
            .unwrap_or(1)),
    }
}

pub fn codegen(spar_stream: SparStream) -> TokenStream {
    let SparStream {
        attrs,
        code,
        mut stages,
    } = spar_stream;

    let _input = &attrs.input;
    let _output = &attrs.output;
    let _replicate = gen_replicate(&attrs.replicate);

    let code = TokenBuffer::new2(code);
    let mut location = 0;

    let cursor = code.begin();
    let mut rest;
    let mut code_stack = vec![TokenStream::new()];
    let mut after_groups = vec![cursor];
    while !after_groups.is_empty() {
        rest = after_groups.pop().unwrap();
        while let Some((token_tree, next)) = rest.token_tree() {
            match &token_tree {
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                    let (group_cursor, _, next) = rest.group(group.delimiter()).unwrap();
                    code_stack.push(TokenStream::new());
                    rest = group_cursor;
                    after_groups.push(next);
                }

                _ => {
                    token_tree.to_tokens(code_stack.last_mut().unwrap());
                    rest = next;
                }
            }

            while let Some(stage) = stages.first() {
                if location + 1 == stage.location {
                    location += 1;
                    let stage = stages.remove(0);
                    let _input = &stage.attrs.input;
                    let _output = &stage.attrs.output;
                    let _replicate = gen_replicate(&stage.attrs.replicate);
                    let _location = &stage.location;

                    let c = code_stack.last_mut().unwrap();
                    c.extend(stage.code.clone().into_iter());
                } else {
                    break;
                }
            }

            location += 1;
        }
        if code_stack.len() > 1 {
            let code = code_stack.pop().unwrap();
            code_stack
                .last_mut()
                .unwrap()
                .extend(TokenTree::Group(Group::new(Delimiter::Brace, code)).into_token_stream());
        }
    }

    let codegen = code_stack.pop().unwrap();
    codegen
}
