pub mod app;
pub mod components;
pub mod mpris_server;
pub mod player;
pub mod playerbar;
pub mod queue;
pub mod router;
pub mod sidebar;

use iced::Subscription;

/// A global data/message receiver.
/// This let's us sidestep the routing problem at the cost of less transparent
/// data flows. See the [macros] crate for implementation details.
pub trait Receiver<LocalMsg> {
    /// Sends the data to the receiver
    fn send(msg: impl Into<LocalMsg>);

    /// Returns a subscription of all received messages
    fn receive() -> Subscription<LocalMsg>;

    /// Maps the received subcriptions to the top-level Message
    /// so that the subscription can be easily registered
    fn collect() -> Subscription<app::Message>;
}

/// A container for subscriptions which can be registered by [`inventory`]
pub struct ReceiverContainer(pub fn() -> iced::Subscription<app::Message>);

// Register
inventory::collect!(ReceiverContainer);
