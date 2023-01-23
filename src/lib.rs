use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Block};

mod kw {
    syn::custom_keyword!(IN);
    syn::custom_keyword!(OUT);
    syn::custom_keyword!(STAGE);
    syn::custom_keyword!(REPLICATE);
}

#[proc_macro]
pub fn to_stream(item: TokenStream) -> TokenStream {
    let ast_item: Block = parse(item).unwrap();
    TokenStream::from(quote!())
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//}
