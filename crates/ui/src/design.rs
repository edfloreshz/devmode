//! The shared visual language: spacing scale, and the handful of composite
//! widgets every screen is built from.
//!
//! Screens compose these rather than styling things ad hoc, so a change here
//! moves the whole app at once and the four screens can't quietly drift apart.

use iced::widget::{
    Column, Row, button, column, container, row, rule, scrollable, space, text, text_input,
};
use iced::{Center, Element, Fill, Font, Left, Padding, Theme};

/// A 4px-based spacing scale. Using named steps instead of magic numbers is
/// what keeps rhythm consistent across screens.
pub const XS: f32 = 4.0;
pub const SM: f32 = 8.0;
pub const MD: f32 = 12.0;
pub const LG: f32 = 16.0;
pub const XL: f32 = 24.0;

pub const TEXT_SM: f32 = 12.0;
pub const TEXT_MD: f32 = 14.0;
pub const TEXT_LG: f32 = 18.0;
pub const TEXT_XL: f32 = 22.0;

/// Monospace, for anything the user might compare character-by-character:
/// filesystem paths, remote URLs, env values, layout templates.
pub const MONO: Font = Font::MONOSPACE;

/// A screen's title row: big heading, optional subtitle, optional actions
/// pinned to the right.
pub fn page_header<'a, Message: 'a>(
    title: &'a str,
    subtitle: impl Into<Option<String>>,
    actions: impl Into<Option<Element<'a, Message>>>,
) -> Element<'a, Message> {
    let mut heading = column![text(title).size(TEXT_XL)].spacing(XS);

    if let Some(subtitle) = subtitle.into() {
        heading = heading.push(muted(text(subtitle).size(TEXT_MD)));
    }

    let mut header = row![heading].align_y(Center).spacing(MD);

    if let Some(actions) = actions.into() {
        header = header.push(space::horizontal()).push(actions);
    }

    header.width(Fill).into()
}

/// De-emphasised text — labels, hints, secondary metadata.
pub fn muted<'a, Message: 'a>(
    content: iced::widget::Text<'a, Theme>,
) -> Element<'a, Message> {
    content
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.extended_palette().background.strong.text),
        })
        .into()
}

/// A `label: value` pair, stacked. Used throughout the detail panes.
pub fn field<'a, Message: 'a>(label: &'a str, value: Element<'a, Message>) -> Element<'a, Message> {
    column![muted(text(label).size(TEXT_SM)), value]
        .spacing(XS)
        .width(Fill)
        .into()
}

/// A `label: value` pair whose value is monospaced (paths, URLs, templates).
pub fn mono_field<'a, Message: 'a>(label: &'a str, value: impl ToString) -> Element<'a, Message> {
    field(
        label,
        text(value.to_string())
            .font(MONO)
            .size(TEXT_MD)
            .width(Fill)
            .into(),
    )
}

/// A titled, bordered group of related controls.
pub fn section<'a, Message: 'a>(
    title: &'a str,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            text(title).size(TEXT_MD),
            rule::horizontal(1.0),
            body.into(),
        ]
        .spacing(MD),
    )
    .padding(LG)
    .width(Fill)
    .style(container::bordered_box)
    .into()
}

/// Shown wherever a list has nothing in it. An empty state that explains what
/// the thing is and offers the action that fills it beats a blank pane.
pub fn empty_state<'a, Message: 'a>(
    headline: &'a str,
    explanation: &'a str,
    action: impl Into<Option<Element<'a, Message>>>,
) -> Element<'a, Message> {
    let mut body = column![
        text(headline).size(TEXT_LG),
        muted(text(explanation).size(TEXT_MD).align_x(Center)),
    ]
    .spacing(SM)
    .align_x(Center)
    .max_width(420);

    if let Some(action) = action.into() {
        body = body.push(container(action).padding(Padding::default().top(SM)));
    }

    container(body)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .padding(XL)
        .into()
}

/// A small pill — workspace membership chips, drift markers, counts.
pub fn badge<'a, Message: 'a>(label: impl ToString, tone: Tone) -> Element<'a, Message> {
    container(text(label.to_string()).size(TEXT_SM))
        .padding(Padding::from([2.0, SM]))
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let pair = match tone {
                Tone::Neutral => palette.background.weak,
                Tone::Info => palette.primary.weak,
                Tone::Warning => palette.warning.weak,
                Tone::Danger => palette.danger.weak,
                Tone::Success => palette.success.weak,
            };

            container::Style {
                background: Some(pair.color.into()),
                text_color: Some(pair.text),
                border: iced::border::rounded(999),
                ..container::Style::default()
            }
        })
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}



/// A row of buttons, right-aligned — the standard footer for dialogs.
pub fn button_row<'a, Message: 'a>(
    buttons: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut row = Row::new().spacing(SM).align_y(Center);
    for button in buttons {
        row = row.push(button);
    }

    container(row).width(Fill).align_x(iced::Right).into()
}

/// The metrics every interactive control shares. Buttons and text inputs
/// both read these, so a button beside an input in a toolbar lines up with
/// it and the two can't drift apart later.
pub const CONTROL_TEXT: f32 = TEXT_MD;
pub const CONTROL_PADDING: Padding = Padding {
    top: SM,
    right: MD,
    bottom: SM,
    left: MD,
};

/// A text input at the shared control size.
///
/// Mirrors `text_input`'s own signature: the strings are copied into the
/// widget, so they aren't tied to the returned element's lifetime.
pub fn input<'a, Message: Clone + 'a>(
    placeholder: &str,
    value: &str,
) -> iced::widget::TextInput<'a, Message> {
    text_input(placeholder, value)
        .padding(CONTROL_PADDING)
        .size(CONTROL_TEXT)
}

fn control<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: Option<Message>,
) -> button::Button<'a, Message> {
    button(text(label).size(CONTROL_TEXT).align_y(Center))
        .padding(CONTROL_PADDING)
        .on_press_maybe(on_press)
}

/// The main action of a screen, dialog, or section.
pub fn primary_button<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    control(label, on_press.into())
        .style(button::primary)
        .into()
}

/// A supporting action, alongside a primary one.
pub fn secondary_button<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    control(label, on_press.into())
        .style(button::secondary)
        .into()
}

/// An action that destroys something: removing, deleting, untracking.
pub fn danger_button<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    control(label, on_press.into()).style(button::danger).into()
}

/// Wraps a screen's body in the standard page padding and scroll behaviour.
pub fn page<'a, Message: 'a>(content: Column<'a, Message>) -> Element<'a, Message> {
    scrollable(container(content.spacing(LG).width(Fill)).padding(XL))
        .width(Fill)
        .height(Fill)
        .into()
}

/// A list row that can be selected — the shared building block of the repo,
/// workspace, and discovery lists.
pub fn list_row<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    is_selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    button(content)
        .width(Fill)
        .padding(Padding::from([SM, MD]))
        .on_press(on_press)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();

            let background = if is_selected {
                Some(palette.primary.weak.color.into())
            } else if matches!(status, button::Status::Hovered) {
                Some(palette.background.weak.color.into())
            } else {
                None
            };

            button::Style {
                background,
                text_color: if is_selected {
                    palette.primary.weak.text
                } else {
                    palette.background.base.text
                },
                border: iced::border::rounded(4),
                ..button::Style::default()
            }
        })
        .into()
}

/// Left-aligns a fixed-width column of content; used for list panes.
pub fn pane<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: f32,
) -> Element<'a, Message> {
    container(content)
        .width(width)
        .height(Fill)
        .align_x(Left)
        .into()
}

