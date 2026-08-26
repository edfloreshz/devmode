//! The shared visual language: spacing scale, and the handful of composite
//! widgets every screen is built from.
//!
//! Screens compose these rather than styling things ad hoc, so a change here
//! moves the whole app at once and the four screens can't quietly drift apart.

use iced::widget::{
    Column, Row, button, column, container, row, rule, scrollable, space, text, text_input,
};
use iced::border;
use iced::{Center, Color, Element, Fill, Font, Left, Padding, Theme};

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
///
/// iced's own `button::secondary` pairs its gray background with dark text —
/// readable, but a flat gray-on-gray that doesn't read as clickable next to
/// a colored primary button. This keeps the gray but switches to white text,
/// darkening the background just enough to stay legible: `secondary.base` is
/// only ~0.59 gray in the Light theme, and white text on that is under 3:1
/// contrast — noticeably harder to read than iced's own dark-on-gray pairing.
pub fn secondary_button<'a, Message: Clone + 'a>(
    label: &'a str,
    on_press: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    control(label, on_press.into())
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            let readable = darken_for_contrast(palette.secondary.base.color, WHITE_TEXT_TARGET);

            let background = match status {
                button::Status::Hovered => darken(readable, 0.15),
                _ => readable,
            };

            let mut style = button::Style {
                background: Some(background.into()),
                text_color: Color::WHITE,
                border: border::rounded(2),
                ..button::Style::default()
            };

            if status == button::Status::Disabled {
                style.background = style.background.map(|bg| bg.scale_alpha(0.5));
                style.text_color = style.text_color.scale_alpha(0.5);
            }

            style
        })
        .into()
}

/// The contrast ratio white text needs against a secondary button's
/// background to stay comfortably readable (WCAG AA for normal-size text).
const WHITE_TEXT_TARGET: f32 = 4.5;

/// Mixes `color` toward black by `amount` (0 = unchanged, 1 = black).
fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: color.r * (1.0 - amount),
        g: color.g * (1.0 - amount),
        b: color.b * (1.0 - amount),
        a: color.a,
    }
}

/// Relative luminance per WCAG, used to compute contrast ratios.
fn luminance(color: Color) -> f32 {
    fn channel(value: f32) -> f32 {
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

fn contrast(a: Color, b: Color) -> f32 {
    let (lighter, darker) = {
        let (la, lb) = (luminance(a), luminance(b));
        if la > lb { (la, lb) } else { (lb, la) }
    };

    (lighter + 0.05) / (darker + 0.05)
}

/// Darkens `color` by binary search until white text on top reaches `target`
/// contrast, so this holds for any theme's palette — built-in or custom —
/// rather than a fixed darkening amount that happens to work for the themes
/// tested against.
fn darken_for_contrast(color: Color, target: f32) -> Color {
    if contrast(Color::WHITE, color) >= target {
        return color;
    }

    let (mut low, mut high) = (0.0_f32, 1.0_f32);

    for _ in 0..16 {
        let mid = (low + high) / 2.0;
        let candidate = darken(color, mid);

        if contrast(Color::WHITE, candidate) >= target {
            high = mid;
        } else {
            low = mid;
        }
    }

    darken(color, high)
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

#[cfg(test)]
mod contrast_tests {
    use super::*;
    use iced::Theme;

    /// Every built-in theme's secondary-button background, after darkening,
    /// must keep white text at least at the WCAG AA threshold for normal
    /// text. This is the guarantee `secondary_button` exists to make.
    #[test]
    fn white_text_stays_readable_on_every_built_in_theme() {
        for theme in Theme::ALL {
            let palette = theme.extended_palette();
            let background = darken_for_contrast(palette.secondary.base.color, WHITE_TEXT_TARGET);
            let ratio = contrast(Color::WHITE, background);

            assert!(
                ratio >= WHITE_TEXT_TARGET - 0.01,
                "{theme}: white-on-secondary contrast is {ratio:.2}:1, below {WHITE_TEXT_TARGET}:1"
            );
        }
    }

    #[test]
    fn a_background_that_already_passes_is_left_alone() {
        // A background already darker than needed shouldn't be darkened
        // further — that would needlessly flatten a theme's own gray.
        let already_dark = Color::from_rgb(0.1, 0.1, 0.1);

        assert_eq!(
            darken_for_contrast(already_dark, WHITE_TEXT_TARGET),
            already_dark
        );
    }

    #[test]
    fn hover_stays_readable_too() {
        // The hover state darkens further for feedback; confirm that step
        // doesn't undo the readability the base darkening established.
        for theme in Theme::ALL {
            let palette = theme.extended_palette();
            let base = darken_for_contrast(palette.secondary.base.color, WHITE_TEXT_TARGET);
            let hovered = darken(base, 0.15);

            assert!(
                contrast(Color::WHITE, hovered) >= WHITE_TEXT_TARGET - 0.01,
                "{theme}: hover state drops white text below {WHITE_TEXT_TARGET}:1"
            );
        }
    }
}

