use std::num::NonZeroU32;

use crate::spar_stream::SparStream;
use proc_macro2::TokenStream;
use quote::quote;

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
        stages,
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

    codegen.extend(code);
    for (i, stage) in stages.iter().enumerate() {
        let input = &stage.attrs.input;
        let output = &stage.attrs.output;
        let replicate = gen_replicate(&stage.attrs.replicate);

        codegen.extend(stage.code.clone().into_iter());
        codegen.extend::<TokenStream>(
            quote! {
                println!("\tStage[{}]:", #i);
                println!("\t\tInput:");
                #(println!("\t\t\t{}", #input));*;
                println!("\t\tOutput:");
                #(println!("\t\t\t{}", #output));*;
                println!("\t\tReplicate: {}", #replicate);
            }
            .into(),
        );
    }

    codegen
}
