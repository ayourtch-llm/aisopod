//! Google Chat card-based message builder.
//!
//! This module provides a builder API for constructing rich card-based messages
//! for Google Chat, following the Google Chat card format.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Builder for Google Chat cards.
#[derive(Debug, Clone, Default)]
pub struct CardBuilder {
    /// Card header
    header: Option<CardHeader>,
    /// Card sections
    sections: Vec<CardSection>,
    /// Card actions
    actions: Vec<CardAction>,
}

impl CardBuilder {
    /// Create a new card builder.
    pub fn new() -> Self {
        Self {
            header: None,
            sections: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Set the card header.
    pub fn header(mut self, header: CardHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Add a section to the card.
    pub fn section(mut self, section: CardSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add multiple sections to the card.
    pub fn sections(mut self, sections: Vec<CardSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add an action to the card.
    pub fn action(mut self, action: CardAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Add multiple actions to the card.
    pub fn actions(mut self, actions: Vec<CardAction>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Build the card into a JSON value.
    pub fn build(self) -> Value {
        let mut card = Map::new();

        // Add header if present
        if let Some(header) = self.header {
            card.insert("header".to_string(), json!(header));
        }

        // Add sections if present
        if !self.sections.is_empty() {
            let sections: Vec<Value> = self.sections.into_iter().map(|s| json!(s)).collect();
            card.insert("sections".to_string(), json!(sections));
        }

        // Add actions if present
        if !self.actions.is_empty() {
            let actions: Vec<Value> = self.actions.into_iter().map(|a| json!(a)).collect();
            card.insert("action".to_string(), json!(actions));
        }

        Value::Object(card)
    }
}

/// Card header with title, subtitle, and image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardHeader {
    /// Title of the card.
    pub title: String,
    /// Subtitle of the card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Image URL or photo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<CardImage>,
    /// Image style.
    #[serde(rename = "imageStyle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_style: Option<ImageStyle>,
}

impl CardHeader {
    /// Create a new card header with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            image: None,
            image_style: None,
        }
    }

    /// Set the subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the image.
    pub fn image(mut self, image: CardImage) -> Self {
        self.image = Some(image);
        self
    }

    /// Set the image style.
    pub fn image_style(mut self, image_style: ImageStyle) -> Self {
        self.image_style = Some(image_style);
        self
    }
}

/// Image for the card header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardImage {
    /// Image URL.
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    /// Optional action when image is clicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ImageAction>,
}

impl CardImage {
    /// Create a new card image with the given URL.
    pub fn new(image_url: impl Into<String>) -> Self {
        Self {
            image_url: image_url.into(),
            action: None,
        }
    }

    /// Set the action when image is clicked.
    pub fn action(mut self, action: ImageAction) -> Self {
        self.action = Some(action);
        self
    }
}

/// Image action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAction {
    /// Action URL.
    pub url: String,
}

impl ImageAction {
    /// Create a new image action with the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// Image style for card header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageStyle {
    /// Circle image style.
    Circle,
    /// Square image style.
    Square,
}

/// Card section with widgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSection {
    /// Section header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Widgets in the section.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub widgets: Vec<Widget>,
}

impl CardSection {
    /// Create a new card section.
    pub fn new() -> Self {
        Self {
            header: None,
            widgets: Vec::new(),
        }
    }

    /// Set the section header.
    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Add a widget to the section.
    pub fn widget(mut self, widget: Widget) -> Self {
        self.widgets.push(widget);
        self
    }

    /// Add multiple widgets to the section.
    pub fn widgets(mut self, widgets: Vec<Widget>) -> Self {
        self.widgets.extend(widgets);
        self
    }
}

/// Widget in a card section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Widget {
    /// Text paragraph widget.
    #[serde(rename = "textParagraph")]
    TextParagraph(TextParagraph),
    /// Image widget.
    #[serde(rename = "image")]
    Image(ImageWidget),
    /// Button widget.
    #[serde(rename = "button")]
    Button(ButtonWidget),
    /// Button list widget.
    #[serde(rename = "buttonList")]
    ButtonList(ButtonList),
    /// Divider widget.
    #[serde(rename = "divider")]
    Divider(Divider),
    /// Grid widget.
    #[serde(rename = "grid")]
    Grid(GridWidget),
    /// Pickers widget.
    #[serde(rename = "pickers")]
    Pickers(Vec<PickersItem>),
    /// Selection input widget.
    #[serde(rename = "selectionInput")]
    SelectionInput(SelectionInputWidget),
    /// Decorated text widget.
    #[serde(rename = "decoratedText")]
    DecoratedText(DecoratedText),
}

