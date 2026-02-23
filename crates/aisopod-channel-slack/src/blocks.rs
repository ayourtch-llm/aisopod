//! Block Kit builder module for Slack.
//!
//! This module provides types and builders for constructing rich Slack
//! messages using the Block Kit framework.

use serde::{Deserialize, Serialize};

// ============================================================================
// Text Types
// ============================================================================

/// Plain text type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlainTextType {
    PlainText,
}

/// Plain text element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlainText {
    pub r#type: PlainTextType,
    pub text: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub emoji: Option<bool>,
}

impl PlainText {
    pub fn new(text: &str) -> Self {
        Self {
            r#type: PlainTextType::PlainText,
            text: text.to_string(),
            emoji: None,
        }
    }

    pub fn with_emoji(mut self, emoji: bool) -> Self {
        self.emoji = Some(emoji);
        self
    }
}

/// Markdown text type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MrkdwnType {
    Mrkdwn,
}

/// Markdown text element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mrkdwn {
    pub r#type: MrkdwnType,
    pub text: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub verbatim: Option<bool>,
}

impl Mrkdwn {
    pub fn new(text: &str) -> Self {
        Self {
            r#type: MrkdwnType::Mrkdwn,
            text: text.to_string(),
            verbatim: None,
        }
    }

    pub fn with_verbatim(mut self, verbatim: bool) -> Self {
        self.verbatim = Some(verbatim);
        self
    }
}

// ============================================================================
// Option Types
// ============================================================================

/// Option for select elements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectOption {
    pub text: PlainText,
    pub value: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub description: Option<PlainText>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub url: Option<String>,
}

impl SelectOption {
    pub fn new(text: &str, value: &str) -> Self {
        Self {
            text: PlainText::new(text),
            value: value.to_string(),
            description: None,
            url: None,
        }
    }

    pub fn with_description(mut self, text: &str, description: &str) -> Self {
        self.description = Some(PlainText::new(description));
        self.text = PlainText::new(text);
        self
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
}

/// Option group for select elements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptionGroup {
    pub label: PlainText,
    pub options: Vec<SelectOption>,
}

impl OptionGroup {
    pub fn new(label: &str) -> Self {
        Self {
            label: PlainText::new(label),
            options: Vec::new(),
        }
    }

    pub fn with_option(mut self, option: SelectOption) -> Self {
        self.options.push(option);
        self
    }
}

// ============================================================================
// Confirm Dialog
// ============================================================================

/// Confirm dialog for interactive elements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Confirm {
    pub title: PlainText,
    pub text: Mrkdwn,
    pub confirm: PlainText,
    pub deny: PlainText,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub style: Option<ConfirmStyle>,
}

/// Confirm button style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmStyle {
    Primary,
    Danger,
}

impl Confirm {
    pub fn new(title: &str, text: &str, confirm: &str, deny: &str) -> Self {
        Self {
            title: PlainText::new(title),
            text: Mrkdwn::new(text),
            confirm: PlainText::new(confirm),
            deny: PlainText::new(deny),
            style: None,
        }
    }

    pub fn with_style(mut self, style: ConfirmStyle) -> Self {
        self.style = Some(style);
        self
    }
}

// ============================================================================
// Overflow Options
// ============================================================================

/// Overflow option for action elements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverflowOption {
    pub text: PlainText,
    pub value: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub description: Option<PlainText>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub url: Option<String>,
}

impl OverflowOption {
    pub fn new(text: &str, value: &str) -> Self {
        Self {
            text: PlainText::new(text),
            value: value.to_string(),
            description: None,
            url: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(PlainText::new(description));
        self
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
}

// ============================================================================
// Block Types
// ============================================================================

/// Block type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Section,
    Divider,
    Image,
    Actions,
    Context,
    File,
    Header,
    Call,
}

/// Section block element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SectionElement {
    #[serde(rename = "mrkdwn")]
    Mrkdwn(Mrkdwn),
    #[serde(rename = "image")]
    Image(Image),
}

/// Image element for section blocks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    pub image_url: String,
    pub alt_text: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub title: Option<PlainText>,
}

