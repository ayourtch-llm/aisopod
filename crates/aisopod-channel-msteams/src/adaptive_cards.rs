//! Adaptive Cards support for Microsoft Teams.
//!
//! This module provides functionality for creating and using Adaptive Cards
//! in Microsoft Teams messages. Adaptive Cards are a way to exchange rich
//! content between applications and services.

use serde::{Deserialize, Serialize};

/// Adaptive Card for Microsoft Teams.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdaptiveCard {
    /// Card type (always "AdaptiveCard")
    #[serde(rename = "type")]
    pub card_type: String,
    /// Adaptive Card version
    pub version: String,
    /// Card body content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Vec<AdaptiveCardElement>>,
    /// Card actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<AdaptiveCardAction>>,
    /// Card style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Background image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<BackgroundImage>,
    /// Fallback text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_text: Option<String>,
    /// Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    /// ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
    /// Select action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<AdaptiveCardAction>,
    /// Is visible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
    /// Run on separate server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_on_separate_server: Option<bool>,
}

impl AdaptiveCard {
    /// Creates a new Adaptive Card.
    pub fn new() -> Self {
        Self {
            card_type: "AdaptiveCard".to_string(),
            version: "1.6".to_string(),
            ..Default::default()
        }
    }

    /// Sets the card body content.
    pub fn with_body(mut self, body: Vec<AdaptiveCardElement>) -> Self {
        self.body = Some(body);
        self
    }

    /// Adds a single element to the card body.
    pub fn add_body_element(mut self, element: AdaptiveCardElement) -> Self {
        let mut body = self.body.unwrap_or_default();
        body.push(element);
        self.body = Some(body);
        self
    }

    /// Sets the card actions.
    pub fn with_actions(mut self, actions: Vec<AdaptiveCardAction>) -> Self {
        self.actions = Some(actions);
        self
    }

    /// Adds a single action to the card.
    pub fn add_action(mut self, action: AdaptiveCardAction) -> Self {
        let mut actions = self.actions.unwrap_or_default();
        actions.push(action);
        self.actions = Some(actions);
        self
    }

    /// Sets the card style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the horizontal alignment.
    pub fn with_horizontal_alignment(mut self, alignment: &str) -> Self {
        self.horizontal_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the fallback text.
    pub fn with_fallback_text(mut self, text: &str) -> Self {
        self.fallback_text = Some(text.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }

    /// Serializes the card to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the card to pretty-printed JSON.
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Creates an Adaptive Card from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Adaptive Card element types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdaptiveCardElement {
    /// Text element
    TextBlock(TextBlock),
    /// Image element
    Image(Image),
    /// Container element
    Container(Container),
    /// Column Set element
    ColumnSet(ColumnSet),
    /// Fact Set element
    FactSet(FactSet),
    /// Image Set element
    ImageSet(ImageSet),
    /// Input Text element
    InputText(InputText),
    /// Input Date element
    InputDate(InputDate),
    /// Input Time element
    InputTime(InputTime),
    /// Input Toggle element
    InputToggle(InputToggle),
    /// Input Choice Set element
    InputChoiceSet(InputChoiceSet),
    /// Input Number element
    InputNumber(InputNumber),
    /// Action Set element
    ActionSet(ActionSet),
    /// Spacer element
    Spacer(Spacer),
    /// Card Element
    CardElement(CardElement),
}

/// Text Block element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    /// The text content
    pub text: String,
    /// Text size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Text weight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>,
    /// Text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Is text dark?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subtle: Option<bool>,
    /// Text wrap
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    /// Horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<String>,
    /// Font type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_type: Option<String>,
    /// Max lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<u32>,
}

impl TextBlock {
    /// Creates a new Text Block element.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// Sets the text size.
    pub fn with_size(mut self, size: &str) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Sets the text weight.
    pub fn with_weight(mut self, weight: &str) -> Self {
        self.weight = Some(weight.to_string());
        self
    }

    /// Sets the text color.
    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Sets whether the text is subtle.
    pub fn with_is_subtle(mut self, is_subtle: bool) -> Self {
        self.is_subtle = Some(is_subtle);
        self
    }

    /// Sets text wrapping.
    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Sets horizontal alignment.
    pub fn with_horizontal_alignment(mut self, alignment: &str) -> Self {
        self.horizontal_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the font type.
    pub fn with_font_type(mut self, font_type: &str) -> Self {
        self.font_type = Some(font_type.to_string());
        self
    }

    /// Sets the maximum number of lines.
    pub fn with_max_lines(mut self, max_lines: u32) -> Self {
        self.max_lines = Some(max_lines);
        self
    }
}

impl Default for TextBlock {
    fn default() -> Self {
        Self {
            text: String::new(),
            size: None,
            weight: None,
            color: None,
            is_subtle: None,
            wrap: None,
            horizontal_alignment: None,
            font_type: None,
            max_lines: None,
        }
    }
}

/// Image element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// The image URL
    pub url: String,
    /// Alternative text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<String>,
    /// Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    /// Width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    /// Pixel width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_width: Option<u32>,
    /// Pixel height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_height: Option<u32>,
}

impl Image {
    /// Creates a new Image element.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..Default::default()
        }
    }

