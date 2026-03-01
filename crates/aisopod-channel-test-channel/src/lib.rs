//! Test Channel channel implementation for aisopod.

mod channel;
mod config;
#[cfg(feature = "gateway")]
mod gateway;
mod outbound;
mod runtime;

pub use channel::TestChannelChannel;
pub use config::TestChannelConfig;

use aisopod_channel::ChannelPlugin;

/// Register this channel plugin with the aisopod runtime.
pub fn register() -> Box<dyn ChannelPlugin> {
    Box::new(channel::TestChannelChannel::default())
}