/// Text paragraph widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextParagraph {
    /// Text content.
    pub text: String,
    /// Optional format for the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextFormat>,
}

impl TextParagraph {
    /// Create a new text paragraph with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            format: None,
        }
    }

    /// Set the text format.
    pub fn format(mut self, format: TextFormat) -> Self {
        self.format = Some(format);
        self
    }
}

/// Text format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextFormat {
    /// Plain text format.
    Plain,
    /// HTML format.
    Html,
}

/// Image widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageWidget {
    /// Image URL.
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    /// Optional action when image is clicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ImageAction>,
    /// Image alt text.
    #[serde(rename = "altText")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// Image layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ImageLayout>,
}

impl ImageWidget {
    /// Create a new image widget with the given URL.
    pub fn new(image_url: impl Into<String>) -> Self {
        Self {
            image_url: image_url.into(),
            action: None,
            alt_text: None,
            layout: None,
        }
    }

    /// Set the action when image is clicked.
    pub fn action(mut self, action: ImageAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Set the alt text.
    pub fn alt_text(mut self, alt_text: impl Into<String>) -> Self {
        self.alt_text = Some(alt_text.into());
        self
    }

    /// Set the layout.
    pub fn layout(mut self, layout: ImageLayout) -> Self {
        self.layout = Some(layout);
        self
    }
}

/// Image layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageLayout {
    /// Fill layout.
    Fill,
    /// Fit layout.
    Fit,
}

/// Button widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonWidget {
    /// Text on the button.
    pub text: String,
    /// Optional icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
    /// Optional icon URL.
    #[serde(rename = "iconUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Optional icon style.
    #[serde(rename = "iconStyle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_style: Option<IconStyle>,
    /// Optional background color.
    #[serde(rename = "backgroundColor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Optional on-click action.
    #[serde(rename = "onClick")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_click: Option<OnClick>,
    /// Optional bottom padding.
    #[serde(rename = "bottomPadding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom_padding: Option<bool>,
}

impl ButtonWidget {
    /// Create a new button widget with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            icon_url: None,
            icon_style: None,
            background_color: None,
            on_click: None,
            bottom_padding: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the icon URL.
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the icon style.
    pub fn icon_style(mut self, icon_style: IconStyle) -> Self {
        self.icon_style = Some(icon_style);
        self
    }

    /// Set the background color.
    pub fn background_color(mut self, background_color: impl Into<String>) -> Self {
        self.background_color = Some(background_color.into());
        self
    }

    /// Set the on-click action.
    pub fn on_click(mut self, on_click: OnClick) -> Self {
        self.on_click = Some(on_click);
        self
    }

    /// Set bottom padding.
    pub fn bottom_padding(mut self, bottom_padding: bool) -> Self {
        self.bottom_padding = Some(bottom_padding);
        self
    }
}