    /// Sets the alternative text.
    pub fn with_alt_text(mut self, alt_text: &str) -> Self {
        self.alt_text = Some(alt_text.to_string());
        self
    }

    /// Sets the image size.
    pub fn with_size(mut self, size: &str) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Sets the image style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets horizontal alignment.
    pub fn with_horizontal_alignment(mut self, alignment: &str) -> Self {
        self.horizontal_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the height.
    pub fn with_height(mut self, height: &str) -> Self {
        self.height = Some(height.to_string());
        self
    }

    /// Sets the width.
    pub fn with_width(mut self, width: &str) -> Self {
        self.width = Some(width.to_string());
        self
    }

    /// Sets the pixel width.
    pub fn with_pixel_width(mut self, width: u32) -> Self {
        self.pixel_width = Some(width);
        self
    }

    /// Sets the pixel height.
    pub fn with_pixel_height(mut self, height: u32) -> Self {
        self.pixel_height = Some(height);
        self
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            url: String::new(),
            alt_text: None,
            size: None,
            style: None,
            horizontal_alignment: None,
            height: None,
            width: None,
            pixel_width: None,
            pixel_height: None,
        }
    }
}

/// Container element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container items
    pub items: Vec<AdaptiveCardElement>,
    /// Select action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<AdaptiveCardAction>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
    /// Fill rule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_rule: Option<String>,
    /// Bleed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bleed: Option<bool>,
    /// Minimum height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl Container {
    /// Creates a new Container element.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            ..Default::default()
        }
    }

    /// Adds an item to the container.
    pub fn add_item(mut self, item: AdaptiveCardElement) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the select action.
    pub fn with_select_action(mut self, action: AdaptiveCardAction) -> Self {
        self.select_action = Some(action);
        self
    }

    /// Sets the container style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the fill rule.
    pub fn with_fill_rule(mut self, rule: &str) -> Self {
        self.fill_rule = Some(rule.to_string());
        self
    }

    /// Sets the bleed property.
    pub fn with_bleed(mut self, bleed: bool) -> Self {
        self.bleed = Some(bleed);
        self
    }

    /// Sets the minimum height.
    pub fn with_min_height(mut self, height: &str) -> Self {
        self.min_height = Some(height.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for Container {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            select_action: None,
            style: None,
            vertical_alignment: None,
            fill_rule: None,
            bleed: None,
            min_height: None,
            speak: None,
        }
    }
}

/// Column Set element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSet {
    /// Columns in the set
    pub columns: Vec<Column>,
    /// Select action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<AdaptiveCardAction>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Bleed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bleed: Option<bool>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl ColumnSet {
    /// Creates a new Column Set element.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            ..Default::default()
        }
    }

    /// Adds a column to the set.
    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the select action.
    pub fn with_select_action(mut self, action: AdaptiveCardAction) -> Self {
        self.select_action = Some(action);
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the bleed property.
    pub fn with_bleed(mut self, bleed: bool) -> Self {
        self.bleed = Some(bleed);
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for ColumnSet {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            select_action: None,
            style: None,
            bleed: None,
            vertical_alignment: None,
            spacing: None,
            speak: None,
        }
    }
}

