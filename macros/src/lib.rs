//! This module implements the proc_macro_derive for the Receiver
//!
//! The idea is that we can automatically generate a publich broadcast for each
//! receiver and expose that broadcast through an associated `send` function.
//!
//! Each message we receive on that broadcast can then be received through
//! subscribing, i.e. the associated `receive` function.
//!
//! To automatically register all receivers so we can send to them from anywhere
//! at any time we use dtolnay's inventory crate. The registered subscriptions
//! are then activated in [`frontend::ui::app`].

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, parse_macro_input};

fn find_message_attr(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs.iter().find(|a| a.path().is_ident("message"))
}

#[proc_macro_derive(Receiver, attributes(message))]
pub fn derive_receiver_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let message_attr = find_message_attr(&input.attrs);
    let name = &input.ident;

    // Find the message attribute
    let message_type = message_attr
        .map(|a| {
            a.parse_args::<syn::Path>()
                .expect("expected a path like PlayerMsg")
        })
        .expect(
            "You need to provide the message type through an attribute, e.g. #[message(PlayerMsg)]",
        );

    // Expand
    let expanded = quote! {
        static __RECEIVER_CHANNEL: ::std::sync::LazyLock<::tokio::sync::broadcast::Sender<#message_type>> =
            ::std::sync::LazyLock::new(|| ::tokio::sync::broadcast::channel(128).0);

        #[automatically_derived]
        impl Receiver<#message_type> for #name {
            // Convert the message and send it into the global channel
            fn send(msg: impl Into<#message_type>) {
                let message = msg.into();
                __RECEIVER_CHANNEL.send(message).unwrap();
            }

            fn receive() -> iced::Subscription<#message_type> {
                fn subscribe_global_events() -> impl ::iced::futures::Stream<Item = #message_type> {
                    use ::tokio_stream::StreamExt;

                    let stream: ::tokio::sync::broadcast::Receiver<#message_type> = __RECEIVER_CHANNEL.subscribe();
                    ::tokio_stream::wrappers::BroadcastStream::new(stream)
                        .filter_map(Result::ok)
                }

                iced::Subscription::run(subscribe_global_events)
            }

            fn collect() -> iced::Subscription<crate::ui::app::Message> {
                Self::receive().map(Into::into)
            }
        }

        ::inventory::submit! {
            crate::ui::ReceiverContainer(#name::collect)
        }
    };

    TokenStream::from(expanded)
}