/// Icon for button.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Icon {
    /// Built-in icon name.
    #[serde(rename = "STAR")]
    Star,
    #[serde(rename = "STAR_HALF")]
    StarHalf,
    #[serde(rename = "STAR_BORDER")]
    StarBorder,
    #[serde(rename = "HEART")]
    Heart,
    #[serde(rename = "HEART_BROKEN")]
    HeartBroken,
    #[serde(rename = "THUMBS_UP")]
    ThumbsUp,
    #[serde(rename = "THUMBS_DOWN")]
    ThumbsDown,
    #[serde(rename = "CALL")]
    Call,
    #[serde(rename = "EMAIL")]
    Email,
    #[serde(rename = "MESSAGE")]
    Message,
    #[serde(rename = "CHAT")]
    Chat,
    #[serde(rename = "PHONE")]
    Phone,
    #[serde(rename = "FOLDER")]
    Folder,
    #[serde(rename = "OPEN_FILE")]
    OpenFile,
    #[serde(rename = "PUBLIC")]
    Public,
    #[serde(rename = "CLOUD")]
    Cloud,
    #[serde(rename = "CLOCK")]
    Clock,
    #[serde(rename = "MORE")]
    More,
    #[serde(rename = "INFO")]
    Info,
    #[serde(rename = "WARNING")]
    Warning,
    #[serde(rename = "SETTINGS")]
    Settings,
    #[serde(rename = "HELP")]
    Help,
    #[serde(rename = "DONE")]
    Done,
    #[serde(rename = "DONE_ALL")]
    DoneAll,
    #[serde(rename = "NOTIFICATION")]
    Notification,
    #[serde(rename = "BOOKMARK")]
    Bookmark,
    #[serde(rename = "BOOKMARK_BORDER")]
    BookmarkBorder,
    #[serde(rename = "ADD")]
    Add,
    #[serde(rename = "CLOSE")]
    Close,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "EDIT")]
    Edit,
    #[serde(rename = "FILTER")]
    Filter,
    #[serde(rename = "SEARCH")]
    Search,
    #[serde(rename = "SHARE")]
    Share,
    #[serde(rename = "UPLOAD")]
    Upload,
    #[serde(rename = "DOWNLOAD")]
    Download,
    #[serde(rename = "LINK")]
    Link,
    #[serde(rename = "MENU")]
    Menu,
    #[serde(rename = "SORT")]
    Sort,
    #[serde(rename = "CHECK_BOX")]
    CheckBox,
    #[serde(rename = "CHECK_BOX_OUTLINE_BLANK")]
    CheckBoxOutlineBlank,
    #[serde(rename = "RADIO_BUTTON_ON")]
    RadioButtonOn,
    #[serde(rename = "RADIO_BUTTON_OFF")]
    RadioButtonOff,
    #[serde(rename = "EXPAND_LESS")]
    ExpandLess,
    #[serde(rename = "EXPAND_MORE")]
    ExpandMore,
    #[serde(rename = "ARROW_LEFT")]
    ArrowLeft,
    #[serde(rename = "ARROW_RIGHT")]
    ArrowRight,
    #[serde(rename = "ARROW_UP")]
    ArrowUp,
    #[serde(rename = "ARROW_DOWN")]
    ArrowDown,
    #[serde(rename = "CANCEL")]
    Cancel,
    #[serde(rename = "CREATE")]
    Create,
    #[serde(rename = "DASHBOARD")]
    Dashboard,
    #[serde(rename = "FAVORITES")]
    Favorites,
    #[serde(rename = "HOME")]
    Home,
    #[serde(rename = "NOTIFICATIONS")]
    Notifications,
    #[serde(rename = "PERSON")]
    Person,
    #[serde(rename = "PERSON_ADD")]
    PersonAdd,
    #[serde(rename = "SETTINGS_BACKUP_RESTORE")]
    SettingsBackupRestore,
    #[serde(rename = "SYSTEM_ALERT")]
    SystemAlert,
    #[serde(rename = "VpnKey")]
    VpnKey,
}

/// Icon style for button.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IconStyle {
    /// Primary icon style.
    Primary,
    /// Neutral icon style.
    Neutral,
    /// Warning icon style.
    Warning,
    /// Error icon style.
    Error,
}

/// On-click action for button.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum OnClick {
    /// Open a URL.
    #[serde(rename = "openLink")]
    OpenLink(OpenLink),
    /// Show an alert.
    #[serde(rename = "action")]
    Action(Action),
    /// Run an action with parameters.
    #[serde(rename = "action")]
    RunAction(RunAction),
}

/// Open URL action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLink {
    /// URL to open.
    pub url: String,
}

impl OpenLink {
    /// Create a new open link action.
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// Action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action name.
    pub name: String,
}

impl Action {
    /// Create a new action.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Run action with parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAction {
    /// Action name.
    pub name: String,
}

impl RunAction {
    /// Create a new run action.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Button list widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonList {
    /// Buttons in the list.
    pub buttons: Vec<ButtonWidget>,
}

impl ButtonList {
    /// Create a new button list.
    pub fn new() -> Self {
        Self { buttons: Vec::new() }
    }

    /// Add a button to the list.
    pub fn button(mut self, button: ButtonWidget) -> Self {
        self.buttons.push(button);
        self
    }

    /// Add multiple buttons to the list.
    pub fn buttons(mut self, buttons: Vec<ButtonWidget>) -> Self {
        self.buttons.extend(buttons);
        self
    }
}

/// Divider widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divider {
    /// Whether the divider has top margin.
    #[serde(rename = "splitDivider")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_divider: Option<bool>,
    /// Whether the divider has top margin.
    #[serde(rename = "topMargin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_margin: Option<bool>,
}