/// Column in a Column Set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column items
    pub items: Vec<AdaptiveCardElement>,
    /// Select action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<AdaptiveCardAction>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    /// Bleed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bleed: Option<bool>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
    /// Min width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<String>,
    /// Max width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl Column {
    /// Creates a new Column.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            ..Default::default()
        }
    }

    /// Adds an item to the column.
    pub fn add_item(mut self, item: AdaptiveCardElement) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the select action.
    pub fn with_select_action(mut self, action: AdaptiveCardAction) -> Self {
        self.select_action = Some(action);
        self
    }

    /// Sets the column style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the column width.
    pub fn with_width(mut self, width: &str) -> Self {
        self.width = Some(width.to_string());
        self
    }

    /// Sets the bleed property.
    pub fn with_bleed(mut self, bleed: bool) -> Self {
        self.bleed = Some(bleed);
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the minimum width.
    pub fn with_min_width(mut self, width: &str) -> Self {
        self.min_width = Some(width.to_string());
        self
    }

    /// Sets the maximum width.
    pub fn with_max_width(mut self, width: &str) -> Self {
        self.max_width = Some(width.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }

    /// Sets the column items.
    pub fn with_items(mut self, items: Vec<AdaptiveCardElement>) -> Self {
        self.items = items;
        self
    }
}

impl Default for Column {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            select_action: None,
            style: None,
            width: None,
            bleed: None,
            vertical_alignment: None,
            min_width: None,
            max_width: None,
            speak: None,
        }
    }
}

/// Fact Set element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSet {
    /// Facts in the set
    pub facts: Vec<Fact>,
    /// Select action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<AdaptiveCardAction>,
    /// Show all facts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_all_facts: Option<bool>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl FactSet {
    /// Creates a new Fact Set element.
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            ..Default::default()
        }
    }

    /// Adds a fact to the set.
    pub fn add_fact(mut self, fact: Fact) -> Self {
        self.facts.push(fact);
        self
    }

    /// Sets the select action.
    pub fn with_select_action(mut self, action: AdaptiveCardAction) -> Self {
        self.select_action = Some(action);
        self
    }

    /// Sets whether to show all facts.
    pub fn with_show_all_facts(mut self, show_all: bool) -> Self {
        self.show_all_facts = Some(show_all);
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }

    /// Sets the facts in the set.
    pub fn with_facts(mut self, facts: Vec<Fact>) -> Self {
        self.facts = facts;
        self
    }
}

impl Default for FactSet {
    fn default() -> Self {
        Self {
            facts: Vec::new(),
            select_action: None,
            show_all_facts: None,
            spacing: None,
            speak: None,
        }
    }
}

/// Fact in a Fact Set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// The fact title
    pub title: String,
    /// The fact value
    pub value: String,
    /// Is the title bold?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subtle: Option<bool>,
    /// Weight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>,
    /// Horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<String>,
    /// Font type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_type: Option<String>,
}

impl Fact {
    /// Creates a new Fact.
    pub fn new(title: &str, value: &str) -> Self {
        Self {
            title: title.to_string(),
            value: value.to_string(),
            ..Default::default()
        }
    }

    /// Sets whether the title is subtle.
    pub fn with_is_subtle(mut self, is_subtle: bool) -> Self {
        self.is_subtle = Some(is_subtle);
        self
    }

    /// Sets the weight.
    pub fn with_weight(mut self, weight: &str) -> Self {
        self.weight = Some(weight.to_string());
        self
    }

    /// Sets horizontal alignment.
    pub fn with_horizontal_alignment(mut self, alignment: &str) -> Self {
        self.horizontal_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the font type.
    pub fn with_font_type(mut self, font_type: &str) -> Self {
        self.font_type = Some(font_type.to_string());
        self
    }
}

impl Default for Fact {
    fn default() -> Self {
        Self {
            title: String::new(),
            value: String::new(),
            is_subtle: None,
            weight: None,
            horizontal_alignment: None,
            font_type: None,
        }
    }
}

/// Image Set element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSet {
    /// Images in the set
    pub images: Vec<SetImage>,
    /// Image size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl ImageSet {
    /// Creates a new Image Set.
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            ..Default::default()
        }
    }

    /// Adds an image to the set.
    pub fn add_image(mut self, image: SetImage) -> Self {
        self.images.push(image);
        self
    }

    /// Sets the image size.
    pub fn with_image_size(mut self, size: &str) -> Self {
        self.image_size = Some(size.to_string());
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for ImageSet {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            image_size: None,
            spacing: None,
            speak: None,
        }
    }
}

/// Image in an Image Set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetImage {
    /// The image URL
    pub url: String,
    /// Alternative text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
}

impl SetImage {
    /// Creates a new Set Image.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            alt_text: None,
        }
    }

    /// Sets the alternative text.
    pub fn with_alt_text(mut self, alt_text: &str) -> Self {
        self.alt_text = Some(alt_text.to_string());
        self
    }
}

