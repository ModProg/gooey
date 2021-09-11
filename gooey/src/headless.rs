use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
    path::Path,
    process::Command,
    time::Duration,
};

use gooey_core::{
    figures::{Angle, Point, Size, Vectorlike},
    styles::SystemTheme,
    Context, Frontend, Pixels, Scaled, Widget,
};
use gooey_kludgine::{
    kludgine::{
        self,
        core::{
            easygpu::{self, wgpu},
            flume, FrameRenderer,
        },
        prelude::{ElementState, Fill, MouseButton, Scene, Shape, Stroke, Target, VirtualKeyCode},
    },
    Kludgine,
};
use gooey_rasterizer::{
    events::{InputEvent, WindowEvent},
    winit::{event::ModifiersState, window::Theme},
    EventResult, Rasterizer, Renderer,
};
use image::{DynamicImage, RgbImage};
use tempfile::NamedTempFile;

/// A headless application.
#[must_use]
pub struct Headless<F: Frontend> {
    frontend: F,
}

impl<F: Frontend> Headless<F> {
    pub(crate) fn new(frontend: F) -> Self {
        Self { frontend }
    }
}

impl<R: Renderer> Headless<Rasterizer<R>> {
    /// Looks up the root widget of the frontend and invokes `callback` with the widget and a context that can be used to interact with it. The result will be returned.
    pub fn map_root_widget<W: Widget, Output, F: FnOnce(&mut W, Context<W>) -> Output>(
        &self,
        callback: F,
    ) -> Option<Output> {
        let root = self.frontend.gooey().root_widget();
        self.frontend
            .with_transmogrifier(root.id(), |_transmogrifier, context| {
                callback(
                    context
                        .widget
                        .as_mut_any()
                        .downcast_mut::<W>()
                        .expect("widget type mismatch"),
                    Context::new(
                        context
                            .channels
                            .as_any()
                            .downcast_ref()
                            .expect("widget type mismatch"),
                        &self.frontend,
                    ),
                )
            })
    }
}

impl Headless<Rasterizer<Kludgine>> {
    /// Process an event. Only supported with a rasterizer frontend.
    pub fn simulate_event(&mut self, event: WindowEvent) -> EventResult {
        let (scene_sender, _) = flume::unbounded();
        let result = self.frontend.handle_event(
            event,
            Kludgine::from(&Target::from(Scene::new(scene_sender, Theme::Light))),
        );
        self.frontend
            .gooey()
            .process_widget_messages(&self.frontend);
        result
    }

    /// Sets the location of the cursor to `position`. Does not render any frames.
    pub fn set_cursor(&mut self, position: impl Into<Point<f32, Scaled>>) {
        self.simulate_event(WindowEvent::Input(InputEvent::MouseMoved {
            position: Some(position.into()),
        }));
    }

    /// Simulates a left click at the current cursor location.
    pub fn left_click(&mut self) {
        self.simulate_event(WindowEvent::Input(InputEvent::MouseButton {
            button: MouseButton::Left,
            state: ElementState::Pressed,
        }));
        self.simulate_event(WindowEvent::Input(InputEvent::MouseButton {
            button: MouseButton::Left,
            state: ElementState::Released,
        }));
    }

    /// Captures a screenshot with the size and theme provided.
    ///
    /// # Panics
    ///
    /// Panics if no `wgpu` adapter can be initialized.
    ///
    /// # Errors
    ///
    /// Returns any errors that arise during the rendering process.
    pub async fn screenshot(
        &self,
        size: Size<u32, Pixels>,
        theme: SystemTheme,
        cursor: Option<Point<f32, Scaled>>,
    ) -> Result<DynamicImage, HeadlessError> {
        let (scene_sender, scene_receiver) = flume::unbounded();
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
            })
            .await
            .expect("No wgpu adapter found");
        let renderer = easygpu::renderer::Renderer::offscreen(&adapter, 4).await?;
        let mut target = Target::from(Scene::new(
            scene_sender,
            match theme {
                SystemTheme::Light => Theme::Light,
                SystemTheme::Dark => Theme::Dark,
            },
        ));
        target.scene_mut().unwrap().set_size(size.cast_unit());
        target.scene_mut().unwrap().start_frame();
        self.frontend
            .gooey()
            .process_widget_messages(&self.frontend);
        self.frontend.render(Kludgine::from(&target));

        if let Some(cursor) = cursor {
            const CURSOR_LENGTH: f32 = 16.;
            const INNER_LENGTH: f32 = 14.;
            const TAIL_LENGTH: f32 = 20.;
            let left_edge_lower = Point::from_y(CURSOR_LENGTH);
            let right_edge_lower = Angle::Degrees(-45.).transform_point(left_edge_lower);
            let left_inner =
                Angle::Degrees(-17.5).transform_point(Point::<f32, Scaled>::new(0., INNER_LENGTH));
            let right_inner =
                Angle::Degrees(-27.5).transform_point(Point::<f32, Scaled>::new(0., INNER_LENGTH));
            let left_tail =
                Angle::Degrees(-17.5).transform_point(Point::<f32, Scaled>::new(0., TAIL_LENGTH));
            let right_tail =
                Angle::Degrees(-27.5).transform_point(Point::<f32, Scaled>::new(0., TAIL_LENGTH));
            Shape::polygon(vec![
                Point::default(),
                left_edge_lower,
                left_inner,
                left_tail,
                right_tail,
                right_inner,
                right_edge_lower,
            ])
            .fill(Fill::new(kludgine::core::Color::BLACK))
            .stroke(Stroke::new(kludgine::core::Color::WHITE))
            .render_at(&cursor, &target);
        }

        target.scene_mut().unwrap().end_frame();

        Ok(
            FrameRenderer::<kludgine::core::sprite::Srgb>::render_one_frame(
                renderer,
                scene_receiver,
            )
            .await?,
        )
    }

    /// Begins a recording session that generates an animation.
    pub fn begin_recording(
        &mut self,
        size: Size<u32, Pixels>,
        theme: SystemTheme,
        render_cursor: bool,
        fps: u16,
    ) -> Recorder<'_> {
        Recorder::new(size, theme, render_cursor, fps, self)
    }
}

