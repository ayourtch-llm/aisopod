# Teloxide Message Properties Access - Comprehensive Report

## Overview
This report analyzes all example files in the teloxide repository to extract patterns for accessing common message properties: `message.id`, `message.chat.id`, and `message.text`.

## Files Analyzed
1. admin.rs
2. buttons.rs
3. chat_member_updates.rs
4. command.rs
5. db_remember.rs
6. deep_linking.rs
7. dialogue.rs
8. dispatching_features.rs
9. heroku_ping_pong.rs
10. inline.rs
11. middlewares.rs
12. middlewares_fallible.rs
13. ngrok_ping_pong.rs
14. purchase.rs
15. shared_state.rs
16. throw_dice.rs

---

## 1. Accessing `message.id`

### Pattern: Direct Access
The `id` field is accessed directly as a property of the Message struct.

### Examples:

#### middlewares.rs
```rust
.inspect(|msg: Message| println!("Before (message #{}).", msg.id))
```
**Explanation**: Uses the `inspect` middleware to log the message ID before processing.

#### middlewares_fallible.rs
```rust
.inspect(|msg: Message| println!("Before (message #{}).", msg.id))
```
**Explanation**: Similar pattern - uses inspect middleware to log message ID.

#### shared_state.rs
```rust
.endpoint(|bot: Bot, messages_total: Arc<AtomicU64>, msg: Message| async move {
    let previous = messages_total.fetch_add(1, Ordering::Relaxed);
    bot.send_message(msg.chat.id, format!("I received {previous} messages in total."))
        .await?;
    respond(())
})
```
**Explanation**: Message ID is available in handlers but not always used - the example focuses on `msg.chat.id`.

#### buttons.rs
```rust
async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
            }
```
**Explanation**: Message ID is available but not always explicitly used in examples.

---

## 2. Accessing `message.chat.id`

### Pattern 1: Direct Field Access (`msg.chat.id`)
This is the most common pattern used throughout all examples.

### Pattern 2: Chat ID from Reply (using `msg.reply_to_message()`)

### Pattern 3: Dialogue Storage (`dialogue.chat_id()`)

### Pattern 4: From CallbackQuery (`q.from.message_chat_id()`)

### Pattern 5: From ChatMemberUpdated (`chat_member.chat.id`)

### Examples:

#### admin.rs (Multiple examples of direct access)
```rust
async fn action(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Kick => kick_user(bot, msg).await?,
        Command::Ban { time, unit } => ban_user(bot, msg, calc_restrict_time(time, unit)).await?,
        Command::Mute { time, unit } => mute_user(bot, msg, calc_restrict_time(time, unit)).await?,
    };
    Ok(())
}
```

**Kick user example:**
```rust
async fn kick_user(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            bot.unban_chat_member(msg.chat.id, replied.from.as_ref().unwrap().id).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Use this command in reply to another message").await?;
        }
    }
    Ok(())
}
```

**Ban user example:**
```rust
async fn ban_user(bot: Bot, msg: Message, time: Duration) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            bot.kick_chat_member(
                msg.chat.id,
                replied.from.as_ref().expect("Must be MessageKind::Common").id,
            )
            .until_date(msg.date + time)
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Use this command in a reply to another message!")
                .await?;
        }
    }
    Ok(())
}
```

#### buttons.rs
```rust
async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
            }
            Ok(Command::Start) => {
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:").reply_markup(keyboard).await?;
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }
    Ok(())
}
```

#### command.rs
```rust
async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}.")).await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(msg.chat.id, format!("Your username is @{username} and age is {age}."))
                .await?
        }
    };
    Ok(())
}
```

#### db_remember.rs
```rust
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(n)) => {
            dialogue.update(State::GotNumber(n)).await?;
            bot.send_message(
                msg.chat.id,
                format!("Remembered number {n}. Now use /get or /reset."),
            )
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please, send me a number.").await?;
        }
    }
    Ok(())
}
```

#### deep_linking.rs
```rust
pub async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    start: String,
    me: Me,
) -> HandlerResult {
    if start.is_empty() {
        bot.send_message(
            msg.chat.id,
            format!(
                "Hello!\n\nThis link allows anyone to message you secretly: {}?start={}",
                me.tme_url(),
                msg.chat.id
            ),
        )
        .await?;
        dialogue.exit().await?;
    }
    // ...
}
```