/// Action in an Adaptive Card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdaptiveCardAction {
    /// Action Show Card
    ShowCard(ShowCardAction),
    /// Action Submit
    Submit(SubmitAction),
    /// Action Open URL
    OpenUrl(OpenUrlAction),
    /// Action Toggle
    Toggle(ToggleAction),
    /// Action Launch Action
    LaunchAction(LaunchAction),
    /// Action Vote
    Vote(VoteAction),
}

/// Show Card Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowCardAction {
    /// Action title
    pub title: String,
    /// Action type (always "Action.ShowCard")
    #[serde(rename = "type")]
    pub action_type: String,
    /// Card to show
    pub card: Box<AdaptiveCard>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl ShowCardAction {
    /// Creates a new Show Card Action.
    pub fn new(title: &str, card: AdaptiveCard) -> Self {
        Self {
            title: title.to_string(),
            action_type: "Action.ShowCard".to_string(),
            card: Box::new(card),
            ..Default::default()
        }
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the icon URL.
    pub fn with_icon_url(mut self, url: &str) -> Self {
        self.icon_url = Some(url.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for ShowCardAction {
    fn default() -> Self {
        Self {
            title: String::new(),
            action_type: "Action.ShowCard".to_string(),
            card: Box::new(AdaptiveCard::new()),
            style: None,
            icon_url: None,
            speak: None,
        }
    }
}

/// Submit Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAction {
    /// Action title
    pub title: String,
    /// Action type (always "Action.Submit")
    #[serde(rename = "type")]
    pub action_type: String,
    /// Data to submit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
    /// Is disabled?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_disabled: Option<bool>,
    /// Fallback text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl SubmitAction {
    /// Creates a new Submit Action.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            action_type: "Action.Submit".to_string(),
            ..Default::default()
        }
    }

    /// Sets the data to submit.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the icon URL.
    pub fn with_icon_url(mut self, url: &str) -> Self {
        self.icon_url = Some(url.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }

    /// Sets whether the action is disabled.
    pub fn with_is_disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = Some(is_disabled);
        self
    }

    /// Sets the fallback text.
    pub fn with_fallback(mut self, text: &str) -> Self {
        self.fallback = Some(text.to_string());
        self
    }
}

impl Default for SubmitAction {
    fn default() -> Self {
        Self {
            title: String::new(),
            action_type: "Action.Submit".to_string(),
            data: None,
            style: None,
            icon_url: None,
            speak: None,
            is_disabled: None,
            fallback: None,
        }
    }
}

/// Open URL Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenUrlAction {
    /// Action title
    pub title: String,
    /// Action type (always "Action.OpenUrl")
    #[serde(rename = "type")]
    pub action_type: String,
    /// The URL to open
    pub url: String,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl OpenUrlAction {
    /// Creates a new Open URL Action.
    pub fn new(title: &str, url: &str) -> Self {
        Self {
            title: title.to_string(),
            action_type: "Action.OpenUrl".to_string(),
            url: url.to_string(),
            ..Default::default()
        }
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the icon URL.
    pub fn with_icon_url(mut self, url: &str) -> Self {
        self.icon_url = Some(url.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for OpenUrlAction {
    fn default() -> Self {
        Self {
            title: String::new(),
            action_type: "Action.OpenUrl".to_string(),
            url: String::new(),
            style: None,
            icon_url: None,
            speak: None,
        }
    }
}

/// Toggle Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleAction {
    /// Action title
    pub title: String,
    /// Action type (always "Action.Toggle")
    #[serde(rename = "type")]
    pub action_type: String,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Selected items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_items: Option<Vec<ToggleSelectedItems>>,
}

/// Selected items for Toggle Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleSelectedItems {
    /// Item type
    #[serde(rename = "type")]
    pub item_type: String,
    /// Item ID
    pub item_id: String,
}

impl ToggleAction {
    /// Creates a new Toggle Action.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            action_type: "Action.Toggle".to_string(),
            ..Default::default()
        }
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the selected items.
    pub fn with_selected_items(mut self, selected_items: Vec<ToggleSelectedItems>) -> Self {
        self.selected_items = Some(selected_items);
        self
    }
}

impl Default for ToggleAction {
    fn default() -> Self {
        Self {
            title: String::new(),
            action_type: "Action.Toggle".to_string(),
            style: None,
            selected_items: None,
        }
    }
}

impl Default for ToggleSelectedItems {
    fn default() -> Self {
        Self {
            item_type: "Item".to_string(),
            item_id: String::new(),
        }
    }
}

/// Launch Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchAction {
    /// Action title
    pub title: String,
    /// Action type (always "Action.LaunchAction")
    #[serde(rename = "type")]
    pub action_type: String,
    /// The URL to open
    pub url: String,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl LaunchAction {
    /// Creates a new Launch Action.
    pub fn new(title: &str, url: &str) -> Self {
        Self {
            title: title.to_string(),
            action_type: "Action.LaunchAction".to_string(),
            url: url.to_string(),
            ..Default::default()
        }
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the icon URL.
    pub fn with_icon_url(mut self, url: &str) -> Self {
        self.icon_url = Some(url.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for LaunchAction {
    fn default() -> Self {
        Self {
            title: String::new(),
            action_type: "Action.LaunchAction".to_string(),
            url: String::new(),
            style: None,
            icon_url: None,
            speak: None,
        }
    }
}

/// Vote Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteAction {
    /// Action title
    pub title: String,
    /// Action type (always "Action.Vote")
    #[serde(rename = "type")]
    pub action_type: String,
    /// Choice
    pub choice: String,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

impl VoteAction {
    /// Creates a new Vote Action.
    pub fn new(title: &str, choice: &str) -> Self {
        Self {
            title: title.to_string(),
            action_type: "Action.Vote".to_string(),
            choice: choice.to_string(),
            ..Default::default()
        }
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }
}

impl Default for VoteAction {
    fn default() -> Self {
        Self {
            title: String::new(),
            action_type: "Action.Vote".to_string(),
            choice: String::new(),
            style: None,
        }
    }
}

/// Input Text element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputText {
    /// Input ID
    pub id: String,
    /// Input type (always "Input.Text")
    #[serde(rename = "type")]
    pub input_type: String,
    /// Placeholder text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Is multiline?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multiline: Option<bool>,
    /// Max length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl InputText {
    /// Creates a new Input Text element.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            input_type: "Input.Text".to_string(),
            ..Default::default()
        }
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    /// Sets the initial value.
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }

    /// Sets whether the input is multiline.
    pub fn with_is_multiline(mut self, is_multiline: bool) -> Self {
        self.is_multiline = Some(is_multiline);
        self
    }

    /// Sets the maximum length.
    pub fn with_max_length(mut self, max_length: u32) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for InputText {
    fn default() -> Self {
        Self {
            id: String::new(),
            input_type: "Input.Text".to_string(),
            placeholder: None,
            value: None,
            is_multiline: None,
            max_length: None,
            style: None,
            vertical_alignment: None,
            spacing: None,
            speak: None,
        }
    }
}

/// Input Date element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDate {
    /// Input ID
    pub id: String,
    /// Input type (always "Input.Date")
    #[serde(rename = "type")]
    pub input_type: String,
    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Minimum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,
    /// Maximum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl InputDate {
    /// Creates a new Input Date element.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            input_type: "Input.Date".to_string(),
            ..Default::default()
        }
    }

    /// Sets the initial value.
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }

    /// Sets the minimum value.
    pub fn with_min(mut self, min: &str) -> Self {
        self.min = Some(min.to_string());
        self
    }

    /// Sets the maximum value.
    pub fn with_max(mut self, max: &str) -> Self {
        self.max = Some(max.to_string());
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for InputDate {
    fn default() -> Self {
        Self {
            id: String::new(),
            input_type: "Input.Date".to_string(),
            value: None,
            min: None,
            max: None,
            style: None,
            vertical_alignment: None,
            spacing: None,
            speak: None,
        }
    }
}

/// Input Toggle element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputToggle {
    /// Input ID
    pub id: String,
    /// Input type (always "Input.Toggle")
    #[serde(rename = "type")]
    pub input_type: String,
    /// Title
    pub title: String,
    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<bool>,
    /// Value on (for switch style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_on: Option<String>,
    /// Value off (for switch style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_off: Option<String>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    /// Speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
}

impl InputToggle {
    /// Creates a new Input Toggle element.
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            input_type: "Input.Toggle".to_string(),
            title: title.to_string(),
            ..Default::default()
        }
    }

    /// Sets the initial value.
    pub fn with_value(mut self, value: bool) -> Self {
        self.value = Some(value);
        self
    }

    /// Sets the value on.
    pub fn with_value_on(mut self, value_on: &str) -> Self {
        self.value_on = Some(value_on.to_string());
        self
    }

    /// Sets the value off.
    pub fn with_value_off(mut self, value_off: &str) -> Self {
        self.value_off = Some(value_off.to_string());
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }

    /// Sets the speak content.
    pub fn with_speak(mut self, text: &str) -> Self {
        self.speak = Some(text.to_string());
        self
    }
}