/// An easy-to-use,offscreen animation recorder.
pub struct Recorder<'a> {
    headless: &'a mut Headless<Rasterizer<Kludgine>>,
    frames: Vec<RecordedFrame>,
    size: Size<u32, Pixels>,
    theme: SystemTheme,
    render_cursor: bool,
    fps: u16,
    cursor: Option<Point<f32, Scaled>>,
}

struct RecordedFrame {
    image: RgbImage,
    duration: Duration,
}

impl<'a> Deref for Recorder<'a> {
    type Target = Headless<Rasterizer<Kludgine>>;

    fn deref(&self) -> &Self::Target {
        self.headless
    }
}

impl<'a> DerefMut for Recorder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.headless
    }
}

impl<'a> Recorder<'a> {
    fn new(
        size: Size<u32, Pixels>,
        theme: SystemTheme,
        render_cursor: bool,
        fps: u16,
        headless: &'a mut Headless<Rasterizer<Kludgine>>,
    ) -> Self {
        Self {
            headless,
            size,
            theme,
            render_cursor,
            fps,
            frames: Vec::default(),
            cursor: None,
        }
    }

    /// Renders the current state of the application and displays it for `duration`.
    ///
    /// # Errors
    ///
    /// Returns any error that occurs while rendering.
    pub async fn render_frame(&mut self, duration: Duration) -> Result<(), HeadlessError> {
        let screenshot = self
            .headless
            .screenshot(
                self.size,
                self.theme,
                if self.render_cursor {
                    self.cursor
                } else {
                    None
                },
            )
            .await?;
        self.frames.push(RecordedFrame {
            image: screenshot.to_rgb8(),
            duration,
        });
        Ok(())
    }

    /// Simulates `event` in the application.
    pub fn simulate_event(&mut self, event: WindowEvent) {
        match &event {
            WindowEvent::Input(InputEvent::MouseMoved { position }) => self.cursor = *position,
            WindowEvent::Input(_)
            | WindowEvent::ReceiveCharacter(_)
            | WindowEvent::ModifiersChanged(_)
            | WindowEvent::LayerChanged { .. }
            | WindowEvent::RedrawRequested
            | WindowEvent::SystemThemeChanged(_) => {}
        }
        self.headless.simulate_event(event);
    }

    /// Extends the last frame to display for an additional `duration`.
    ///
    /// # Panics
    ///
    /// Panics if no frames have been rendered.
    pub fn pause(&mut self, duration: Duration) {
        self.frames
            .last_mut()
            .expect("can't pause with no frames")
            .duration += duration;
    }

