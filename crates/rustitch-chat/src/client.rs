//! High-level chat client scaffolding.

use tokio_util::sync::CancellationToken;

/// Primary runtime used to read chat messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatReadTransport {
    /// EventSub-based chat intake.
    EventSub,
    /// Optional IRC compatibility intake.
    Irc,
}

/// Primary runtime used to write chat messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatWriteTransport {
    /// Helix `Send Chat Message`.
    HelixSendChatMessage,
    /// IRC compatibility sending path.
    IrcCompatibility,
}

/// High-level, transport-agnostic chat client scaffold.
#[derive(Clone)]
pub struct ChatClient {
    read_transport: ChatReadTransport,
    write_transport: ChatWriteTransport,
    shutdown: CancellationToken,
    #[cfg(feature = "chat-irc")]
    irc_transport: Option<rustitch_chat_irc::IrcTransportConfig>,
}

impl ChatClient {
    /// Starts building a chat client.
    #[must_use]
    pub fn builder() -> ChatClientBuilder {
        ChatClientBuilder::default()
    }

    /// Returns the configured primary read transport.
    #[must_use]
    pub fn read_transport(&self) -> ChatReadTransport {
        self.read_transport
    }

    /// Returns the configured primary write path.
    #[must_use]
    pub fn write_transport(&self) -> ChatWriteTransport {
        self.write_transport
    }

    /// Returns the graceful shutdown token.
    #[must_use]
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }

    /// Returns the configured IRC transport, when the feature is enabled.
    #[cfg(feature = "chat-irc")]
    #[must_use]
    pub fn irc_transport(&self) -> Option<&rustitch_chat_irc::IrcTransportConfig> {
        self.irc_transport.as_ref()
    }
}

/// Builder for [`ChatClient`].
#[derive(Clone, Default)]
pub struct ChatClientBuilder {
    read_transport: Option<ChatReadTransport>,
    write_transport: Option<ChatWriteTransport>,
    #[cfg(feature = "chat-irc")]
    irc_transport: Option<rustitch_chat_irc::IrcTransportConfig>,
}

impl ChatClientBuilder {
    /// Selects the primary read transport.
    #[must_use]
    pub fn read_transport(mut self, read_transport: ChatReadTransport) -> Self {
        self.read_transport = Some(read_transport);
        self
    }

    /// Selects the primary write transport.
    #[must_use]
    pub fn write_transport(mut self, write_transport: ChatWriteTransport) -> Self {
        self.write_transport = Some(write_transport);
        self
    }

    /// Configures the optional IRC transport adapter.
    #[cfg(feature = "chat-irc")]
    #[must_use]
    pub fn irc_transport(mut self, irc_transport: rustitch_chat_irc::IrcTransportConfig) -> Self {
        self.irc_transport = Some(irc_transport);
        self
    }

    /// Builds the chat client.
    #[must_use]
    pub fn build(self) -> ChatClient {
        ChatClient {
            read_transport: self.read_transport.unwrap_or(ChatReadTransport::EventSub),
            write_transport: self
                .write_transport
                .unwrap_or(ChatWriteTransport::HelixSendChatMessage),
            shutdown: CancellationToken::new(),
            #[cfg(feature = "chat-irc")]
            irc_transport: self.irc_transport,
        }
    }
}
