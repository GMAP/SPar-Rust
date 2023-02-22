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
        let (out_idents, out_types) = get_idents_and_types_from_spar_vars(&outputs);
        let in_types = make_tuple(&in_types);
        let out_types = make_tuple(&out_types);

        let input_tuple = make_tuple(&in_idents);
        let output_tuple = make_tuple(&out_idents);
        let mut_output_tuple = make_mut_tuple(&out_idents);

        quote! {
            struct #struct_name {
                output: #out_types
            }

            impl #struct_name {
                fn new(output: #out_types) -> Self {
                    Self { output }
                }

                fn process(&mut self, input: #in_types, order: u64) {
                    let #input_tuple = input;
                    let #mut_output_tuple = self.output;
                    self.output = {
                        #code
                    }
                }
            }

            let spar_output = spar_pipeline.collect();
            let mut spar_collector = #struct_name::new(#output_tuple);
            for (i, output) in spar_output.into_iter().enumerate() {
                spar_collector.process(output, i as u64);
            }
            spar_collector.output
        }
    }
}

struct Dispatcher {
    code: TokenStream,
}

impl Dispatcher {
    fn copy_code(tokens: TokenTree, found: &mut bool, replacement: &TokenStream) -> TokenStream {
        match tokens {
            TokenTree::Group(group) => Group::new(
                group.delimiter(),
                group
                    .stream()
                    .into_iter()
                    .map(|token| Self::copy_code(token, found, replacement))
                    .collect(),
            )
            .into_token_stream(),
            TokenTree::Ident(ident) => {
                if ident == "__SPAR_MARKER__" {
                    *found = true;
                    replacement.clone()
                } else {
                    ident.into_token_stream()
                }
            }
            TokenTree::Punct(punct) => punct.into_token_stream(),
            TokenTree::Literal(literal) => literal.into_token_stream(),
        }
    }

    pub fn new(stage: &SparStage) -> (Self, bool) {
        let mut idents = Vec::new();

        for input in &stage.attrs.input {
            idents.push(input.identifier.clone())
        }

        let inputs = make_tuple(&idents);
        let pipeline_post = quote! { spar_pipeline.post(#inputs).unwrap(); };

        let mut gen = TokenStream::new();
        let mut found = false;
        for token in stage.code.clone().into_iter() {
            gen.extend(Self::copy_code(token, &mut found, &pipeline_post));
        }

        if found {
            (Self { code: gen }, true)
        } else {
            (
                Self {
                    code: pipeline_post,
                },
                false,
            )
        }
    }
}

impl ToTokens for Dispatcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.code.clone());
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

fn make_mut_tuple<T: ToTokens>(tokens: &[T]) -> TokenStream {
    quote! { ( #(mut #tokens),* ) }
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

fn rust_spp_stage_struct_gen(stage: &SparStage) -> TokenStream {
    let (in_idents, in_types) = get_idents_and_types_from_spar_vars(&stage.attrs.input);
    let (out_idents, out_types) = get_idents_and_types_from_spar_vars(&stage.attrs.output);

    let struct_name = format!("SparStage{}", stage.id);
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

fn rust_spp_gen_top_level_code(
    spar_stream: &mut SparStream,
) -> (Vec<TokenStream>, Dispatcher, Option<Collector>) {
    let SparStream {
        ref mut stages,
        attrs,
    } = spar_stream;
    let mut structs = Vec::new();

    let (dispatcher, found) = Dispatcher::new(&stages[0]);
    if found {
        stages.remove(0);
    }

    for stage in stages[0..stages.len() - 1].iter() {
        structs.push(rust_spp_stage_struct_gen(stage));
    }

    let last_stage = stages.last().unwrap();
    if last_stage.attrs.output == attrs.output {
        if let Some(n) = last_stage.attrs.replicate {
            if n > std::num::NonZeroU32::new(1).unwrap() {
                let ident = Ident::new("Collector", Span::call_site());
                let collector = Collector::new(ident.into_token_stream(), last_stage);
                return (structs, dispatcher, Some(collector));
            }
        } else {
            let ident = Ident::new("Collector", Span::call_site());
            let collector = Collector::new(ident.into_token_stream(), last_stage);
            return (structs, dispatcher, Some(collector));
        }
    }

    structs.push(rust_spp_stage_struct_gen(last_stage));
    (structs, dispatcher, None)
}

fn rust_spp_pipeline_arg(stage: &SparStage) -> TokenStream {
    let SparStage { attrs, id, .. } = stage;
    let struct_name = format!("SparStage{id}");
    let struct_ident = Ident::new(&struct_name, Span::call_site());

    if attrs.replicate.is_none() {
        quote! { rust_spp::sequential!(#struct_ident::new()) }
    } else {
        let replicate = gen_replicate(&attrs.replicate);
        quote! { rust_spp::parallel!(#struct_ident::new(), #replicate) }
    }
}

fn rust_spp_gen(spar_stream: &mut SparStream) -> TokenStream {
    let (spar_structs, dispatcher, collector) = rust_spp_gen_top_level_code(spar_stream);
    let mut gen = TokenStream::new();

    let mut code = quote! {
        use rust_spp::*;
    };
    for (stage, spar_struct) in spar_stream.stages.iter().zip(spar_structs) {
        code.extend(spar_struct);

        if !gen.is_empty() {
            gen.extend(quote!(,));
        }

        gen.extend(rust_spp_pipeline_arg(stage));
    }

    code.extend(quote! {
        let spar_pipeline = rust_spp::pipeline![
            #gen,
            collect!()
        ];

        #dispatcher
    });

    if let Some(collector) = collector {
        code.extend(collector.gen());
    }

    code
}

pub fn codegen(mut spar_stream: SparStream) -> TokenStream {
    let mut code = gen_spar_num_workers();
    code.extend(rust_spp_gen(&mut spar_stream));

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