impl Image {
    pub fn new(image_url: &str, alt_text: &str) -> Self {
        Self {
            image_url: image_url.to_string(),
            alt_text: alt_text.to_string(),
            title: None,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(PlainText::new(title));
        self
    }
}

/// Section block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SectionBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub text: Mrkdwn,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub fields: Option<Vec<Mrkdwn>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub accessory: Option<SectionElement>,
}

impl SectionBlock {
    pub fn new(text: &str) -> Self {
        Self {
            block_type: BlockType::Section,
            block_id: None,
            text: Mrkdwn::new(text),
            fields: None,
            accessory: None,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }

    pub fn with_fields(mut self, fields: Vec<&str>) -> Self {
        self.fields = Some(fields.into_iter().map(|s| Mrkdwn::new(s)).collect());
        self
    }

    pub fn with_image(mut self, image_url: &str, alt_text: &str) -> Self {
        self.accessory = Some(SectionElement::Image(Image::new(image_url, alt_text)));
        self
    }
}

/// Divider block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DividerBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
}

impl DividerBlock {
    pub fn new() -> Self {
        Self {
            block_type: BlockType::Divider,
            block_id: None,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }
}

/// Image block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub image_url: String,
    pub alt_text: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub title: Option<PlainText>,
}

impl ImageBlock {
    pub fn new(image_url: &str, alt_text: &str) -> Self {
        Self {
            block_type: BlockType::Image,
            block_id: None,
            image_url: image_url.to_string(),
            alt_text: alt_text.to_string(),
            title: None,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(PlainText::new(title));
        self
    }
}

/// Actions block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionsBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub elements: Vec<ActionElement>,
}

/// Action element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ActionElement {
    #[serde(rename = "button")]
    Button(Button),
    #[serde(rename = "static_select")]
    StaticSelect(StaticSelect),
    #[serde(rename = "overflow")]
    Overflow(Overflow),
    #[serde(rename = "datepicker")]
    DatePicker(DatePicker),
}

/// Button element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Button {
    pub action_id: String,
    pub text: PlainText,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub style: Option<ButtonStyle>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub confirm: Option<Confirm>,
}

/// Button style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    Primary,
    Danger,
}

impl Button {
    pub fn new(action_id: &str, text: &str, value: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            text: PlainText::new(text),
            value: Some(value.to_string()),
            url: None,
            style: None,
            confirm: None,
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn with_primary_style(mut self) -> Self {
        self.style = Some(ButtonStyle::Primary);
        self
    }

    pub fn with_danger_style(mut self) -> Self {
        self.style = Some(ButtonStyle::Danger);
        self
    }

    pub fn with_confirm(mut self, confirm: Confirm) -> Self {
        self.confirm = Some(confirm);
        self
    }
}

/// Static select element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StaticSelect {
    pub placeholder: PlainText,
    pub action_id: String,
    pub options: Vec<SelectOption>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub option_groups: Option<Vec<OptionGroup>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub initial_option: Option<SelectOption>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub confirm: Option<Confirm>,
}

impl StaticSelect {
    pub fn new(placeholder: &str, action_id: &str, options: Vec<SelectOption>) -> Self {
        Self {
            placeholder: PlainText::new(placeholder),
            action_id: action_id.to_string(),
            options,
            option_groups: None,
            initial_option: None,
            confirm: None,
        }
    }

    pub fn with_option_groups(mut self, groups: Vec<OptionGroup>) -> Self {
        self.option_groups = Some(groups);
        self
    }

    pub fn with_initial_option(mut self, option: SelectOption) -> Self {
        self.initial_option = Some(option);
        self
    }

    pub fn with_confirm(mut self, confirm: Confirm) -> Self {
        self.confirm = Some(confirm);
        self
    }
}

/// Overflow element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Overflow {
    pub action_id: String,
    pub options: Vec<OverflowOption>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub confirm: Option<Confirm>,
}

impl Overflow {
    pub fn new(action_id: &str, options: Vec<OverflowOption>) -> Self {
        Self {
            action_id: action_id.to_string(),
            options,
            confirm: None,
        }
    }

    pub fn with_confirm(mut self, confirm: Confirm) -> Self {
        self.confirm = Some(confirm);
        self
    }
}

/// Date picker element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatePicker {
    pub action_id: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub placeholder: Option<PlainText>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub initial_date: Option<String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub confirm: Option<Confirm>,
}