impl Divider {
    /// Create a new divider.
    pub fn new() -> Self {
        Self {
            split_divider: None,
            top_margin: None,
        }
    }

    /// Set split divider.
    pub fn split_divider(mut self, split_divider: bool) -> Self {
        self.split_divider = Some(split_divider);
        self
    }

    /// Set top margin.
    pub fn top_margin(mut self, top_margin: bool) -> Self {
        self.top_margin = Some(top_margin);
        self
    }
}

/// Grid widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridWidget {
    /// Items in the grid.
    pub items: Vec<GridItem>,
}

impl GridWidget {
    /// Create a new grid widget.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an item to the grid.
    pub fn item(mut self, item: GridItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items to the grid.
    pub fn items(mut self, items: Vec<GridItem>) -> Self {
        self.items.extend(items);
        self
    }
}

/// Grid item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridItem {
    /// Text to display.
    pub text: String,
    /// Optional icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
    /// Optional on-click action.
    #[serde(rename = "onClick")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_click: Option<OnClick>,
}

impl GridItem {
    /// Create a new grid item.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            on_click: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the on-click action.
    pub fn on_click(mut self, on_click: OnClick) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

/// Pickers widget item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickersItem {
    /// Picker type.
    #[serde(rename = "pickerType")]
    pub picker_type: PickerType,
    /// Item ID.
    #[serde(rename = "itemId")]
    pub item_id: String,
    /// Display text.
    pub text: String,
    /// Selected state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
}

impl PickersItem {
    /// Create a new pickers item.
    pub fn new(picker_type: PickerType, item_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            picker_type,
            item_id: item_id.into(),
            text: text.into(),
            selected: None,
        }
    }

    /// Set the selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }
}

/// Picker type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PickerType {
    /// Date picker.
    Date,
    /// Time picker.
    Time,
    /// Date and time picker.
    DateTime,
}

/// Selection input widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionInputWidget {
    /// Item ID.
    #[serde(rename = "itemId")]
    pub item_id: String,
    /// Text to display.
    pub text: String,
    /// Selection type.
    #[serde(rename = "selectionType")]
    pub selection_type: SelectionType,
    /// Items to select from.
    pub items: Vec<SelectionItem>,
    /// Whether multiple items can be selected.
    #[serde(rename = "multiSelect")]
    pub multi_select: Option<bool>,
    /// Initial selected items.
    #[serde(rename = "initialSelectedItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_selected_items: Option<Vec<String>>,
    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

impl SelectionInputWidget {
    /// Create a new selection input widget.
    pub fn new(item_id: impl Into<String>, text: impl Into<String>, selection_type: SelectionType) -> Self {
        Self {
            item_id: item_id.into(),
            text: text.into(),
            selection_type,
            items: Vec::new(),
            multi_select: None,
            initial_selected_items: None,
            placeholder: None,
        }
    }

