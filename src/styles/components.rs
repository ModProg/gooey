//! All style components supported by the built-in widgets.
use std::borrow::Cow;

use kludgine::figures::units::{Lp, Px};
use kludgine::figures::Rect;
use kludgine::Color;

use crate::animation::easings::{EaseInQuadradic, EaseOutQuadradic};
use crate::animation::EasingFunction;
use crate::styles::{ComponentDefinition, ComponentName, Dimension, Global, NamedComponent};

/// The [`Dimension`] to use as the size to render text.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TextSize;

impl NamedComponent for TextSize {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("text_size"))
    }
}

impl ComponentDefinition for TextSize {
    type ComponentType = Dimension;

    fn default_value(&self) -> Dimension {
        Dimension::Lp(Lp::points(12))
    }
}

/// The [`Dimension`] to use to space multiple lines of text.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct LineHeight;

impl NamedComponent for LineHeight {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("line_height"))
    }
}

impl ComponentDefinition for LineHeight {
    type ComponentType = Dimension;

    fn default_value(&self) -> Dimension {
        Dimension::Lp(Lp::points(14))
    }
}

/// The [`Color`] to use when rendering text.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TextColor;

impl NamedComponent for TextColor {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("text_color"))
    }
}

impl ComponentDefinition for TextColor {
    type ComponentType = Color;

    fn default_value(&self) -> Color {
        Color::WHITE
    }
}

/// A [`Color`] to be used as a highlight color.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct HighlightColor;

impl NamedComponent for HighlightColor {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("highlight_color"))
    }
}

impl ComponentDefinition for HighlightColor {
    type ComponentType = Color;

    fn default_value(&self) -> Color {
        Color::AQUA
    }
}

/// Intrinsic, uniform padding for a widget.
///
/// This component is opt-in and does not automatically work for all widgets.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct IntrinsicPadding;

impl NamedComponent for IntrinsicPadding {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("padding"))
    }
}

impl ComponentDefinition for IntrinsicPadding {
    type ComponentType = Dimension;

    fn default_value(&self) -> Dimension {
        Dimension::Lp(Lp::points(5))
    }
}

/// The [`EasingFunction`] to apply to animations that have no inherent
/// directionality.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Easing;

impl NamedComponent for Easing {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("easing"))
    }
}

impl ComponentDefinition for Easing {
    type ComponentType = EasingFunction;

    fn default_value(&self) -> Self::ComponentType {
        EasingFunction::from(EaseInQuadradic)
    }
}

/// The [`EasingFunction`] to apply to animations that transition a value from
/// "nothing" to "something". For example, if an widget is animating a color's
/// alpha channel towards opaqueness, it would query for this style component.
/// Otherwise, it would use [`EasingOut`].
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct EasingIn;

impl NamedComponent for EasingIn {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("easing_in"))
    }
}

impl ComponentDefinition for EasingIn {
    type ComponentType = EasingFunction;

    fn default_value(&self) -> Self::ComponentType {
        EasingFunction::from(EaseInQuadradic)
    }
}

/// The [`EasingFunction`] to apply to animations that transition a value from
/// "something" to "nothing". For example, if an widget is animating a color's
/// alpha channel towards transparency, it would query for this style component.
/// Otherwise, it would use [`EasingIn`].
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct EasingOut;

impl NamedComponent for EasingOut {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("easing_out"))
    }
}

impl ComponentDefinition for EasingOut {
    type ComponentType = EasingFunction;

    fn default_value(&self) -> Self::ComponentType {
        EasingFunction::from(EaseOutQuadradic)
    }
}

/// A 2d ordering configuration.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VisualOrder {
    /// The ordering to apply horizontally.
    pub horizontal: HorizontalOrder,
    /// The ordering to apply vertically.
    pub vertical: VerticalOrder,
}

impl VisualOrder {
    /// Returns a right-to-left ordering.
    #[must_use]
    pub const fn right_to_left() -> Self {
        Self {
            horizontal: HorizontalOrder::RightToLeft,
            vertical: VerticalOrder::TopToBottom,
        }
    }

    /// Returns a left-to-right ordering.
    #[must_use]
    pub const fn left_to_right() -> Self {
        Self {
            horizontal: HorizontalOrder::LeftToRight,
            vertical: VerticalOrder::TopToBottom,
        }
    }

    /// Returns the reverse ordering of `self`.
    #[must_use]
    pub fn rev(self) -> Self {
        Self {
            horizontal: self.horizontal.rev(),
            vertical: self.vertical.rev(),
        }
    }
}

impl NamedComponent for VisualOrder {
    fn name(&self) -> Cow<'_, ComponentName> {
        Cow::Owned(ComponentName::named::<Global>("visual_order"))
    }
}

/// A horizontal direction.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HorizontalOrder {
    /// Describes an order starting at the left and proceeding to the right.
    LeftToRight,
    /// Describes an order starting at the right and proceeding to the left.
    RightToLeft,
}

impl HorizontalOrder {
    /// Returns the reverse order of `self`.
    #[must_use]
    pub fn rev(self) -> Self {
        match self {
            Self::LeftToRight => Self::RightToLeft,
            Self::RightToLeft => Self::LeftToRight,
        }
    }

    pub(crate) fn sort_key(self, rect: &Rect<Px>) -> Px {
        match self {
            HorizontalOrder::LeftToRight => rect.origin.x,
            HorizontalOrder::RightToLeft => -(rect.origin.x + rect.size.width),
        }
    }
}

/// A vertical direction.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum VerticalOrder {
    /// Describes an order starting at the top and proceeding to the bottom.
    TopToBottom,
    /// Describes an order starting at the bottom and proceeding to the top.
    BottomToTop,
}

impl VerticalOrder {
    /// Returns the reverse order of `self`.
    #[must_use]
    pub fn rev(self) -> Self {
        match self {
            Self::TopToBottom => VerticalOrder::BottomToTop,
            Self::BottomToTop => VerticalOrder::TopToBottom,
        }
    }

    pub(crate) fn max_px(self) -> Px {
        match self {
            VerticalOrder::TopToBottom => Px::MAX,
            VerticalOrder::BottomToTop => Px::MIN,
        }
    }

    pub(crate) fn smallest_px(self, a: Px, b: Px) -> Px {
        match self {
            VerticalOrder::TopToBottom => a.min(b),
            VerticalOrder::BottomToTop => b.max(a),
        }
    }
}