impl DatePicker {
    pub fn new(action_id: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            placeholder: None,
            initial_date: None,
            confirm: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(PlainText::new(placeholder));
        self
    }

    pub fn with_initial_date(mut self, date: &str) -> Self {
        self.initial_date = Some(date.to_string());
        self
    }

    pub fn with_confirm(mut self, confirm: Confirm) -> Self {
        self.confirm = Some(confirm);
        self
    }
}

impl ActionsBlock {
    pub fn new(elements: Vec<ActionElement>) -> Self {
        Self {
            block_type: BlockType::Actions,
            block_id: None,
            elements,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }
}

/// Context block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub elements: Vec<ContextElement>,
}

/// Context element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContextElement {
    #[serde(rename = "image")]
    Image(Image),
    #[serde(rename = "mrkdwn")]
    Mrkdwn(Mrkdwn),
}

impl ContextBlock {
    pub fn new(elements: Vec<ContextElement>) -> Self {
        Self {
            block_type: BlockType::Context,
            block_id: None,
            elements,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }
}

/// Header block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeaderBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub text: PlainText,
}

impl HeaderBlock {
    pub fn new(text: &str) -> Self {
        Self {
            block_type: BlockType::Header,
            block_id: None,
            text: PlainText::new(text),
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }
}

/// File block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub external_id: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub source: Option<String>,
}

impl FileBlock {
    pub fn new(external_id: &str) -> Self {
        Self {
            block_type: BlockType::File,
            block_id: None,
            external_id: external_id.to_string(),
            source: None,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }
}

/// Call block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallBlock {
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub block_id: Option<String>,
    pub call_id: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub participants: Option<Vec<Participant>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub app_display_name: Option<String>,
}

/// Call participant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    pub user_id: String,
    pub enterprise_id: Option<String>,
}

impl CallBlock {
    pub fn new(call_id: &str) -> Self {
        Self {
            block_type: BlockType::Call,
            block_id: None,
            call_id: call_id.to_string(),
            participants: None,
            app_display_name: None,
        }
    }

    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }

    pub fn with_participants(mut self, participants: Vec<Participant>) -> Self {
        self.participants = Some(participants);
        self
    }

    pub fn with_app_display_name(mut self, name: &str) -> Self {
        self.app_display_name = Some(name.to_string());
        self
    }
}

// ============================================================================
// Block Types Union
// ============================================================================

/// A block in a Slack message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Block {
    #[serde(rename = "section")]
    Section(SectionBlock),
    #[serde(rename = "divider")]
    Divider(DividerBlock),
    #[serde(rename = "image")]
    Image(ImageBlock),
    #[serde(rename = "actions")]
    Actions(ActionsBlock),
    #[serde(rename = "context")]
    Context(ContextBlock),
    #[serde(rename = "file")]
    File(FileBlock),
    #[serde(rename = "header")]
    Header(HeaderBlock),
    #[serde(rename = "call")]
    Call(CallBlock),
}

// ============================================================================
// Block Builder
// ============================================================================

/// Builder for creating blocks.
pub struct BlockBuilder;

impl BlockBuilder {
    pub fn section(text: &str) -> SectionBlock {
        SectionBlock::new(text)
    }

    pub fn divider() -> DividerBlock {
        DividerBlock::new()
    }

    pub fn image(image_url: &str, alt_text: &str) -> ImageBlock {
        ImageBlock::new(image_url, alt_text)
    }

    pub fn actions(elements: Vec<ActionElement>) -> ActionsBlock {
        ActionsBlock::new(elements)
    }

    pub fn context(elements: Vec<ContextElement>) -> ContextBlock {
        ContextBlock::new(elements)
    }

    pub fn file(external_id: &str) -> FileBlock {
        FileBlock::new(external_id)
    }

    pub fn header(text: &str) -> HeaderBlock {
        HeaderBlock::new(text)
    }

    pub fn call(call_id: &str) -> CallBlock {
        CallBlock::new(call_id)
    }

    pub fn plain_text(text: &str) -> PlainText {
        PlainText::new(text)
    }

    pub fn mrkdwn(text: &str) -> Mrkdwn {
        Mrkdwn::new(text)
    }

    pub fn button(action_id: &str, text: &str, value: &str) -> Button {
        Button::new(action_id, text, value)
    }