impl Default for InputToggle {
    fn default() -> Self {
        Self {
            id: String::new(),
            input_type: "Input.Toggle".to_string(),
            title: String::new(),
            value: None,
            value_on: None,
            value_off: None,
            style: None,
            vertical_alignment: None,
            spacing: None,
            speak: None,
        }
    }
}

/// Background image for Adaptive Cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// The image URL
    pub url: String,
    /// Fill mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_mode: Option<String>,
    /// Horizontal alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<String>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
}

impl BackgroundImage {
    /// Creates a new Background Image.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..Default::default()
        }
    }

    /// Sets the fill mode.
    pub fn with_fill_mode(mut self, fill_mode: &str) -> Self {
        self.fill_mode = Some(fill_mode.to_string());
        self
    }

    /// Sets the horizontal alignment.
    pub fn with_horizontal_alignment(mut self, alignment: &str) -> Self {
        self.horizontal_alignment = Some(alignment.to_string());
        self
    }

    /// Sets the vertical alignment.
    pub fn with_vertical_alignment(mut self, alignment: &str) -> Self {
        self.vertical_alignment = Some(alignment.to_string());
        self
    }
}

impl Default for BackgroundImage {
    fn default() -> Self {
        Self {
            url: String::new(),
            fill_mode: None,
            horizontal_alignment: None,
            vertical_alignment: None,
        }
    }
}

