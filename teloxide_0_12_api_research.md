# Teloxide 0.12 Crate API Research

## Summary

This document provides detailed answers about the teloxide 0.12 crate API based on source code analysis.

---

## 1. How to use Dispatcher - Constructor and Usage

### Dispatcher Struct Definition

The `Dispatcher` struct is defined in `teloxide/src/dispatching/dispatcher.rs`:

```rust
pub struct Dispatcher<R, Err, Key = DefaultKey> {
    pub(crate) bot: R,
    pub(crate) dependencies: DependencyMap,
    pub(crate) handler: UpdateHandler<Err>,
    pub(crate) default_handler: DefaultHandler<Err>,
    pub(crate) distribution_f: fn(&Update) -> Option<Key>,
    pub(crate) worker_queue_size: usize,
    pub(crate) workers: usize,
    pub(crate) default_worker: usize,
    pub(crate) error_handler: Arc<dyn ErrorHandler<Err> + Send + Sync>,
    pub(crate) state: DispatcherState,
}
```

### Constructor Pattern

**The correct way to construct a Dispatcher is using the builder pattern:**

```rust
Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![SomeDependency::new()])
    .default_handler(|upd| async move { /* handle unhandled updates */ })
    .error_handler(LoggingErrorHandler::with_custom_text("Error!"))
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
```

### DispatcherBuilder Methods

```rust
pub struct DispatcherBuilder<R, Err, Key> {
    // ...
}

impl<R, Err> DispatcherBuilder<R, Err, DefaultKey> {
    pub fn builder(bot: R, handler: UpdateHandler<Err>) -> Self {
        // Constructor
    }
}

impl<R, Err, Key> DispatcherBuilder<R, Err, Key> {
    pub fn default_handler<H, Fut>(self, handler: H) -> Self
    where
        H: IntoDefaultHandler<Err, Fut>;

    pub fn error_handler(self, handler: Arc<dyn ErrorHandler<Err> + Send + Sync>) -> Self;

    pub fn dependencies(self, dependencies: DependencyMap) -> Self;

    pub fn distribution_function<K>(self, f: fn(&Update) -> Option<K>) -> DispatcherBuilder<R, Err, K>;

    pub fn worker_queue_size(self, size: usize) -> Self;

    pub fn workers(self, workers: usize) -> Self;

    pub fn default_worker(self, workers: usize) -> Self;

    pub fn build(self) -> Dispatcher<R, Err, Key>;
}
```

### Usage Example

```rust
use teloxide::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .endpoint(|bot: Bot, msg: Message| async move {
            bot.send_message(msg.chat.id, "Hello!").await?;
            Ok(())
        });

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
```

---

## 2. How to Access Message Fields - Methods vs Fields

### Message Struct Definition

From `teloxide-core/src/types/message.rs`:

```rust
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,

    /// Unique identifier of a message thread to which the message belongs
    pub thread_id: Option<ThreadId>,

    /// Sender, if the message was sent by an authorized user
    pub from: Option<User>,

    /// Source of the message
    pub sender_chat: Option<Chat>,

    /// Date the message was sent
    pub date: DateTime<Utc>,

    /// Chat the message belongs to
    pub chat: Chat,

    /// True if the message is a topic message
    pub is_topic_message: bool,

    /// True if the message is a forward
    pub is_forward: bool,

    /// True if the message is a reply
    pub is_reply: bool,

    /// Bot through which the message was sent
    pub via_bot: Option<User>,

    /// Business bot that sent the message
    pub sender_business_bot: Option<User>,

    /// Message content
    pub kind: MessageKind,
}
```

### Accessing Message Fields

- **Direct Fields (no method needed):**
  - `msg.id` - MessageId
  - `msg.from` - Option<User>
  - `msg.chat` - Chat (direct field)
  - `msg.date` - DateTime<Utc>
  - `msg.is_topic_message` - bool
  - `msg.is_forward` - bool
  - `msg.is_reply` - bool

- **Getter Methods:**
  - `msg.text()` - `Option<&str>` - Returns text content if available
  - `msg.caption()` - `Option<&str>` - Returns caption if available

### Example Usage

```rust
// Direct field access
let message_id = msg.id;           // MessageId
let chat = msg.chat.clone();       // Chat
let chat_id = msg.chat.id;         // ChatId

// Using getter methods
let text = msg.text();             // Option<&str>
let text_str = msg.text().unwrap_or("");  // &str with default
```

---

## 3. How to Access Chat Fields - Methods vs Fields

### Chat Struct Definition

From `teloxide-core/src/types/chat.rs`:

```rust
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Chat {
    /// A unique identifier for this chat.
    pub id: ChatId,

    #[serde(flatten)]
    pub kind: ChatKind,
}
```

