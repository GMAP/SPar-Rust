use syn::parse_macro_input;

mod codegen;
mod spar_stream;

use codegen::codegen;
use spar_stream::SparStream;

#[proc_macro]
pub fn to_stream(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let spar_stream: SparStream = parse_macro_input!(item as SparStream);
    codegen(spar_stream).into()
}

#[cfg(test)]
mod tests {
    //use super::*;
}