/// Spacer element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacer {
    /// Spacer ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Spacer type (always "Spacer")
    #[serde(rename = "type")]
    pub spacer_type: String,
    /// Size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
}

impl Spacer {
    /// Creates a new Spacer element.
    pub fn new() -> Self {
        Self {
            spacer_type: "Spacer".to_string(),
            ..Default::default()
        }
    }

    /// Sets the spacer ID.
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Sets the size.
    pub fn with_size(mut self, size: &str) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Sets the spacing.
    pub fn with_spacing(mut self, spacing: &str) -> Self {
        self.spacing = Some(spacing.to_string());
        self
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self {
            id: None,
            spacer_type: "Spacer".to_string(),
            size: None,
            spacing: None,
        }
    }
}

/// Card Element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardElement {
    /// Card element type
    #[serde(rename = "type")]
    pub card_type: String,
    /// Items
    pub items: Vec<AdaptiveCardElement>,
}

impl CardElement {
    /// Creates a new Card Element.
    pub fn new() -> Self {
        Self {
            card_type: "CardElement".to_string(),
            items: Vec::new(),
        }
    }

    /// Adds an item to the card element.
    pub fn add_item(mut self, item: AdaptiveCardElement) -> Self {
        self.items.push(item);
        self
    }
}

/// Input Time element.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputTime {
    /// Input ID
    pub id: String,
    /// Input type (always "Input.Time")
    #[serde(rename = "type")]
    pub input_type: String,
    /// Initial value (HH:MM:SS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Placeholder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

impl InputTime {
    /// Creates a new Input Time element.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            input_type: "Input.Time".to_string(),
            ..Default::default()
        }
    }
}

/// Input Choice Set element.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputChoiceSet {
    /// Input ID
    pub id: String,
    /// Input type (always "Input.ChoiceSet")
    #[serde(rename = "type")]
    pub input_type: String,
    /// Choices
    pub choices: Vec<Choice>,
    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Is multi-select
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multi_select: Option<bool>,
    /// Label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Placeholder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Choice title
    pub title: String,
    /// Choice value
    pub value: String,
}

impl InputChoiceSet {
    /// Creates a new Input Choice Set element.
    pub fn new(id: &str, choices: Vec<Choice>) -> Self {
        Self {
            id: id.to_string(),
            input_type: "Input.ChoiceSet".to_string(),
            choices,
            ..Default::default()
        }
    }
}

/// Input Number element.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputNumber {
    /// Input ID
    pub id: String,
    /// Input type (always "Input.Number")
    #[serde(rename = "type")]
    pub input_type: String,
    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Minimum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Placeholder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

impl InputNumber {
    /// Creates a new Input Number element.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            input_type: "Input.Number".to_string(),
            ..Default::default()
        }
    }
}

/// Action Set element.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionSet {
    /// Actions
    pub actions: Vec<AdaptiveCardAction>,
    /// Style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Vertical alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_alignment: Option<String>,
}

