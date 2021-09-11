use gooey::{
    core::{Context, DefaultWidget, StyledWidget},
    widgets::{
        checkbox::Checkbox,
        component::{Behavior, Component, Content, EventMapper},
        container::Container,
    },
    App,
};

#[cfg(test)]
mod harness;

fn app() -> App {
    App::from_root(|storage| Component::<Counter>::default_for(storage)).with_component::<Counter>()
}

fn main() {
    app().run();
}

#[derive(Default, Debug)]
struct Counter;

impl Behavior for Counter {
    type Content = Container;
    type Event = CounterEvent;
    type Widgets = CounterWidgets;

    fn build_content(
        &mut self,
        builder: <Self::Content as Content<Self>>::Builder,
        events: &EventMapper<Self>,
    ) -> StyledWidget<Container> {
        builder
            .child(
                CounterWidgets::Checkbox,
                Checkbox::build()
                    .labeled("I'm a checkbox. Hear me roar.")
                    .on_clicked(events.map(|_| CounterEvent::ButtonClicked))
                    .finish(),
            )
            .finish()
    }

    fn receive_event(
        component: &mut Component<Self>,
        event: Self::Event,
        context: &Context<Component<Self>>,
    ) {
        let CounterEvent::ButtonClicked = event;

        let checkbox_state = component
            .widget_state(&CounterWidgets::Checkbox, context)
            .unwrap();
        let mut checkbox = checkbox_state.lock::<Checkbox>(context.frontend()).unwrap();
        if checkbox.widget.checked() {
            checkbox
                .widget
                .set_label("I'm a checked checkbox now.", &checkbox.context);
        } else {
            checkbox
                .widget
                .set_label("I am no longer checked.", &checkbox.context);
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum CounterWidgets {
    Checkbox,
}

#[derive(Debug)]
enum CounterEvent {
    ButtonClicked,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gooey::{
        core::{figures::Size, styles::SystemTheme},
        HeadlessError,
    };

    use super::*;

    #[cfg(not(target_arch = "wasm32-unknown-unknown"))]
    #[tokio::test]
    async fn demo() -> Result<(), HeadlessError> {
        for theme in [SystemTheme::Dark, SystemTheme::Light] {
            let mut headless = app().headless();
            let mut recorder = headless.begin_recording(Size::new(320, 240), theme, true, 30);
            recorder.set_cursor((100., 200.));
            recorder.render_frame(Duration::from_millis(100)).await?;
            recorder
                .move_cursor_to((160., 120.), Duration::from_millis(300))
                .await?;
            recorder.left_click().await?;

            assert!(recorder
                .map_root_widget(|component: &mut Component<Counter>, context| {
                    component
                        .map_widget(
                            &CounterWidgets::Checkbox,
                            &context,
                            |button: &Checkbox, _context| button.checked(),
                        )
                        .unwrap()
                })
                .unwrap());

            // Wiggle the cursor to make the second click seem like a click.
            recorder
                .move_cursor_to((150., 140.), Duration::from_millis(100))
                .await?;
            recorder.pause(Duration::from_millis(00));
            recorder
                .move_cursor_to((160., 120.), Duration::from_millis(200))
                .await?;

            recorder.left_click().await?;

            assert!(!recorder
                .map_root_widget(|component: &mut Component<Counter>, context| {
                    component
                        .map_widget(
                            &CounterWidgets::Checkbox,
                            &context,
                            |button: &Checkbox, _context| button.checked(),
                        )
                        .unwrap()
                })
                .unwrap());

            recorder
                .move_cursor_to((200., 180.), Duration::from_millis(300))
                .await?;
            recorder.pause(Duration::from_millis(1000));

            recorder.save_apng(harness::snapshot_path(
                "checkbox",
                &format!("Demo-{:?}.png", theme),
            )?)?;
        }
        Ok(())
    }
}
