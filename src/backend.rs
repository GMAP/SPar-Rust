use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn make_tuple(identifiers: &[Ident]) -> TokenStream {
    quote! {
        ( #( #identifiers),* )
    }
}

pub trait Emitter {
    fn gen_clone(&mut self) -> TokenStream;
    fn emit(&mut self) -> TokenStream;
}

pub trait Collector {
    fn gen_clone(&mut self) -> TokenStream;
    fn collect(&mut self) -> TokenStream;
}

pub trait Messenger<E, C>
where
    E: Emitter,
    C: Collector,
{
    fn prepare(&mut self) -> TokenStream;

    fn channel(&mut self, identifiers: &[Ident]) -> (E, C);

    fn finish(&mut self) -> TokenStream;
}

pub struct CrossbeamEmitter {
    identifiers: Vec<Ident>,
    emitter: Ident,
}

impl Emitter for CrossbeamEmitter {
    fn gen_clone(&mut self) -> TokenStream {
        let emitter = &self.emitter;
        quote! {let #emitter = #emitter.clone();}
    }

    fn emit(&mut self) -> TokenStream {
        let tuple = make_tuple(&self.identifiers);
        let emitter = &self.emitter;
        quote! { let _ = #emitter.send(#tuple); }
    }
}

pub struct CrossbeamCollector {
    identifiers: Vec<Ident>,
    collector: Ident,
}

impl Collector for CrossbeamCollector {
    fn gen_clone(&mut self) -> TokenStream {
        let collector = &self.collector;
        quote! {let #collector = #collector.clone();}
    }

    fn collect(&mut self) -> TokenStream {
        let tuple = make_tuple(&self.identifiers);
        let collector = &self.collector;
        quote! {
            let #tuple = match #collector.recv() {
                Ok(v) => v,
                Err(_) => return,
            };
        }
    }
}

pub struct CrossbeamMessenger {
    id: u32,
    emitter: Option<Ident>,
    collector: Option<Ident>,
}

impl CrossbeamMessenger {
    pub fn new() -> Self {
        Self {
            id: 0,
            emitter: None,
            collector: None,
        }
    }
}

impl Messenger<CrossbeamEmitter, CrossbeamCollector> for CrossbeamMessenger {
    fn prepare(&mut self) -> TokenStream {
        if self.emitter.is_some() || self.collector.is_some() {
            panic!("ChannelMessenger has already been prepared");
        }

        let emitter = format!("channel_messenger_sender_{}", self.id);
        let emitter = Ident::new(&emitter, Span::call_site());
        let collector = format!("channel_messenger_receiver_{}", self.id);
        let collector = Ident::new(&collector, Span::call_site());
        let tokens = quote! {
            let ((#emitter, #collector)) = crossbeam_channel::unbounded();
        };

        self.id += 1;
        self.emitter = Some(emitter);
        self.collector = Some(collector);
        tokens
    }

    fn channel(&mut self, identifiers: &[Ident]) -> (CrossbeamEmitter, CrossbeamCollector) {
        if self.emitter.is_none() || self.collector.is_none() {
            panic!("ChannelMessenger must be prepared first!")
        }
        let emitter = self.emitter.as_ref().unwrap();
        let collector = self.collector.as_ref().unwrap();

        (
            CrossbeamEmitter {
                emitter: emitter.clone(),
                identifiers: identifiers.to_vec(),
            },
            CrossbeamCollector {
                collector: collector.clone(),
                identifiers: identifiers.to_vec(),
            },
        )
    }

    fn finish(&mut self) -> TokenStream {
        self.emitter = None;
        self.collector = None;

        quote! {}
    }
}
