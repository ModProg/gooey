use std::{any::TypeId, convert::TryFrom, ops::Deref};

use gooey_core::{
    euclid::{Point2D, Rect, Size2D},
    styles::{BackgroundColor, Style},
    AnyTransmogrifier, AnyTransmogrifierContext, AnyWidget, Points, Transmogrifier,
    TransmogrifierContext, TransmogrifierState, Widget, WidgetRegistration,
};
use gooey_renderer::Renderer;
use winit::event::MouseButton;

use crate::Rasterizer;

pub trait WidgetRasterizer<R: Renderer>: Transmogrifier<Rasterizer<R>> + Sized + 'static {
    fn widget_type_id(&self) -> TypeId {
        TypeId::of::<<Self as Transmogrifier<Rasterizer<R>>>::Widget>()
    }

    fn render_within(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        bounds: Rect<f32, Points>,
        parent_style: &Style,
    ) {
        if let Some(rasterizer) = context.frontend.clipped_to(bounds) {
            rasterizer.rasterizerd_widget(
                context.registration.id().clone(),
                rasterizer.renderer().unwrap().clip_bounds(),
            );
            let effective_style = context
                .frontend
                .ui
                .stylesheet()
                .effective_style_for::<<Self as Transmogrifier<Rasterizer<R>>>::Widget>(
                    context.style.merge_with(parent_style, true),
                    context.ui_state,
                );

            if let Some(&color) = <Self::Widget as Widget>::background_color(&effective_style) {
                let renderer = rasterizer.renderer().unwrap();
                renderer.fill_rect::<BackgroundColor>(
                    &renderer.bounds(),
                    &Style::default().with(BackgroundColor(color)),
                );
            }

            self.render(TransmogrifierContext::new(
                context.registration.clone(),
                context.state,
                &rasterizer,
                context.widget,
                context.channels,
                &effective_style,
                context.ui_state,
            ));
        }
    }

    fn render(&self, context: TransmogrifierContext<'_, Self, Rasterizer<R>>);

    /// Calculate the content-size needed for this `widget`, trying to stay
    /// within `constraints`.
    fn content_size(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        constraints: Size2D<Option<f32>, Points>,
    ) -> Size2D<f32, Points>;

    #[allow(unused_variables)]
    fn hit_test(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn hovered(&self, context: TransmogrifierContext<'_, Self, Rasterizer<R>>) {}

    #[allow(unused_variables)]
    fn unhovered(&self, context: TransmogrifierContext<'_, Self, Rasterizer<R>>) {}

    #[allow(unused_variables)]
    fn mouse_move(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> bool {
        self.hit_test(context, location, rastered_size)
    }

    #[allow(unused_variables)]
    fn mouse_down(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        button: MouseButton,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> EventStatus {
        EventStatus::Ignored
    }

    #[allow(unused_variables)]
    fn mouse_drag(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        button: MouseButton,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) {
    }

    #[allow(unused_variables)]
    fn mouse_up(
        &self,
        context: TransmogrifierContext<'_, Self, Rasterizer<R>>,
        button: MouseButton,
        location: Option<Point2D<f32, Points>>,
        rastered_size: Size2D<f32, Points>,
    ) {
    }
}

pub trait AnyWidgetRasterizer<R: Renderer>: AnyTransmogrifier<Rasterizer<R>> + Send + Sync {
    fn render_within(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        bounds: Rect<f32, Points>,
        parent_style: &Style,
    );
    fn content_size(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        constraints: Size2D<Option<f32>, Points>,
    ) -> Size2D<f32, Points>;

    fn hit_test(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> bool;

    fn hovered(&self, context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>);

    fn unhovered(&self, context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>);

    fn mouse_move(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> bool;

    fn mouse_down(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        button: MouseButton,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> EventStatus;

    fn mouse_drag(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        button: MouseButton,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    );

    fn mouse_up(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        button: MouseButton,
        location: Option<Point2D<f32, Points>>,
        rastered_size: Size2D<f32, Points>,
    );
}

impl<T, R> AnyWidgetRasterizer<R> for T
where
    T: WidgetRasterizer<R> + AnyTransmogrifier<Rasterizer<R>> + Send + Sync + 'static,
    R: Renderer,
{
    fn render_within(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        bounds: Rect<f32, Points>,
        parent_style: &Style,
    ) {
        <Self as WidgetRasterizer<R>>::render_within(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            bounds,
            parent_style,
        );
    }

    fn content_size(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        constraints: Size2D<Option<f32>, Points>,
    ) -> Size2D<f32, Points> {
        <Self as WidgetRasterizer<R>>::content_size(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            constraints,
        )
    }

    fn hit_test(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> bool {
        <Self as WidgetRasterizer<R>>::hit_test(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            location,
            rastered_size,
        )
    }

    fn hovered(&self, context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>) {
        <Self as WidgetRasterizer<R>>::hovered(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
        );
    }

    fn unhovered(&self, context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>) {
        <Self as WidgetRasterizer<R>>::unhovered(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
        );
    }

    fn mouse_move(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> bool {
        <Self as WidgetRasterizer<R>>::mouse_move(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            location,
            rastered_size,
        )
    }

    fn mouse_down(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        button: MouseButton,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) -> EventStatus {
        <Self as WidgetRasterizer<R>>::mouse_down(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            button,
            location,
            rastered_size,
        )
    }

    fn mouse_drag(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        button: MouseButton,
        location: Point2D<f32, Points>,
        rastered_size: Size2D<f32, Points>,
    ) {
        <Self as WidgetRasterizer<R>>::mouse_drag(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            button,
            location,
            rastered_size,
        );
    }

    fn mouse_up(
        &self,
        context: &mut AnyTransmogrifierContext<'_, Rasterizer<R>>,
        button: MouseButton,
        location: Option<Point2D<f32, Points>>,
        rastered_size: Size2D<f32, Points>,
    ) {
        <Self as WidgetRasterizer<R>>::mouse_up(
            self,
            TransmogrifierContext::try_from(context).unwrap(),
            button,
            location,
            rastered_size,
        );
    }
}

impl<R: Renderer> AnyTransmogrifier<Rasterizer<R>> for RegisteredTransmogrifier<R> {
    fn process_messages(&self, context: AnyTransmogrifierContext<'_, Rasterizer<R>>) {
        self.0.as_ref().process_messages(context);
    }

    fn widget_type_id(&self) -> TypeId {
        self.0.widget_type_id()
    }

    fn default_state_for(
        &self,
        widget: &mut dyn AnyWidget,
        registration: &WidgetRegistration,
        frontend: &Rasterizer<R>,
    ) -> TransmogrifierState {
        self.0.default_state_for(widget, registration, frontend)
    }
}

#[derive(Debug)]
pub struct RegisteredTransmogrifier<R: Renderer>(pub Box<dyn AnyWidgetRasterizer<R>>);

impl<R: Renderer> Deref for RegisteredTransmogrifier<R> {
    type Target = Box<dyn AnyWidgetRasterizer<R>>;

    fn deref(&self) -> &'_ Self::Target {
        &self.0
    }
}

#[macro_export]
macro_rules! make_rasterized {
    ($transmogrifier:ident) => {
        impl<R: $crate::Renderer> From<$transmogrifier> for $crate::RegisteredTransmogrifier<R> {
            fn from(transmogrifier: $transmogrifier) -> Self {
                Self(std::boxed::Box::new(transmogrifier))
            }
        }
    };
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum EventStatus {
    Ignored,
    Processed,
}