    pub fn static_select(placeholder: &str, action_id: &str, options: Vec<SelectOption>) -> StaticSelect {
        StaticSelect::new(placeholder, action_id, options)
    }

    pub fn overflow(action_id: &str, options: Vec<OverflowOption>) -> Overflow {
        Overflow::new(action_id, options)
    }

    pub fn date_picker(action_id: &str) -> DatePicker {
        DatePicker::new(action_id)
    }

    pub fn option(text: &str, value: &str) -> SelectOption {
        SelectOption::new(text, value)
    }

    pub fn option_group(label: &str) -> OptionGroup {
        OptionGroup::new(label)
    }

    pub fn confirm(title: &str, text: &str, confirm: &str, deny: &str) -> Confirm {
        Confirm::new(title, text, confirm, deny)
    }

    pub fn overflow_option(text: &str, value: &str) -> OverflowOption {
        OverflowOption::new(text, value)
    }

    pub fn image_element(image_url: &str, alt_text: &str) -> Image {
        Image::new(image_url, alt_text)
    }
}

// ============================================================================
// Block Kit
// ============================================================================

/// A complete Block Kit message payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockKit {
    pub blocks: Vec<Block>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub text: Option<String>,
}

impl BlockKit {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            text: None,
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_serialization() {
        let text = PlainText::new("Hello, world!");
        let json = serde_json::to_string(&text).unwrap();
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("plain_text"));
    }

    #[test]
    fn test_mrkdwn_serialization() {
        let text = Mrkdwn::new("*bold* and _italic_");
        let json = serde_json::to_string(&text).unwrap();
        assert!(json.contains("bold"));
        assert!(json.contains("italic"));
        assert!(json.contains("mrkdwn"));
    }

    #[test]
    fn test_option_serialization() {
        let option = SelectOption::new("Option 1", "opt1");
        let json = serde_json::to_string(&option).unwrap();
        assert!(json.contains("Option 1"));
        assert!(json.contains("opt1"));
    }

    #[test]
    fn test_section_block_serialization() {
        let block = SectionBlock::new("*Hello* _world_");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("section"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_divider_block_serialization() {
        let block = DividerBlock::new();
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("divider"));
    }

    #[test]
    fn test_image_block_serialization() {
        let block = ImageBlock::new("https://example.com/image.png", "Description");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("image"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_button_serialization() {
        // Button is serialized as part of an ActionElement in ActionsBlock
        let element = ActionElement::Button(Button::new("btn1", "Click me", "click_value"));
        let json = serde_json::to_string(&element).unwrap();
        assert!(json.contains("button"));
        assert!(json.contains("Click me"));
    }

    #[test]
    fn test_static_select_serialization() {
        let options = vec![SelectOption::new("Option 1", "opt1")];
        let select = StaticSelect::new("Select an option", "select1", options);
        // StaticSelect is serialized as part of an ActionElement
        let element = ActionElement::StaticSelect(select);
        let json = serde_json::to_string(&element).unwrap();
        assert!(json.contains("static_select"));
        assert!(json.contains("Select an option"));
    }

    #[test]
    fn test_confirm_serialization() {
        let confirm = Confirm::new("Confirm", "Are you sure?", "Yes", "No");
        let json = serde_json::to_string(&confirm).unwrap();
        assert!(json.contains("Confirm"));
        assert!(json.contains("Yes"));
    }

    #[test]
    fn test_block_kit_serialization() {
        let section = SectionBlock::new("*Result*");
        let divider = DividerBlock::new();
        
        let blocks = vec![
            Block::Section(section),
            Block::Divider(divider),
        ];
        
        let kit = BlockKit::new(blocks);
        let json = serde_json::to_string(&kit).unwrap();
        assert!(json.contains("blocks"));
        assert!(json.contains("section"));
        assert!(json.contains("divider"));
    }

    #[test]
    fn test_block_builder() {
        let section = BlockBuilder::section("*Hello* world");
        let divider = BlockBuilder::divider();
        let button = BlockBuilder::button("btn1", "Click", "val1");
        
        assert_eq!(section.block_type, BlockType::Section);
        assert_eq!(divider.block_type, BlockType::Divider);
        assert_eq!(button.action_id, "btn1");
        assert_eq!(button.text.text, "Click");
        assert_eq!(button.value, Some("val1".to_string()));
    }
}