#### dialogue.rs
```rust
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue.update(State::ReceiveAge { full_name: text.into() }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }
    Ok(())
}
```

#### dispatching_features.rs
```rust
.branch(
    dptree::entry()
        .filter_command::<SimpleCommand>()
        .endpoint(simple_commands_handler),
)

// Inside handler:
async fn simple_commands_handler(
    cfg: ConfigParameters,
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
    cmd: SimpleCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        SimpleCommand::Help => {
            // ...
        }
        SimpleCommand::MyId => {
            format!("{}", msg.from.unwrap().id)
        }
    };
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}
```

#### purchase.rs
```rust
async fn receive_product_selection(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String,
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(product) = &q.data {
        bot.send_message(
            dialogue.chat_id(),  // Using dialogue.chat_id() for CallbackQuery
            format!("{full_name}, product '{product}' has been purchased successfully!"),
        )
        .await?;
        dialogue.exit().await?;
    }
    Ok(())
}
```

#### chat_member_updates.rs
```rust
async fn new_chat_member(bot: Bot, chat_member: ChatMemberUpdated) -> ResponseResult<()> {
    let user = chat_member.old_chat_member.user.clone();
    let telegram_group_name = chat_member.chat.title().unwrap_or("");
    let username = user.mention().unwrap_or_else(|| html::user_mention(user.id, user.full_name().as_str()));
    bot.send_message(chat_member.chat.id, format!("Welcome to {telegram_group_name} {username}!"))
        .await?;
    Ok(())
}

async fn left_chat_member(bot: Bot, chat_member: ChatMemberUpdated) -> ResponseResult<()> {
    let user = chat_member.old_chat_member.user;
    let username = user.mention().unwrap_or_else(|| html::user_mention(user.id, user.full_name().as_str()));
    bot.send_message(chat_member.chat.id, format!("Goodbye {username}!")).await?;
    Ok(())
}
```

#### heroku_ping_pong.rs
```rust
teloxide::repl_with_listener(
    bot,
    |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, "pong").await?;
        Ok(())
    },
    listener,
)
```

#### throw_dice.rs
```rust
teloxide::repl(bot, |bot: Bot, msg: Message| async move {
    bot.send_dice(msg.chat.id).await?;
    Ok(())
})
```

---

## 3. Accessing `message.text`

### Pattern 1: Direct field access (`msg.text`)
### Pattern 2: Method call (`msg.text()`)
### Pattern 3: Pattern matching on `msg.text()`

### Examples:

#### buttons.rs
```rust
if let Some(text) = msg.text() {
    match BotCommands::parse(text, me.username()) {
        Ok(Command::Help) => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        // ...
    }
}
```

#### dialogue.rs
```rust
async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue.update(State::ReceiveAge { full_name: text.into() }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }
    Ok(())
}
```

#### db_remember.rs
```rust
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(n)) => {
            dialogue.update(State::GotNumber(n)).await?;
            bot.send_message(
                msg.chat.id,
                format!("Remembered number {n}. Now use /get or /reset."),
            )
            .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please, send me a number.").await?;
        }
    }
    Ok(())
}
```

#### deep_linking.rs
```rust
pub async fn send_message(
    bot: Bot,
    id: ChatId,
    msg: Message,
    dialogue: MyDialogue,
    me: Me,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let sent_result = bot
                .send_message(id, format!("You have a new message!\n\n<i>{text}</i>"))
                .parse_mode(ParseMode::Html)
                .await;
            // ...
        }
        None => {
            bot.send_message(msg.chat.id, "This bot can send only text.").await?;
        }
    };
    Ok(())
}
```

#### purchase.rs
```rust
async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(full_name) => {
            // ...
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me your full name.").await?;
        }
    }
    Ok(())
}
```

#### simple example from buttons.rs
```rust
if let Some(text) = msg.text() {
    match BotCommands::parse(text, me.username()) {
        Ok(Command::Help) => { /* ... */ }
        // ...
    }
}
```

---

## 4. Other Commonly Accessed Message Properties

### `msg.from` - User who sent the message
```rust
// admin.rs
bot.unban_chat_member(msg.chat.id, replied.from.as_ref().unwrap().id).await?;

// buttons.rs
bot.send_message(msg.chat.id, format!("Your username is @{username}.")).await?

// dispatching_features.rs
format!("{}", msg.from.unwrap().id)
```

