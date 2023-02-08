use std::num::NonZeroU32;

use crate::{
    backend::{self, CrossbeamMessenger},
    spar_stream::{SparAttrs, SparStream},
};
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::buffer::{Cursor, TokenBuffer};

///Note: replicate defaults to 1 when it is not given.
///If REPLICATE argument exists, then it defaults to what was written in the code
///if SPAR_NUM_WORKERS is set, all REPLICATES are set to that value
fn gen_replicate(replicate: &Option<NonZeroU32>) -> TokenStream {
    match replicate {
        Some(n) => {
            let n: u32 = (*n).into();
            quote! {
                if let Some(workers) = spar_num_workers {
                    workers
                } else {
                    #n
                }
            }
        }
        None => quote!(1),
    }
}

fn spar_code_top_level(attrs: &SparAttrs) -> TokenStream {
    let input = &attrs.input;
    quote! {
        // Set spar_num_workers according to the envvar SPAR_NUM_WORKERS
        // If it doesn't exist, OR it is invalid, we simply set it to NONE
        let spar_num_workers: Option<u32> = match std::env::var("SPAR_NUM_WORKERS") {
            Ok(var) => match var.parse() {
                Ok(value) => if value < 1 {
                    eprintln!("SPAR_NUM_WORKERS must be a number > 0. Found {}. Defaulting to 1...", value);
                    Some(1)
                } else {
                    Some(value)
                },
                Err(_) => {
                    eprintln!("invalid value for SPAR_NUM_WORKERS variable: {}. Ignoring...", var);
                    None
                }
            }
            Err(_) => None
        };

        // variable to tag packages
        let mut spar_tag: u64 = 0;

        // Set inputs to their respective names
        // This assures we will move any necessary variable into the spar_stream, if it is necessary
        #(let mut #input = #input;)*
    }
}

fn gen_stage<E, C, M>(attrs: &SparAttrs, messenger: &mut M, code: TokenStream) -> TokenStream
where
    E: backend::Emitter,
    C: backend::Collector,
    M: backend::Messenger<E, C>,
{
    let mut pre_worker_code = TokenStream::new();
    let mut worker_code = TokenStream::new();
    let mut post_worker_code = TokenStream::new();

    let mut sender_clone = TokenStream::new();
    let mut receiver_clone = TokenStream::new();

    let replicate = gen_replicate(&attrs.replicate);
    worker_code.extend(quote!(println!("replicate: {:?}", #replicate);));

    if !attrs.input.is_empty() {
        pre_worker_code.extend(messenger.prepare());
        let (mut emitter, mut collector) = messenger.channel(&attrs.input);
        pre_worker_code.extend(emitter.emit());

        receiver_clone = collector.gen_clone();
        worker_code.extend(collector.collect());

        post_worker_code.extend(messenger.finish());
    }

    worker_code.extend(code);

    if !attrs.output.is_empty() {
        pre_worker_code.extend(messenger.prepare());
        let (mut emitter, mut collector) = messenger.channel(&attrs.output);

        sender_clone = emitter.gen_clone();
        worker_code.extend(emitter.emit());
        post_worker_code.extend(collector.collect());

        post_worker_code.extend(messenger.finish());
    }

    quote! {
        #pre_worker_code
        for _ in 0..#replicate {
            #receiver_clone
            #sender_clone
            std::thread::Builder::new()
                .name("SPar worker".to_string())
                .spawn_scoped(spar_scope, move || {
                    #worker_code
                })
                .expect("Failed to spawn SPar worker");
        }
        #post_worker_code
    }
}

fn skip_attributes(cursor: Cursor) -> Cursor {
    let mut rest = cursor;
    while let Some((tt, next)) = rest.token_tree() {
        if let TokenTree::Group(group) = tt {
            if group.delimiter() == Delimiter::Brace {
                let (group_cursor, _, _) = rest.group(group.delimiter()).unwrap();
                rest = group_cursor;
                break;
            }
        }
        rest = next;
    }
    rest
}

pub fn codegen(spar_stream: SparStream, code: proc_macro::TokenStream) -> TokenStream {
    let SparStream { attrs, mut stages } = spar_stream;

    let mut messenger = CrossbeamMessenger::new();
    let code = TokenBuffer::new(code);
    let cursor = skip_attributes(code.begin());
    let mut code_stack = vec![spar_code_top_level(&attrs)];
    let mut after_groups = vec![cursor];

    while !after_groups.is_empty() {
        let mut rest = after_groups.pop().unwrap();
        while let Some((token_tree, next)) = rest.token_tree() {
            match &token_tree {
                TokenTree::Ident(ident) if *ident == "STAGE" => {
                    let (in_group, _, after_group) = next.group(Delimiter::Parenthesis).unwrap();

                    let stage = stages.remove(0);
                    let stage_code = gen_stage(
                        &stage.attrs,
                        &mut messenger,
                        skip_attributes(in_group).token_stream(),
                    );
                    code_stack.last_mut().unwrap().extend(stage_code);
                    // Make sure to skip the ';'
                    rest = after_group.token_tree().unwrap().1;
                }
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
        }
        if code_stack.len() > 1 {
            let code = code_stack.pop().unwrap();
            code_stack
                .last_mut()
                .unwrap()
                .extend(TokenTree::Group(Group::new(Delimiter::Brace, code)).into_token_stream());
        }
    }

    //Make the stream return a tuple with its 'OUTPUT'
    let mut code = code_stack.pop().unwrap();
    code.extend(crate::backend::make_tuple(&attrs.output));
    quote! {
        std::thread::scope(|spar_scope| {
            #code
        })
    }
}

//TODO: test the code generation, once we figure it out
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn should_() {
//
//    }
//}