    /// Moves the cursor from the current location (or -16,-16 if no current
    /// location) to `location`. The animation is performed over `duration`
    /// using the recorder's framerate.
    ///
    /// # Errors
    ///
    /// Returns any error that occurs while rendering.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    pub async fn move_cursor_to(
        &mut self,
        location: impl Into<Point<f32, Scaled>>,
        duration: Duration,
    ) -> Result<(), HeadlessError> {
        let duration = duration.as_secs_f32();
        let frames = (duration * f32::from(self.fps)).floor().max(1.);
        let frame_duration = Duration::from_secs_f32(duration / frames);
        let origin = self.cursor.unwrap_or_else(|| Point::new(-16., -16.));
        let step = (location.into().to_vector() - origin.to_vector()) / frames;
        for frame in 1..=frames as u32 {
            self.simulate_event(WindowEvent::Input(InputEvent::MouseMoved {
                position: Some(origin + step * frame as f32),
            }));
            self.render_frame(frame_duration).await?;
        }
        Ok(())
    }

    /// Simulates a left click at the current cursor location.
    ///
    /// # Errors
    ///
    /// Returns any error that occurs while rendering.
    pub async fn left_click(&mut self) -> Result<(), HeadlessError> {
        self.simulate_event(WindowEvent::Input(InputEvent::MouseButton {
            button: MouseButton::Left,
            state: ElementState::Pressed,
        }));
        self.render_frame(Duration::from_millis(100)).await?;
        self.simulate_event(WindowEvent::Input(InputEvent::MouseButton {
            button: MouseButton::Left,
            state: ElementState::Released,
        }));
        self.render_frame(Duration::from_millis(200)).await
    }

    /// Simulates a typing a key once.
    ///
    /// # Errors
    ///
    /// Returns any error that occurs while rendering.
    pub async fn press_key(
        &mut self,
        key: VirtualKeyCode,
        modifiers: impl Into<Option<ModifiersState>>,
    ) -> Result<(), HeadlessError> {
        let modifiers = modifiers.into();
        if let Some(modifiers) = modifiers {
            self.simulate_event(WindowEvent::ModifiersChanged(modifiers));
        }

        self.simulate_event(WindowEvent::Input(InputEvent::Keyboard {
            scancode: 0,
            key: Some(key),
            state: ElementState::Pressed,
        }));
        self.render_frame(Duration::from_millis(75)).await?;
        self.simulate_event(WindowEvent::Input(InputEvent::Keyboard {
            scancode: 0,
            key: Some(key),
            state: ElementState::Released,
        }));
        if modifiers.is_some() {
            self.simulate_event(WindowEvent::ModifiersChanged(ModifiersState::default()));
        }

        self.render_frame(Duration::from_millis(75)).await
    }

    /// Saves the current frames to `path` as an animated png.
    ///
    /// # Errors
    ///
    /// Can error from io or png encoding errors.
    pub fn save_apng<P: AsRef<Path>>(&self, path: P) -> Result<(), HeadlessError> {
        let file = std::fs::File::create(path)?;
        let mut encoder = png::Encoder::new(file, self.size.width, self.size.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_animated(self.frames.len().try_into()?, 0)?;
        let mut writer = encoder.write_header()?;
        for frame in &self.frames {
            writer.set_frame_delay(frame.duration.as_millis().try_into()?, 1000)?;
            writer.write_image_data(&frame.image)?;
        }
        Ok(())
    }

    /// Saves the current frames to `path` as an mp4. Requires the `ffmpeg`
    /// executable in the path.
    ///
    /// # Errors
    ///
    /// Can error from io or png encoding errors or from ffmpeg itself.
    ///
    /// # Panics
    ///
    /// Panics if ffmpeg errors and the output cannot be interpreted as utf8.
    pub fn save_webm<P: AsRef<Path>>(&self, path: P) -> Result<(), HeadlessError> {
        let (temp_file, temp_path) = NamedTempFile::new()?.into_parts();
        let mut encoder = png::Encoder::new(temp_file, self.size.width, self.size.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_animated(self.frames.len().try_into()?, 0)?;
        let mut writer = encoder.write_header()?;
        for frame in &self.frames {
            writer.set_frame_delay(frame.duration.as_millis().try_into()?, 1000)?;
            writer.write_image_data(&frame.image)?;
        }
        drop(writer);

        let result = Command::new("ffmpeg")
            // input
            .arg("-i")
            .arg(temp_path.as_os_str())
            // overwrite
            .arg("-y")
            // x264
            .arg("-c:v")
            .arg("libvpx-vp9")
            .arg("-crf")
            .arg("37")
            .arg("-b:v")
            .arg("0")
            // output
            .arg(path.as_ref().as_os_str())
            .output()?;

        if result.status.success() {
            Ok(())
        } else {
            Err(HeadlessError::Ffmpeg(
                String::from_utf8(result.stderr).unwrap(),
            ))
        }
    }
}

/// Errors that can occur while using [`Headless`].
#[derive(thiserror::Error, Debug)]
pub enum HeadlessError {
    /// An error from `kludgine` occurred.
    #[error("kludgine error: {0}")]
    Kludgine(#[from] kludgine::core::Error),
    /// An error from `easygpu` occurred.
    #[error("gpu error: {0}")]
    Gpu(#[from] easygpu::error::Error),
    /// A png encoding error occurred.
    #[error("png error: {0}")]
    Png(#[from] png::EncodingError),
    /// An io error occcurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// A value encountered was too large. This error shouldn't happen in practical use cases.
    #[error("value too large: a numerical conversion couldn't happen without truncation")]
    ValueTooLarge(#[from] std::num::TryFromIntError),
    /// An error occurred while interacting with `ffmpeg`. The contained string
    /// is the `stderr` output.
    #[error("ffmpeg error: {0}")]
    Ffmpeg(String),
}
