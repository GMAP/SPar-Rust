use std::num::NonZeroU32;

use crate::spar_stream::{SparStage, SparStream, SparVar};
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};

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

fn gen_spar_num_workers() -> TokenStream {
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
    }
}

fn make_tuple<T: ToTokens>(tokens: &[T]) -> TokenStream {
    quote! { ( #(#tokens),* ) }
}

fn rust_spp_gen_top_level_code(stages: &[SparStage]) -> TokenStream {
    let mut code = quote! {
        use rust_spp::*;
    };

    for (i, stage) in stages.iter().enumerate() {
        let mut input_types = Vec::new();
        let mut input_idents = Vec::new();

        for input in &stage.attrs.input {
            input_idents.push(input.identifier.clone());
            input_types.push(input.var_type.clone());
        }

        let mut output_types = Vec::new();
        let mut output_idents = Vec::new();

        for output in &stage.attrs.output {
            output_idents.push(output.identifier.clone());
            output_types.push(output.var_type.clone());
        }

        let struct_name = format!("SparStage{i}");
        let struct_ident = Ident::new(&struct_name, Span::call_site());
        let stage_code = &stage.code;

        code.extend(quote! {
            struct #struct_ident {
                // TODO: we need to declare variables for stateful
                // computation
            }

            impl #struct_ident {
                fn new() -> Self {
                    Self {}
                }
            }
        });

        if !input_types.is_empty() && !output_types.is_empty() {
            let input_types = make_tuple(&input_types);
            let output_types = make_tuple(&output_types);

            let input_tuple = make_tuple(&input_idents);
            let output_tuple = make_tuple(&output_idents);

            code.extend(quote! {
                impl rust_spp::blocks::inout_block::InOut<#input_types, #output_types>
                for #struct_ident {
                    fn process(
                        &mut self,
                        input: #input_types
                    ) -> Option<#output_types> {
                        let #input_tuple = input;
                        #stage_code
                        Some(#output_tuple)
                    }
                }
            });
        } else if !input_types.is_empty() {
            let input_types = make_tuple(&input_types);
            let input_tuple = make_tuple(&input_idents);
            code.extend(quote! {
                impl rust_spp::blocks::in_block::In<#input_types> for #struct_ident {
                    fn process(&mut self, input: #input_types, order: u64) -> () {
                        let #input_tuple = input;
                        #stage_code
                    }
                }
            });
        } else {
            //ERROR
            panic!("ERROR: Stage without input is invalid!");
        }
    }

    code
}

fn rust_spp_gen_first_stage(stage: &SparStage) -> TokenStream {
    let mut gen = TokenStream::new();
    let code = stage.code.clone();
    let mut found = false;
    for token in code.into_iter() {
        if let TokenTree::Ident(ref ident) = token {
            if ident == "__SPAR_MARKER__" {
                gen.extend(rust_spp_gen_pipeline_post(&stage.attrs.input));
                found = true;
                continue;
            }
        }
        gen.extend(token.to_token_stream());
    }

    if found {
        gen
    } else {
        TokenStream::new()
    }
}

fn rust_spp_gen_pipeline_post(inputs: &[SparVar]) -> TokenStream {
    let mut idents = Vec::new();

    for input in inputs {
        idents.push(input.identifier.clone())
    }

    let inputs = make_tuple(&idents);
    quote! {
        spar_pipeline.post(#inputs);
    }
}

fn rust_spp_gen(spar_stream: &SparStream) -> TokenStream {
    let SparStream { stages, attrs } = spar_stream;
    let mut spar_pipeline = rust_spp_gen_pipeline_post(&attrs.input);
    let top_level = rust_spp_gen_top_level_code(stages);
    let mut gen = TokenStream::new();

    for (i, stage) in stages.iter().enumerate() {
        if i == 0 {
            let code = rust_spp_gen_first_stage(stage);
            if !code.is_empty() {
                spar_pipeline = code;
                continue;
            }
        }
        let SparStage { attrs, .. } = stage;
        let struct_name = format!("SparStage{i}");
        let struct_ident = Ident::new(&struct_name, Span::call_site());

        if !gen.is_empty() {
            gen.extend(quote!(,));
        }

        if attrs.replicate.is_none() {
            gen.extend(quote! { rust_spp::sequential!(#struct_ident::new()) });
        } else {
            let replicate = gen_replicate(&attrs.replicate);
            gen.extend(quote! { rust_spp::parallel!(#struct_ident::new(), #replicate) })
        }
    }

    quote! {
        #top_level
        let spar_pipeline = rust_spp::pipeline![
            #gen
        ];
        #spar_pipeline
    }
}

pub fn codegen(spar_stream: SparStream) -> TokenStream {
    let mut code = gen_spar_num_workers();
    dbg!(spar_stream.stages.len());
    code.extend(rust_spp_gen(&spar_stream));

    //TODO: stream analysis and code generation

    Group::new(Delimiter::Brace, code).into_token_stream()
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
