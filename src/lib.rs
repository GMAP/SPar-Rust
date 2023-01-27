mod codegen;
mod spar_stream;

use codegen::codegen;
use spar_stream::SparStream;

#[proc_macro]
pub fn to_stream(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match SparStream::try_from(item) {
        Ok(spar_stream) => {
            //dbg!(&spar_stream.code);
            for stage in &spar_stream.stages {
                //dbg!(&stage.code);
            }
            codegen(spar_stream).into()
        }
        Err(e) => e.into_compile_error().into(),
    }
}