    /// Add an item to the selection.
    pub fn item(mut self, item: SelectionItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items to the selection.
    pub fn items(mut self, items: Vec<SelectionItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Set multi-select.
    pub fn multi_select(mut self, multi_select: bool) -> Self {
        self.multi_select = Some(multi_select);
        self
    }

    /// Set initial selected items.
    pub fn initial_selected_items(mut self, items: Vec<String>) -> Self {
        self.initial_selected_items = Some(items);
        self
    }

    /// Set placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
}

/// Selection item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionItem {
    /// Item ID.
    #[serde(rename = "itemId")]
    pub item_id: String,
    /// Text to display.
    #[serde(rename = "text")]
    pub text: String,
    /// Optional icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
}

impl SelectionItem {
    /// Create a new selection item.
    pub fn new(item_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            text: text.into(),
            icon: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// Selection type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SelectionType {
    /// Radio button selection.
    RadioButton,
    /// Checkbox selection.
    Checkbox,
}

/// Decorated text widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoratedText {
    /// Optional start icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_icon: Option<Icon>,
    /// Optional start icon URL.
    #[serde(rename = "startIconUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_icon_url: Option<String>,
    /// Optional start tooltip.
    #[serde(rename = "startTooltip")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_tooltip: Option<String>,
    /// Optional top label.
    #[serde(rename = "topLabel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_label: Option<String>,
    /// Primary text.
    pub text: String,
    /// Optional end icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_icon: Option<Icon>,
    /// Optional end icon URL.
    #[serde(rename = "endIconUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_icon_url: Option<String>,
    /// Optional end tooltip.
    #[serde(rename = "endTooltip")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_tooltip: Option<String>,
    /// Optional bottom label.
    #[serde(rename = "bottomLabel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom_label: Option<String>,
    /// Optional on-click action.
    #[serde(rename = "onClick")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_click: Option<OnClick>,
    /// Whether the text is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Optional divider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divider: Option<Divider>,
}

impl DecoratedText {
    /// Create a new decorated text with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            start_icon: None,
            start_icon_url: None,
            start_tooltip: None,
            top_label: None,
            text: text.into(),
            end_icon: None,
            end_icon_url: None,
            end_tooltip: None,
            bottom_label: None,
            on_click: None,
            disabled: None,
            divider: None,
        }
    }

    /// Set the start icon.
    pub fn start_icon(mut self, start_icon: Icon) -> Self {
        self.start_icon = Some(start_icon);
        self
    }

    /// Set the start icon URL.
    pub fn start_icon_url(mut self, start_icon_url: impl Into<String>) -> Self {
        self.start_icon_url = Some(start_icon_url.into());
        self
    }

    /// Set the start tooltip.
    pub fn start_tooltip(mut self, start_tooltip: impl Into<String>) -> Self {
        self.start_tooltip = Some(start_tooltip.into());
        self
    }

    /// Set the top label.
    pub fn top_label(mut self, top_label: impl Into<String>) -> Self {
        self.top_label = Some(top_label.into());
        self
    }

    /// Set the end icon.
    pub fn end_icon(mut self, end_icon: Icon) -> Self {
        self.end_icon = Some(end_icon);
        self
    }

    /// Set the end icon URL.
    pub fn end_icon_url(mut self, end_icon_url: impl Into<String>) -> Self {
        self.end_icon_url = Some(end_icon_url.into());
        self
    }

    /// Set the end tooltip.
    pub fn end_tooltip(mut self, end_tooltip: impl Into<String>) -> Self {
        self.end_tooltip = Some(end_tooltip.into());
        self
    }

    /// Set the bottom label.
    pub fn bottom_label(mut self, bottom_label: impl Into<String>) -> Self {
        self.bottom_label = Some(bottom_label.into());
        self
    }

    /// Set the on-click action.
    pub fn on_click(mut self, on_click: OnClick) -> Self {
        self.on_click = Some(on_click);
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Set divider.
    pub fn divider(mut self, divider: Divider) -> Self {
        self.divider = Some(divider);
        self
    }
}

/// Card actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAction {
    /// Action text.
    pub text: String,
    /// On-click action.
    #[serde(rename = "onClick")]
    pub on_click: OnClick,
}

