use proc_macro2::{Delimiter, Group, Ident, Punct, Span, TokenStream};
use quote::{quote, ToTokens};

pub fn make_tuple(identifiers: &[Ident]) -> TokenStream {
    let mut tuple = TokenStream::new();
    let mut ident_n = identifiers.len();
    for ident in identifiers {
        ident_n -= 1;
        tuple.extend(ident.to_token_stream());
        if ident_n > 0 {
            tuple.extend(Punct::new(',', proc_macro2::Spacing::Alone).to_token_stream());
        }
    }

    Group::new(Delimiter::Parenthesis, tuple).to_token_stream()
}

pub trait Messenger {
    fn gen_prep(&mut self) -> TokenStream;
    fn gen_send(&mut self) -> TokenStream;
    fn gen_recv(&mut self) -> TokenStream;
}

pub struct ChannelMessenger {
    id: u32,
    sender: Option<Ident>,
    receiver: Option<Ident>,
    identifiers: Vec<Ident>,
}

impl ChannelMessenger {
    pub fn new(identifiers: Vec<Ident>) -> Self {
        Self {
            id: 0,
            identifiers,
            sender: None,
            receiver: None,
        }
    }
}

impl Messenger for ChannelMessenger {
    fn gen_prep(&mut self) -> TokenStream {
        let sender = format!("channel_messenger_sender_{}", self.id);
        let sender = Ident::new(&sender, Span::call_site());
        let receiver = format!("channel_messenger_receiver_{}", self.id);
        let receiver = Ident::new(&receiver, Span::call_site());
        let tokens = quote! {
            let ((#sender, #receiver)) = std::sync::mpsc::channel();
        };

        self.id += 1;
        self.sender = Some(sender);
        self.receiver = Some(receiver);
        tokens
    }

    fn gen_send(&mut self) -> TokenStream {
        let tuple = make_tuple(&self.identifiers);
        let sender = self
            .sender
            .as_ref()
            .expect("call `gen_prep` before `gen_send`");

        quote! {
            #sender.send(#tuple).unwrap();
        }
    }

    fn gen_recv(&mut self) -> TokenStream {
        let tuple = make_tuple(&self.identifiers);
        let receiver = self
            .receiver
            .as_ref()
            .expect("call `gen_prep` before `gen_recv`");

        quote! {
            let (#tuple) = #receiver.recv().unwrap();
        }
    }
}
