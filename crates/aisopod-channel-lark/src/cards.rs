//! Rich message cards for Lark/Feishu.
//!
//! This module provides types for constructing rich interactive message cards
//! that can be sent through the Lark/Feishu API.

use serde::{Deserialize, Serialize};

/// A rich interactive message card.
///
/// Message cards are used to display rich content in Lark chats,
/// including titles, text, images, buttons, and more.
#[derive(Debug, Serialize, Clone)]
pub struct MessageCard {
    /// Card configuration
    pub config: CardConfig,
    /// Card header
    pub header: CardHeader,
    /// Card content elements
    pub elements: Vec<CardElement>,
}

/// Card configuration options.
#[derive(Debug, Serialize, Clone)]
pub struct CardConfig {
    /// Whether to enable wide screen mode for the card
    pub wide_screen_mode: bool,
}

/// Card header with title and optional color template.
#[derive(Debug, Serialize, Clone)]
pub struct CardHeader {
    /// The card title
    pub title: CardText,
    /// Optional color template: "blue", "green", "red", "orange", "purple"
    pub template: Option<String>,
}

/// Text content for card elements.
#[derive(Debug, Serialize, Clone)]
pub struct CardText {
    /// Text type: "plain_text" or "lark_md"
    pub tag: String,
    /// The text content
    pub content: String,
}

/// Card content elements.
#[derive(Debug, Serialize, Clone)]
pub enum CardElement {
    /// A div element containing text
    Div {
        #[serde(rename = "tag")]
        tag_type: String,
        text: CardText,
        #[serde(skip_serializing_if = "Option::is_none")]
        extra: Option<CardExtra>,
    },
    /// A markdown element
    Markdown {
        #[serde(rename = "tag")]
        tag_type: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        extra: Option<CardExtra>,
    },
    /// An image element
    Image {
        #[serde(rename = "tag")]
        tag_type: String,
        img_key: String,
        alt: CardText,
        #[serde(skip_serializing_if = "Option::is_none")]
        extra: Option<CardExtra>,
    },
    /// A button element
    Button {
        #[serde(rename = "tag")]
        tag_type: String,
        text: CardText,
        #[serde(skip_serializing_if = "Option::is_none")]
        type_: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        variant: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        extra: Option<CardExtra>,
    },
}

/// Extra configuration for card elements.
#[derive(Debug, Serialize, Clone)]
pub struct CardExtra {
    #[serde(rename = "type")]
    pub element_type: String,
}

impl MessageCard {
    /// Creates a simple message card with a title and body.
    ///
    /// # Arguments
    ///
    /// * `title` - The card title
    /// * `body` - The main body content (markdown format)
    pub fn simple(title: &str, body: &str) -> Self {
        Self {
            config: CardConfig {
                wide_screen_mode: true,
            },
            header: CardHeader {
                title: CardText {
                    tag: "plain_text".to_string(),
                    content: title.to_string(),
                },
                template: Some("blue".to_string()),
            },
            elements: vec![CardElement::Markdown {
                tag_type: "markdown".to_string(),
                content: body.to_string(),
                extra: None,
            }],
        }
    }

    /// Creates a message card with a div element.
    ///
    /// # Arguments
    ///
    /// * `title` - The card title
    /// * `body` - The body text content
    pub fn with_div(title: &str, body: &str) -> Self {
        Self {
            config: CardConfig {
                wide_screen_mode: true,
            },
            header: CardHeader {
                title: CardText {
                    tag: "plain_text".to_string(),
                    content: title.to_string(),
                },
                template: Some("green".to_string()),
            },
            elements: vec![CardElement::Div {
                tag_type: "div".to_string(),
                text: CardText {
                    tag: "lark_md".to_string(),
                    content: body.to_string(),
                },
                extra: None,
            }],
        }
    }

    /// Creates a message card with an image.
    ///
    /// # Arguments
    ///
    /// * `title` - The card title
    /// * `img_key` - The image key from Lark upload API
    /// * `body` - Optional body text
    pub fn with_image(title: &str, img_key: &str, body: Option<&str>) -> Self {
        let mut elements = vec![CardElement::Image {
            tag_type: "image".to_string(),
            img_key: img_key.to_string(),
            alt: CardText {
                tag: "plain_text".to_string(),
                content: title.to_string(),
            },
            extra: None,
        }];

        if let Some(body_text) = body {
            elements.push(CardElement::Markdown {
                tag_type: "markdown".to_string(),
                content: body_text.to_string(),
                extra: None,
            });
        }

        Self {
            config: CardConfig {
                wide_screen_mode: true,
            },
            header: CardHeader {
                title: CardText {
                    tag: "plain_text".to_string(),
                    content: title.to_string(),
                },
                template: Some("orange".to_string()),
            },
            elements,
        }
    }

    /// Converts the message card to JSON.
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_card() {
        let card = MessageCard::simple("Test Title", "This is a **test** card.");
        assert_eq!(card.config.wide_screen_mode, true);
        assert_eq!(card.header.template, Some("blue".to_string()));
        assert_eq!(card.header.title.tag, "plain_text");
        assert_eq!(card.header.title.content, "Test Title");
        assert_eq!(card.elements.len(), 1);
    }

    #[test]
    fn test_div_card() {
        let card = MessageCard::with_div("Test Title", "This is a *test* card.");
        assert_eq!(card.header.template, Some("green".to_string()));
    }

    #[test]
    fn test_image_card() {
        let card =
            MessageCard::with_image("Test Title", "img_v3_02abc123", Some("Image description"));
        assert_eq!(card.header.template, Some("orange".to_string()));
        assert_eq!(card.elements.len(), 2);
    }

    #[test]
    fn test_card_to_json() {
        let card = MessageCard::simple("Test", "Hello **World**");
        let json = card.to_json().unwrap();

        assert_eq!(json["config"]["wide_screen_mode"], true);
        assert_eq!(json["header"]["title"]["content"], "Test");
        assert_eq!(json["header"]["template"], "blue");
    }
}