impl CardAction {
    /// Create a new card action.
    pub fn new(text: impl Into<String>, on_click: OnClick) -> Self {
        Self {
            text: text.into(),
            on_click,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_builder_empty() {
        let card = CardBuilder::new().build();
        assert_eq!(card, json!({}));
    }

    #[test]
    fn test_card_builder_with_header() {
        let header = CardHeader::new("Test Card")
            .subtitle("Subtitle")
            .image(CardImage::new("https://example.com/image.png"));

        let card = CardBuilder::new()
            .header(header)
            .build();

        assert_eq!(card["header"]["title"], "Test Card");
        assert_eq!(card["header"]["subtitle"], "Subtitle");
        assert_eq!(card["header"]["image"]["imageUrl"], "https://example.com/image.png");
    }

    #[test]
    fn test_card_builder_with_section() {
        let section = CardSection::new()
            .header("Section Title")
            .widget(Widget::TextParagraph(TextParagraph::new("Hello, world!")));

        let card = CardBuilder::new()
            .section(section)
            .build();

        assert_eq!(card["sections"][0]["header"], "Section Title");
        assert_eq!(card["sections"][0]["widgets"][0]["textParagraph"]["text"], "Hello, world!");
    }

    #[test]
    fn test_card_builder_with_button() {
        let button = ButtonWidget::new("Click me")
            .on_click(OnClick::OpenLink(OpenLink::new("https://example.com")));

        let card = CardBuilder::new()
            .section(
                CardSection::new()
                    .widget(Widget::Button(button))
            )
            .build();

        assert_eq!(card["sections"][0]["widgets"][0]["button"]["text"], "Click me");
    }

    #[test]
    fn test_text_paragraph_with_format() {
        let widget = Widget::TextParagraph(TextParagraph::new("<b>Bold text</b>").format(TextFormat::Html));
        let json = json!(widget);
        assert_eq!(json["textParagraph"]["text"], "<b>Bold text</b>");
        assert_eq!(json["textParagraph"]["format"], "HTML");
    }

    #[test]
    fn test_image_widget_with_layout() {
        let widget = Widget::Image(ImageWidget::new("https://example.com/image.png")
            .layout(ImageLayout::Fill));

        let json = json!(widget);
        assert_eq!(json["image"]["layout"], "FILL");
    }

    #[test]
    fn test_button_list() {
        let button1 = ButtonWidget::new("Button 1");
        let button2 = ButtonWidget::new("Button 2");

        let widget = Widget::ButtonList(ButtonList::new()
            .buttons(vec![button1, button2]));

        let json = json!(widget);
        assert_eq!(json["buttonList"]["buttons"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_decorated_text() {
        let widget = Widget::DecoratedText(DecoratedText::new("Primary text")
            .top_label("Top label")
            .bottom_label("Bottom label"));

        let json = json!(widget);
        assert_eq!(json["decoratedText"]["text"], "Primary text");
        assert_eq!(json["decoratedText"]["topLabel"], "Top label");
        assert_eq!(json["decoratedText"]["bottomLabel"], "Bottom label");
    }

    #[test]
    fn test_selection_input_widget() {
        let item1 = SelectionItem::new("item1", "Item 1");
        let item2 = SelectionItem::new("item2", "Item 2");

        let widget = Widget::SelectionInput(SelectionInputWidget::new("select1", "Select an item", SelectionType::RadioButton)
            .items(vec![item1, item2])
            .multi_select(false));

        let json = json!(widget);
        assert_eq!(json["selectionInput"]["multiSelect"], false);
        assert_eq!(json["selectionInput"]["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_card_with_action() {
        let card = CardBuilder::new()
            .action(CardAction::new("Open", OnClick::OpenLink(OpenLink::new("https://example.com"))));

        let json = card.build();
        assert_eq!(json["action"][0]["text"], "Open");
    }

    #[test]
    fn test_icon_serialization() {
        let json = json!(Icon::Star);
        assert_eq!(json, "STAR");

        let json = json!(Icon::Heart);
        assert_eq!(json, "HEART");
    }

    #[test]
    fn test_text_format_serialization() {
        let json = json!(TextFormat::Plain);
        assert_eq!(json, "PLAIN");

        let json = json!(TextFormat::Html);
        assert_eq!(json, "HTML");
    }

    #[test]
    fn test_selection_type_serialization() {
        let json = json!(SelectionType::RadioButton);
        assert_eq!(json, "RADIO_BUTTON");

        let json = json!(SelectionType::Checkbox);
        assert_eq!(json, "CHECKBOX");
    }

    #[test]
    fn test_full_card_example() {
        // Create a full card with header, sections, and actions
        let card = CardBuilder::new()
            .header(
                CardHeader::new("Project Update")
                    .subtitle("Weekly status report")
                    .image(CardImage::new("https://example.com/logo.png"))
            )
            .section(
                CardSection::new()
                    .header("Overview")
                    .widget(Widget::TextParagraph(TextParagraph::new("This week we made progress on the new features.")))
            )
            .section(
                CardSection::new()
                    .widget(Widget::DecoratedText(DecoratedText::new("Task 1: Complete UI design")
                        .top_label("Status")
                        .bottom_label("In progress")
                        .start_icon(Icon::CheckBoxOutlineBlank)
                    ))
            )
            .section(
                CardSection::new()
                    .widget(Widget::ButtonList(ButtonList::new()
                        .button(
                            ButtonWidget::new("View Details")
                                .on_click(OnClick::OpenLink(OpenLink::new("https://example.com/task1")))
                        )
                        .button(
                            ButtonWidget::new("Update Status")
                                .on_click(OnClick::Action(Action::new("update_status")))
                        )
                    ))
            )
            .action(CardAction::new("Action 1", OnClick::OpenLink(OpenLink::new("https://example.com/action1"))))
            .action(CardAction::new("Action 2", OnClick::OpenLink(OpenLink::new("https://example.com/action2"))))
            .build();

        assert_eq!(card["header"]["title"], "Project Update");
        assert_eq!(card["sections"].as_array().unwrap().len(), 3);
        assert_eq!(card["action"].as_array().unwrap().len(), 2);
    }
}