### `msg.chat` - Chat where the message was sent
```rust
// chat_member_updates.rs
let telegram_group_name = chat_member.chat.title().unwrap_or("");

// dispatching_features.rs
msg.chat.is_group() || msg.chat.is_supergroup()
```

### `msg.reply_to_message()` - Reply to another message
```rust
// admin.rs
match msg.reply_to_message() {
    Some(replied) => {
        bot.unban_chat_member(msg.chat.id, replied.from.as_ref().unwrap().id).await?;
    }
    None => {
        bot.send_message(msg.chat.id, "Use this command in reply to another message").await?;
    }
}
```

### `msg.date` - Message timestamp
```rust
// admin.rs
bot.kick_chat_member(msg.chat.id, replied.from.as_ref().expect("Must be MessageKind::Common").id)
    .until_date(msg.date + time)
    .await?;
```

### `msg.id` - Message ID (used in middlewares)
```rust
// middlewares.rs
.inspect(|msg: Message| println!("Before (message #{}).", msg.id))
```

### `msg.from.as_ref().unwrap().id` - Extracting user ID from message
```rust
// dispatching_features.rs
format!("{}", msg.from.unwrap().id)
```

### `msg.chat.is_group()` and `msg.chat.is_supergroup()` - Chat type checks
```rust
// dispatching_features.rs
dptree::filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
```

### `msg.chat.title()` - Chat title
```rust
// chat_member_updates.rs
let telegram_group_name = chat_member.chat.title().unwrap_or("");
```

---

## Summary of Access Patterns

### Message.ID
- **Pattern**: `msg.id`
- **Usage**: Mostly in middleware `inspect` handlers for logging
- **Note**: Not commonly used in regular message handlers

### Message.Chat.ID
- **Pattern**: `msg.chat.id`
- **Usage**: Extremely common - used in almost every handler
- **Purpose**: Sending messages back to the chat where the original message was received

### Message.Text
- **Pattern 1**: `msg.text()` - Returns `Option<&str>`
- **Pattern 2**: `if let Some(text) = msg.text() { ... }` - Pattern matching
- **Pattern 3**: `msg.text().map(...)` - For parsing or transforming
- **Usage**: Common when processing text commands or content

### Key Observations

1. **Most Common Pattern**: `msg.chat.id` is used extensively to send responses back to the correct chat.

2. **Text Access**: `msg.text()` returns `Option<&str>` and must be handled appropriately (pattern matching, map, or unwrap).

3. **Reply Handling**: `msg.reply_to_message()` returns `Option<&Message>` for handling replies.

4. **Chat Member Updates**: For `ChatMemberUpdated` updates, use `chat_member.chat.id` instead of `msg.chat.id`.

5. **Dialogue Pattern**: When working with dialogues, `dialogue.chat_id()` can be used for callback queries.

6. **Middleware Pattern**: The `inspect` middleware often logs `msg.id` for debugging purposes.

7. **User Access**: `msg.from` returns `Option<User>` and requires safe access with `as_ref()`, `unwrap()`, or pattern matching.

---

## Complete Code Snippet Reference

### Basic Message Handler Pattern
```rust
async fn handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    // Accessing message properties
    let message_id = msg.id;
    let chat_id = msg.chat.id;
    let text = msg.text(); // Option<&str>
    
    // Sending a response
    bot.send_message(chat_id, "Response text").await?;
    
    Ok(())
}
```

### With Text Processing
```rust
async fn handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        // Process text
        bot.send_message(msg.chat.id, format!("You said: {}", text)).await?;
    } else {
        bot.send_message(msg.chat.id, "Please send text messages only.").await?;
    }
    Ok(())
}
```

### With Reply Handling
```rust
async fn handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            // Handle reply
            let replied_user_id = replied.from.as_ref().unwrap().id;
            bot.send_message(msg.chat.id, format!("Replied to user: {}", replied_user_id)).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please reply to another message.").await?;
        }
    }
    Ok(())
}
```

### With Chat Type Detection
```rust
async fn handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    if msg.chat.is_group() || msg.chat.is_supergroup() {
        bot.send_message(msg.chat.id, "This is a group chat").await?;
    } else {
        bot.send_message(msg.chat.id, "This is a private chat").await?;
    }
    Ok(())
}
```
