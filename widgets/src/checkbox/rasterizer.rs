use gooey_core::{
    figures::{Displayable, Point, Rect, Rectlike, Size, SizedRect, Vector},
    styles::{ForegroundColor, HighlightColor},
    Scaled, Transmogrifier, TransmogrifierContext,
};
use gooey_rasterizer::{
    winit::event::{ElementState, MouseButton, ScanCode, VirtualKeyCode},
    ContentArea, EventStatus, Rasterizer, Renderer, TransmogrifierContextExt, WidgetRasterizer,
};
use gooey_text::{prepared::PreparedText, wrap::TextWrap, Text};

use crate::{
    button::ButtonColor,
    checkbox::{
        Checkbox, CheckboxCommand, CheckboxTransmogrifier, InternalCheckboxEvent, LABEL_PADDING,
    },
};

impl<R: Renderer> Transmogrifier<Rasterizer<R>> for CheckboxTransmogrifier {
    type State = ();
    type Widget = Checkbox;

    fn receive_command(
        &self,
        _command: CheckboxCommand,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
    ) {
        context.frontend.set_needs_redraw();
    }
}

#[derive(Clone, Default, Debug)]
pub struct LayoutState {
    content_size: Size<f32, Scaled>,
    checkbox_size: Size<f32, Scaled>,
    label_size: Size<f32, Scaled>,
    label: PreparedText,
}

fn calculate_layout<R: Renderer>(
    context: &TransmogrifierContext<'_, CheckboxTransmogrifier, Rasterizer<R>>,
    renderer: &R,
    size: Size<f32, Scaled>,
) -> LayoutState {
    // Determine the checkbox size by figuring out the line height
    let line_height = renderer
        .measure_text_with_style("m", context.style())
        .height();
    let checkbox_size = Size::from_figures(line_height, line_height);

    // Measure the label, allowing the text to wrap within the remaining space.
    let label_size = Size::new(
        (size.width - checkbox_size.width - LABEL_PADDING.get()).max(0.),
        size.height,
    );
    let label = Text::span(&context.widget.label, context.style().clone()).wrap(
        renderer,
        TextWrap::MultiLine { size: label_size },
        Some(context.style()),
    );

    let label_size = label.size();

    LayoutState {
        content_size: label_size + Vector::from_x(checkbox_size.width + LABEL_PADDING.get()),
        checkbox_size,
        label_size,
        label,
    }
}

impl<R: Renderer> WidgetRasterizer<R> for CheckboxTransmogrifier {
    fn render(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        content_area: &ContentArea,
    ) {
        if let Some(renderer) = context.frontend.renderer() {
            let layout = calculate_layout(context, renderer, content_area.size.content);
            let scale = renderer.scale();

            // Render the checkbox
            let checkbox_rect = SizedRect::new(content_area.location, layout.checkbox_size)
                .to_pixels(&scale)
                .round_out();
            renderer
                .fill_rect_with_style::<ButtonColor, _>(&checkbox_rect.as_rect(), context.style());
            if context.ui_state.focused {
                renderer.stroke_rect_with_style::<HighlightColor, _>(
                    &checkbox_rect.as_rect(),
                    context.style(),
                );
            }
            if context.widget.checked {
                // Fill a square in the middle with the mark.
                let check_box = checkbox_rect.inflate(
                    Vector::from_figures(
                        -checkbox_rect.size.width() / 3.,
                        -checkbox_rect.size.width() / 3.,
                    )
                    .to_pixels(&scale),
                );
                renderer.fill_rect_with_style::<ForegroundColor, _>(
                    &check_box.as_rect(),
                    context.style(),
                );
            }

            // Render the label
            let label_rect = Rect::sized(
                Point::from_x(layout.checkbox_size.width + LABEL_PADDING.get())
                    + content_area.location,
                layout.label_size,
            );
            layout
                .label
                .render_within::<ForegroundColor, _>(renderer, label_rect, context.style());
        }
    }

    fn measure_content(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        constraints: Size<Option<f32>, Scaled>,
    ) -> Size<f32, Scaled> {
        // Always render a rect
        context
            .frontend
            .renderer()
            .map_or_else(Size::default, |renderer| {
                let renderer_size = renderer.size();
                let layout = calculate_layout(
                    context,
                    renderer,
                    Size::new(
                        constraints.width.unwrap_or(renderer_size.width),
                        constraints.height.unwrap_or(renderer_size.height),
                    ),
                );

                layout.content_size
            })
    }

    fn mouse_down(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        button: MouseButton,
        _location: Point<f32, Scaled>,
        _area: &ContentArea,
    ) -> EventStatus {
        if button == MouseButton::Left {
            context.activate();
            EventStatus::Processed
        } else {
            EventStatus::Ignored
        }
    }

    fn mouse_drag(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        _button: MouseButton,
        location: Point<f32, Scaled>,
        area: &ContentArea,
    ) {
        if area.bounds().contains(location) {
            context.activate();
        } else {
            context.frontend.deactivate();
        }
    }

    fn mouse_up(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        _button: MouseButton,
        location: Option<Point<f32, Scaled>>,
        area: &ContentArea,
    ) {
        if location
            .map(|location| area.bounds().contains(location))
            .unwrap_or_default()
        {
            context.channels.post_event(InternalCheckboxEvent::Clicked);
            context.focus();
        }
        context.deactivate();
    }

    fn keyboard(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        _scancode: ScanCode,
        keycode: Option<VirtualKeyCode>,
        state: ElementState,
    ) -> EventStatus {
        match keycode {
            Some(VirtualKeyCode::NumpadEnter | VirtualKeyCode::Return | VirtualKeyCode::Space) => {
                if matches!(state, ElementState::Pressed) {
                    context.activate();
                } else {
                    context.deactivate();
                    context.channels.post_event(InternalCheckboxEvent::Clicked);
                }
                EventStatus::Processed
            }
            _ => EventStatus::Ignored,
        }
    }
}
