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

    let input = &attrs.input;
    let output = &attrs.output;
    let replicate = gen_replicate(&attrs.replicate);

    let mut codegen: TokenStream = quote! {
        println!("SparStream:");
        println!("\tInput:");
        #(println!("\t\t{}", #input));*;
        println!("\tOutput:");
        #(println!("\t\t{}", #output));*;
        println!("\tReplicate: {}", #replicate);
    }
    .into();

    let code = TokenBuffer::new2(code);
    let mut location = 0;

    let cursor = code.begin();
    let mut rest;
    let mut groups_code = Vec::new();
    let mut after_groups = vec![cursor];
    while !after_groups.is_empty() {
        rest = after_groups.pop().unwrap();
        while let Some((token_tree, next)) = rest.token_tree() {
            dbg!(location);
            dbg!(&token_tree);
            match &token_tree {
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                    let (group_cursor, _, next) = rest.group(group.delimiter()).unwrap();
                    groups_code.push(TokenStream::new());
                    rest = group_cursor;
                    after_groups.push(next);
                }

                _ => {
                    token_tree.to_tokens(groups_code.last_mut().unwrap_or(&mut codegen));
                    rest = next;
                }
            }

            while let Some(stage) = stages.first() {
                if location + 1 == stage.location {
                    location += 1;
                    let stage = stages.remove(0);
                    let input = &stage.attrs.input;
                    let output = &stage.attrs.output;
                    let replicate = gen_replicate(&stage.attrs.replicate);
                    let location = &stage.location;

                    let c = groups_code.last_mut().unwrap_or(&mut codegen);
                    c.extend(stage.code.clone().into_iter());
                    c.extend::<TokenStream>(
                        quote! {
                            println!("\tStage[{}]:", #location);
                            println!("\t\tInput:");
                            #(println!("\t\t\t{}", #input));*;
                            println!("\t\tOutput:");
                            #(println!("\t\t\t{}", #output));*;
                            println!("\t\tReplicate: {}", #replicate);
                        }
                        .into(),
                    );
                } else {
                    break;
                }
            }

            location += 1;
        }
        if let Some(code) = groups_code.pop() {
            codegen
                .extend(TokenTree::Group(Group::new(Delimiter::Brace, code)).into_token_stream());
        }
    }

    codegen
}
