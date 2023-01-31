use std::num::NonZeroU32;

use crate::spar_stream::{SparAttrs, SparStream};
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
    let mut code = TokenStream::new();

    // Set spar_num_workers according to the envvar SPAR_NUM_WORKERS
    // If it doesn't exist, OR it is invalid, we simply set it to NONE
    code.extend(quote! {
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
    });

    // Set inputs to their respective names
    // This assures we will move any necessary variable into the spar_stream, if it is necessary
    for identifier in &attrs.input {
        code.extend(quote! {
            let #identifier = #identifier;
        })
    }

    // Create output variables
    //for identifier in &attrs.output {
    //    code.extend(quote! {
    //        let #identifier;
    //    })
    //}

    code
}

fn gen_variables(attrs: &SparAttrs) -> TokenStream {
    let mut code = TokenStream::new();
    let replicate = gen_replicate(&attrs.replicate);

    // Create input variables
    for identifier in &attrs.input {
        code.extend(quote! {
            let #identifier = #identifier;
        })
    }

    // Create output variables
    for identifier in &attrs.output {
        code.extend(quote! {
            let #identifier;
        })
    }

    code.extend(quote!(println!("replicate: {:?}", #replicate);));
    code
}

/// generates the necessary channels for communition between the SPar Stages
fn generate_channels(_spar_stream: &SparStream) -> TokenStream {
    //TODO: we must analyze the stream to find the pairs output / input, then we generate all the
    //necessary channels at the top level. WE MUST ALSO FIND A WAY OF STORING THIS INFORMATION
    //(which channels go to which stage)
    todo!()
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
                    code_stack.push(TokenStream::new());
                    after_groups.push(after_group);
                    rest = skip_attributes(in_group);

                    // Generate the stage's code:
                    let stage = stages.remove(0);
                    let c = code_stack.last_mut().unwrap();
                    c.extend(gen_variables(&stage.attrs));
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

    TokenTree::Group(Group::new(Delimiter::Brace, code_stack.pop().unwrap())).into_token_stream()
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