### ChatKind Variants

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(untagged)]
pub enum ChatKind {
    Public(ChatPublic),
    Private(ChatPrivate),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ChatPublic {
    pub title: Option<String>,
    #[serde(flatten)]
    pub kind: PublicChatKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ChatPrivate {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
```

### PublicChatKind

```rust
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum PublicChatKind {
    Channel(PublicChatChannel),
    Group,
    Supergroup(PublicChatSupergroup),
}
```

### Direct Fields

- `chat.id` - ChatId (direct field)
- `chat.kind` - ChatKind (direct field)
- For Public chats: `chat.kind` contains `ChatPublic` with `title`, `username`, etc.
- For Private chats: `chat.kind` contains `ChatPrivate` with `username`, `first_name`, `last_name`

### Getter Methods

```rust
impl Chat {
    pub fn is_private(&self) -> bool;
    pub fn is_group(&self) -> bool;
    pub fn is_supergroup(&self) -> bool;
    pub fn is_channel(&self) -> bool;
    pub fn is_chat(&self) -> bool;

    // Getters
    pub fn title(&self) -> Option<&str>;
    pub fn username(&self) -> Option<&str>;
    pub fn first_name(&self) -> Option<&str>;
    pub fn last_name(&self) -> Option<&str>;
}
```

### Example Usage

```rust
// Direct field access
let chat_id = msg.chat.id;  // ChatId

// Using getter methods
let chat_title = msg.chat.title();     // Option<&str>
let username = msg.chat.username();    // Option<&str>
let first_name = msg.chat.first_name(); // Option<&str>

// Checking chat type
if msg.chat.is_group() || msg.chat.is_supergroup() {
    // Handle group chat
} else if msg.chat.is_private() {
    // Handle private chat
}
```

---

## 4. Handler Function Type for Dispatcher

### UpdateHandler Type Alias

```rust
pub type UpdateHandler<Err> = dptree::Handler<'static, Result<(), Err>, DpHandlerDescription>;
```

### Handler Signature

The handler is a `dptree::Handler` with:
- `'static` lifetime
- Returns `Result<(), Err>` 
- With `DpHandlerDescription` for metadata

### Common Handler Patterns

```rust
// Simple handler with bot and message
|bot: Bot, msg: Message| async move {
    bot.send_message(msg.chat.id, "Hello!").await?;
    Ok(())
}

// Handler with dependencies
|cfg: Config, bot: Bot, msg: Message| async move {
    bot.send_message(msg.chat.id, cfg.message).await?;
    Ok(())
}

// Handler with extracted values
|msg: Message, bot: Bot, text: String| async move {
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}
```

### HandlerExt Methods

```rust
pub trait HandlerExt: Sized {
    fn endpoint<H, Fut>(self, endpoint: H) -> Self
    where
        H: IntoEndpoint<Handler = Self>;

    fn branch<H: Handler<Err = Self::Err>>(self, other: H) -> Self;
    fn filter<F, T>(self, filter: F) -> Self
    where
        F: Fn(T) -> bool + Send + Sync + 'static,
        T: 'static;
}
```

---

## 5. Getting Message IDs, Chat IDs, and Other Properties

### Message Properties

```rust
// Message ID
let message_id: MessageId = msg.id;

// Chat ID
let chat_id: ChatId = msg.chat.id;

// From User ID
let from_user_id: Option<UserId> = msg.from.as_ref().map(|user| user.id);

// Date
let date: DateTime<Utc> = msg.date;

// Thread ID (for topic messages)
let thread_id: Option<ThreadId> = msg.thread_id;

// Message text
let text: Option<&str> = msg.text();

// Reply information
let is_reply: bool = msg.is_reply;
let is_forward: bool = msg.is_forward;
```

### Complete Example

```rust
use teloxide::prelude::*;

async fn handle_message(bot: Bot, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    // Message ID
    log::info!("Message ID: {}", msg.id.0);  // MessageId is a newtype
    
    // Chat ID
    log::info!("Chat ID: {}", msg.chat.id.0);
    
    // User information
    if let Some(from) = &msg.from {
        log::info!("From user: {} ({})", from.first_name, from.id.0);
    }
    
    // Text content
    if let Some(text) = msg.text() {
        log::info!("Text: {}", text);
    }
    
    // Chat information
    let chat_id = msg.chat.id;
    let chat_title = msg.chat.title();
    
    Ok(())
}
```

---

## 6. Notify's Notify Struct - Clone Method

### Findings

**The `notify` crate (version 8.2.0) does NOT have a `Notify` struct.**

The `notify` crate is a file watching library with these main types:
- `Event` - File system event
- `Watcher` - Trait for file watchers
- `RecommendedWatcher` - Platform-specific watcher implementation
- `EventKind` - Type of event
- `Config` - Watcher configuration

### Notify Crate Types

From the notify crate documentation and source:

```rust
// Event implements Clone
#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub paths: Vec<PathBuf>,
    pub cause: EventCause,
}

// Config implements Clone + Copy
#[derive(Clone, Copy, Debug)]
pub struct Config {
    // ...
}

// Watchers do NOT implement Clone
// INotifyWatcher (Linux), PollWatcher, NullWatcher - no Clone impl
```

### Async Usage Pattern

```rust
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Result};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    
    while let Some(event) = rx.recv().await {
        match event {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    // Handle file modification
                }
            }
            Err(e) => log::error!("Watch error: {}", e),
        }
    }
    
    Ok(())
}
```

### For Async Signaling in Teloxide

The user likely meant `tokio::sync::Notify` (from the tokio crate), which is commonly used in async Rust code for notification/signaling purposes.

#### tokio::sync::Notify

From tokio crate (not notify crate):

```rust
use tokio::sync::Notify;
use std::sync::Arc;

// Create a new Notify
let notify = Arc::new(Notify::new());

// Clone is available (Arc<Notify> is Send + Sync)
let notify_clone = notify.clone();
```

**Key characteristics:**
- Implements `Clone` - can be cloned to share across tasks
- Is `Send + Sync` - safe to use across threads
- Used for async coordination (like a condition variable)

#### Example from Codebase (aisopod-channel-telegram/src/lib.rs)

```rust
use tokio::sync::Notify;
use std::sync::Arc;

// In struct definition
struct TelegramChannel {
    shutdown_signal: Option<Arc<Notify>>,
    // ...
}

// Creating and using Notify
impl TelegramChannel {
    pub async fn start_long_polling(&mut self, account_id: &str) -> Result<()> {
        // Create shutdown signal
        let shutdown = Arc::new(Notify::new());
        self.shutdown_signal = Some(shutdown.clone());

        let shutdown_task = shutdown.clone();

        let task = async move {
            // ... do work ...
            
            // Wait for shutdown signal
            shutdown_task.notified().await;
        };

        Ok(task)
    }

    pub async fn stop(&mut self) {
        if let Some(shutdown) = &self.shutdown_signal {
            shutdown.notify_one();  // Notify one waiter
            // or shutdown.notify_waiters() to notify all
        }
    }
}
```

#### std::sync::Notify

From standard library (Rust 1.75+):

```rust
use std::sync::Notify;
use std::sync::Arc;

let notify = Arc::new(Notify::new());
let notify_clone = notify.clone();  // Clone is available
```

**Differences:**
- `std::sync::Notify` - synchronous, simpler
- `tokio::sync::Notify` - async-aware, more features (like `notified()` future)

#### Alternative: tokio::sync::watch::Sender

For watching value changes:

```rust
use tokio::sync::watch;

let (tx, rx) = watch::channel(initial_value);
let tx_clone = tx.clone();  // Sender implements Clone

// Send updates
tx.send(new_value).unwrap();

// In async task
loop {
    tokio::select! {
        _ = rx.changed() => {
            let value = rx.borrow();
            // Handle change
        }
    }
}
```

---

## Complete Working Examples

### Example 1: Basic Dispatcher with Message Handling

```rust
use teloxide::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .branch(
            Message::filter_text().endpoint(|bot: Bot, msg: Message, text: String| async move {
                bot.send_message(msg.chat.id, format!("You said: {}", text)).await?;
                Ok(())
            }),
        )
        .branch(
            Message::filter_dice().endpoint(|bot: Bot, msg: Message| async move {
                bot.send_message(msg.chat.id, "Nice dice!").await?;
                Ok(())
            }),
        );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
```

### Example 2: Using Chat Methods

```rust
async fn process_message(bot: Bot, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    let chat_id = msg.chat.id;
    
    // Check chat type
    if msg.chat.is_group() || msg.chat.is_supergroup() {
        // Get title for group chats
        if let Some(title) = msg.chat.title() {
            log::info!("Group: {} (title: {})", chat_id.0, title);
        }
    } else if msg.chat.is_private() {
        // Get user name for private chats
        let name = msg.chat.first_name().unwrap_or("Unknown");
        log::info!("Private chat with: {}", name);
    }
    
    Ok(())
}
```

### Example 3: With Dependencies

```rust
use teloxide::dispatching::dialogue::InMemStorage;
use dptree::deps;

#[derive(Clone)]
struct AppState {
    bot_maintainer: UserId,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::from_env();
    let state = AppState { bot_maintainer: UserId(12345) };

    let handler = Update::filter_message()
        .endpoint(|state: AppState, bot: Bot, msg: Message| async move {
            if msg.from.map(|u| u.id).unwrap_or_default() == state.bot_maintainer {
                bot.send_message(msg.chat.id, "Hello, maintainer!").await?;
            } else {
                bot.send_message(msg.chat.id, "Hello, user!").await?;
            }
            Ok(())
        });

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
```

---

## Key Takeaways

1. **Dispatcher**: Use `Dispatcher::builder(bot, handler)` pattern
2. **Message fields**: Direct fields (`msg.id`, `msg.chat`) with methods like `msg.text()`
3. **Chat fields**: Direct fields (`chat.id`, `chat.kind`) with getter methods (`chat.title()`, `chat.username()`)
4. **Handler type**: `UpdateHandler<Err> = dptree::Handler<'static, Result<(), Err>, DpHandlerDescription>`
5. **Notify crate**: Has no `Notify` struct - use `tokio::sync::Notify` for async signaling
6. **Message/Chat IDs**: Access via `msg.id`, `msg.chat.id` (both are newtype wrappers around i64)