/// Helper functions for creating common Adaptive Card patterns.
pub mod helpers {
    use super::*;

    /// Creates a simple message card with text.
    pub fn create_message_card(title: &str, message: &str) -> AdaptiveCard {
        AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(title)
                        .with_weight("bolder")
                        .with_size("large")
                        .with_color("accent")
                ),
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(message).with_wrap(true)
                ),
            ])
    }

    /// Creates a card with action buttons.
    pub fn create_action_card(title: &str, message: &str) -> AdaptiveCard {
        let card = AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(title)
                        .with_weight("bolder")
                        .with_size("large")
                        .with_color("accent")
                ),
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(message).with_wrap(true)
                ),
                AdaptiveCardElement::Spacer(
                    Spacer::new().with_size("medium")
                ),
            ])
            .with_actions(vec![
                AdaptiveCardAction::Submit(
                    SubmitAction::new("OK")
                        .with_style("positive")
                        .with_data(serde_json::json!({"action": "ok"}))
                ),
                AdaptiveCardAction::Submit(
                    SubmitAction::new("Cancel")
                        .with_style("destructive")
                        .with_data(serde_json::json!({"action": "cancel"}))
                ),
            ]);

        card
    }

    /// Creates a card with input fields.
    pub fn create_form_card(title: &str, fields: Vec<InputText>) -> AdaptiveCard {
        let mut body = vec![
            AdaptiveCardElement::TextBlock(
                TextBlock::new(title)
                    .with_weight("bolder")
                    .with_size("medium")
                    .with_color("accent")
            ),
            AdaptiveCardElement::Spacer(
                Spacer::new().with_size("small")
            ),
        ];

        for field in fields {
            body.push(AdaptiveCardElement::InputText(field));
        }

        body.push(AdaptiveCardElement::Spacer(
            Spacer::new().with_size("medium")
        ));

        AdaptiveCard::new()
            .with_body(body)
            .with_actions(vec![
                AdaptiveCardAction::Submit(
                    SubmitAction::new("Submit")
                        .with_style("positive")
                        .with_data(serde_json::json!({"form": "submitted"}))
                ),
            ])
    }

    /// Creates a card with an image.
    pub fn create_image_card(title: &str, image_url: &str, caption: &str) -> AdaptiveCard {
        AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(title)
                        .with_weight("bolder")
                        .with_size("large")
                        .with_color("accent")
                ),
                AdaptiveCardElement::Image(
                    Image::new(image_url).with_alt_text(caption)
                ),
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(caption).with_wrap(true)
                ),
            ])
    }

    /// Creates a card with facts.
    pub fn create_fact_card(title: &str, facts: Vec<Fact>) -> AdaptiveCard {
        AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(title)
                        .with_weight("bolder")
                        .with_size("large")
                        .with_color("accent")
                ),
                AdaptiveCardElement::FactSet(
                    FactSet::new().with_facts(facts)
                ),
            ])
    }

    /// Creates a card with a column layout.
    pub fn create_column_card(
        title: &str,
        left_content: Vec<AdaptiveCardElement>,
        right_content: Vec<AdaptiveCardElement>,
    ) -> AdaptiveCard {
        let column_set = ColumnSet::new()
            .add_column(
                Column::new()
                    .with_width("200px")
                    .with_items(left_content),
            )
            .add_column(Column::new().with_items(right_content));

        AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(
                    TextBlock::new(title)
                        .with_weight("bolder")
                        .with_size("large")
                        .with_color("accent")
                ),
                AdaptiveCardElement::ColumnSet(column_set),
            ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_card_creation() {
        let card = AdaptiveCard::new();
        assert_eq!(card.card_type, "AdaptiveCard");
        assert_eq!(card.version, "1.6");
    }

    #[test]
    fn test_adaptive_card_with_body() {
        let card = AdaptiveCard::new()
            .with_body(vec![
                AdaptiveCardElement::TextBlock(TextBlock::new("Hello").with_weight("bolder")),
                AdaptiveCardElement::TextBlock(TextBlock::new("World").with_wrap(true)),
            ]);

        let body = card.body.unwrap();
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn test_text_block_creation() {
        let block = TextBlock::new("Test Text")
            .with_size("large")
            .with_weight("bolder")
            .with_color("accent");

        assert_eq!(block.text, "Test Text");
        assert_eq!(block.size, Some("large".to_string()));
        assert_eq!(block.weight, Some("bolder".to_string()));
        assert_eq!(block.color, Some("accent".to_string()));
    }

    #[test]
    fn test_image_creation() {
        let image = Image::new("https://example.com/image.png")
            .with_alt_text("Test Image")
            .with_size("medium");

        assert_eq!(image.url, "https://example.com/image.png");
        assert_eq!(image.alt_text, Some("Test Image".to_string()));
        assert_eq!(image.size, Some("medium".to_string()));
    }

    #[test]
    fn test_container_creation() {
        let container = Container::new()
            .add_item(AdaptiveCardElement::TextBlock(TextBlock::new("Item 1")))
            .add_item(AdaptiveCardElement::TextBlock(TextBlock::new("Item 2")));

        assert_eq!(container.items.len(), 2);
    }

    #[test]
    fn test_submit_action_creation() {
        let action = SubmitAction::new("Submit")
            .with_style("positive")
            .with_data(serde_json::json!({"action": "submit"}));

        assert_eq!(action.title, "Submit");
        assert_eq!(action.action_type, "Action.Submit");
        assert_eq!(action.style, Some("positive".to_string()));
        assert!(action.data.is_some());
    }

    #[test]
    fn test_open_url_action_creation() {
        let action = OpenUrlAction::new("View Website", "https://example.com");

        assert_eq!(action.title, "View Website");
        assert_eq!(action.url, "https://example.com");
        assert_eq!(action.action_type, "Action.OpenUrl");
    }

    #[test]
    fn test_adaptive_card_serialization() {
        let card = AdaptiveCard::new()
            .with_body(vec![AdaptiveCardElement::TextBlock(TextBlock::new("Hello World"))]);

        let json = card.to_json().unwrap();
        assert!(json.contains("AdaptiveCard"));
        assert!(json.contains("Hello World"));
    }

    #[test]
    fn test_adaptive_card_deserialization() {
        let json = r#"{
            "type": "AdaptiveCard",
            "version": "1.6",
            "body": [
                {
                    "type": "TextBlock",
                    "text": "Hello World",
                    "size": "large"
                }
            ]
        }"#;

        let card = AdaptiveCard::from_json(json).unwrap();
        assert_eq!(card.card_type, "AdaptiveCard");
        assert_eq!(card.version, "1.6");
        assert!(card.body.is_some());
    }

    #[test]
    fn test_helper_create_message_card() {
        let card = helpers::create_message_card("Project Update", "All tasks completed!");
        let body = card.body.unwrap();

        assert_eq!(body.len(), 2);
        if let AdaptiveCardElement::TextBlock(block) = &body[0] {
            assert_eq!(block.text, "Project Update");
            assert_eq!(block.weight, Some("bolder".to_string()));
        }
    }

    #[test]
    fn test_helper_create_action_card() {
        let card = helpers::create_action_card("Confirm Action", "Are you sure?");
        let body = card.body.unwrap();

        assert_eq!(body.len(), 3);
        assert!(card.actions.is_some());

        let actions = card.actions.unwrap();
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_helper_create_form_card() {
        let fields = vec![
            InputText::new("name").with_placeholder("Enter your name"),
            InputText::new("email").with_placeholder("Enter your email"),
        ];

        let card = helpers::create_form_card("Registration Form", fields);
        let body = card.body.unwrap();

        // Should have title, spacer, 2 inputs, spacer, and actions
        assert!(body.len() >= 5);
        assert!(card.actions.is_some());
    }

    #[test]
    fn test_helper_create_image_card() {
        let card = helpers::create_image_card(
            "Team Photo",
            "https://example.com/team.jpg",
            "Our amazing team",
        );

        let body = card.body.unwrap();
        assert_eq!(body.len(), 3);
    }

    #[test]
    fn test_helper_create_fact_card() {
        let facts = vec![
            Fact::new("Name", "John Doe"),
            Fact::new("Email", "john@example.com"),
            Fact::new("Role", "Developer"),
        ];

        let card = helpers::create_fact_card("User Information", facts);

        let body = card.body.unwrap();
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn test_helper_create_column_card() {
        let card = helpers::create_column_card(
            "Dashboard",
            vec![AdaptiveCardElement::TextBlock(TextBlock::new("Left Panel"))],
            vec![AdaptiveCardElement::TextBlock(TextBlock::new("Right Panel"))],
        );

        let body = card.body.unwrap();
        assert_eq!(body.len(), 2);
    }
}
