use thiserror::Error;

#[derive(Error, Debug)]
pub enum SipError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Fetch error: {0}")]
    FetchError(String),

    #[error("Selector error: {0}")]
    SelectorError(String),

    #[error("Text parse error: {0}")]
    TextParseError(String),

    #[error("Post error: {0}")]
    PostError(String),

    #[error("Post parse error: {0}")]
    PostParseError(String),

    #[error("File system error: {0}")]
    FileSystemError(String)
}

#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("Discord app info error: {0}")]
    DiscordAppInfoError(String),

    #[error("Discord builder error: {0}")]
    DiscordBuilderError(String),

    #[error("Webhook error: {0}")]
    DiscordWebhookError(String),

    #[error("Message error: {0}")]
    DiscordMessageError(String)
}