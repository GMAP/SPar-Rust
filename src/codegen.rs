use std::num::NonZeroU32;

use crate::spar_stream::{SparStage, SparStream, SparVar};
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};

struct Collector {
    struct_name: TokenStream,
    inputs: Vec<SparVar>,
    outputs: Vec<SparVar>,
    code: TokenStream,
}

impl Collector {
    fn new(struct_name: TokenStream, stage: &SparStage) -> Self {
        Self {
            struct_name,
            inputs: stage.attrs.input.clone(),
            outputs: stage.attrs.output.clone(),
            code: stage.code.clone(),
        }
    }

    fn gen(self) -> TokenStream {
        let Self {
            struct_name,
            code,
            inputs,
            outputs,
        } = self;
        let (in_idents, in_types) = get_idents_and_types_from_spar_vars(&inputs);
        let (_, out_types) = get_idents_and_types_from_spar_vars(&outputs);
        let in_types = make_tuple(&in_types);
        let out_types = make_tuple(&out_types);

        let input_tuple = make_tuple(&in_idents);

        quote! {
            struct #struct_name {
                output: #out_types
            }

            impl #struct_name {
                fn new(input: #in_types) -> Self {
                    let #input_tuple = input;
                    let output = {
                        #code
                    };
                    Self { output }
                }

                fn process(&mut self, input: #in_types) {
                    let #input_tuple = input;
                    let _ = {
                        #code
                    };
                }
            }

            let spar_output = spar_pipeline.collect();
            let mut spar_collector = #struct_name::new();
            for (i, output) in spar_output.iter().enumerate() {
                spar_collector.process(output, i as u64);
            }
            spar_collector.output
        }
    }
}

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

fn get_idents_and_types_from_spar_vars(vars: &[SparVar]) -> (Vec<Ident>, Vec<TokenStream>) {
    let mut idents = Vec::new();
    let mut types = Vec::new();

    for var in vars {
        idents.push(var.identifier.clone());
        types.push(var.var_type.clone());
    }

    (idents, types)
}

fn rust_spp_stage_struct_gen(stage: &SparStage, stage_number: usize) -> TokenStream {
    let (in_idents, in_types) = get_idents_and_types_from_spar_vars(&stage.attrs.input);
    let (out_idents, out_types) = get_idents_and_types_from_spar_vars(&stage.attrs.output);

    let struct_name = format!("SparStage{stage_number}");
    let struct_ident = Ident::new(&struct_name, Span::call_site());
    let stage_code = &stage.code;

    let mut code = quote! {
        struct #struct_ident {
            // TODO: we need to declare variables for stateful computation
        }

        impl #struct_ident {
            fn new() -> Self {
                Self {}
            }
        }
    };

    if !in_types.is_empty() && !out_types.is_empty() {
        let in_types = make_tuple(&in_types);
        let out_types = make_tuple(&out_types);

        let input_tuple = make_tuple(&in_idents);
        let output_tuple = make_tuple(&out_idents);

        code.extend(quote! {
            impl rust_spp::blocks::inout_block::InOut<#in_types, #out_types> for #struct_ident {
                fn process(&mut self, input: #in_types) -> Option<#out_types> {
                    let #input_tuple = input;
                    #stage_code
                    Some(#output_tuple)
                }
            }
        });
    } else if !in_types.is_empty() {
        let in_types = make_tuple(&in_types);
        let input_tuple = make_tuple(&in_idents);
        code.extend(quote! {
            impl rust_spp::blocks::in_block::In<#in_types> for #struct_ident {
                fn process(&mut self, input: #in_types, order: u64) -> () {
                    let #input_tuple = input;
                    #stage_code
                }
            }
        });
    } else {
        //ERROR
        panic!("ERROR: Stage without input is invalid!");
    }

    code
}

fn rust_spp_gen_top_level_code(spar_stream: &SparStream) -> (TokenStream, Option<Collector>) {
    let SparStream { stages, attrs } = spar_stream;
    let mut code = quote! {
        use rust_spp::*;
    };

    for (i, stage) in stages[0..stages.len() - 1].iter().enumerate() {
        code.extend(rust_spp_stage_struct_gen(stage, i));
    }

    let last_stage = stages.last().unwrap();
    if last_stage.attrs.output == attrs.output {
        if let Some(n) = last_stage.attrs.replicate {
            if n > std::num::NonZeroU32::new(1).unwrap() {
                let ident = Ident::new("Collector", Span::call_site());
                let collector = Collector::new(ident.into_token_stream(), last_stage);
                return (code, Some(collector));
            }
        } else {
            let ident = Ident::new("Collector", Span::call_site());
            let collector = Collector::new(ident.into_token_stream(), last_stage);
            return (code, Some(collector));
        }
    }

    code.extend(rust_spp_stage_struct_gen(last_stage, stages.len() - 1));
    (code, None)
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
        let spar_output = spar_pipeline.collect();
    }
}

fn rust_spp_gen(spar_stream: &SparStream) -> TokenStream {
    let (top_level, collector) = rust_spp_gen_top_level_code(spar_stream);
    let SparStream { stages, attrs } = spar_stream;
    let mut spar_pipeline = rust_spp_gen_pipeline_post(&attrs.input);
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

    let mut code  = quote! {
        #top_level
        let spar_pipeline = rust_spp::pipeline![
            #gen
        ];
        #spar_pipeline
    };

    if let Some(collector) = collector {
        code.extend(collector.gen());
    }

    code
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
