use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn make_tuple(identifiers: &[Ident]) -> TokenStream {
    quote!{ 
        ( #( #identifiers),* )
    }
}

pub trait Messenger {
    fn gen_prep(&mut self) -> TokenStream;
    fn gen_send(&mut self, identifiers: &[Ident]) -> TokenStream;
    fn gen_recv(&mut self, identifiers: &[Ident]) -> TokenStream;
    fn gen_finish(&mut self) -> TokenStream;

    fn gen_sender_clone(&self) -> TokenStream;
    fn gen_receiver_clone(&self) -> TokenStream;
}

pub struct CrossbeamMessenger {
    id: u32,
    sender: Option<Ident>,
    receiver: Option<Ident>,
}

impl CrossbeamMessenger {
    pub fn new() -> Self {
        Self {
            id: 0,
            sender: None,
            receiver: None,
        }
    }
}

impl Messenger for CrossbeamMessenger {
    fn gen_prep(&mut self) -> TokenStream {
        if self.sender.is_some() || self.receiver.is_some() {
            panic!("ChannelMessenger has already been prepared");
        }

        let sender = format!("channel_messenger_sender_{}", self.id);
        let sender = Ident::new(&sender, Span::call_site());
        let receiver = format!("channel_messenger_receiver_{}", self.id);
        let receiver = Ident::new(&receiver, Span::call_site());
        let tokens = quote! {
            let ((#sender, #receiver)) = crossbeam_channel::unbounded();
        };

        self.id += 1;
        self.sender = Some(sender);
        self.receiver = Some(receiver);
        tokens
    }

    fn gen_send(&mut self, identifiers: &[Ident]) -> TokenStream {
        let tuple = make_tuple(identifiers);
        let sender = self
            .sender
            .as_ref()
            .expect("call `gen_prep` before `gen_send`");

        quote! {
            #sender.send(#tuple).unwrap();
        }
    }

    fn gen_recv(&mut self, identifiers: &[Ident]) -> TokenStream {
        let tuple = make_tuple(identifiers);
        let receiver = self
            .receiver
            .as_ref()
            .expect("call `gen_prep` before `gen_recv`");

        quote! {
            let (#tuple) = #receiver.recv().unwrap();
        }
    }

    fn gen_finish(&mut self) -> TokenStream {
        self.sender = None;
        self.receiver = None;
        quote! {}
    }

    fn gen_sender_clone(&self) -> TokenStream {
        if self.sender.is_none() {
            panic!("call `gen_prep` before `gen_clone_sender`")
        }

        let sender = self.sender.as_ref().unwrap();
        quote! {let #sender = #sender.clone();}
    }

    fn gen_receiver_clone(&self) -> TokenStream {
        if self.sender.is_none() {
            panic!("call `gen_prep` before `gen_clone_receiver`")
        }
        let receiver = &self.receiver.as_ref().unwrap();
        quote! {let #receiver = #receiver.clone();}
    }
}
