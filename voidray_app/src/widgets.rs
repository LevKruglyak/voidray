use crate::egui::*;

pub struct FatButton {
    text: WidgetText,
    wrap: Option<bool>,
    /// None means default for interact
    fill: Option<Color32>,
    stroke: Option<Stroke>,
    small: bool,
    frame: Option<bool>,
    min_size: Vec2,
}

impl FatButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            wrap: None,
            fill: None,
            stroke: None,
            small: false,
            frame: None,
            min_size: Vec2::ZERO,
        }
    }

    /// If `true`, the text will wrap to stay within the max width of the [`Ui`].
    ///
    /// By default [`Self::wrap`] will be true in vertical layouts
    /// and horizontal layouts with wrapping,
    /// and false on non-wrapping horizontal layouts.
    ///
    /// Note that any `\n` in the text will always produce a new line.
    #[inline]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Override background fill color. Note that this will override any on-hover effects.
    /// Calling this will also turn on the frame.
    pub fn fill(mut self, fill: impl Into<Color32>) -> Self {
        self.fill = Some(fill.into());
        self.frame = Some(true);
        self
    }

    /// Override button stroke. Note that this will override any on-hover effects.
    /// Calling this will also turn on the frame.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self.frame = Some(true);
        self
    }

    /// Make this a small button, suitable for embedding into text.
    pub fn small(mut self) -> Self {
        self.text = self.text.text_style(TextStyle::Body);
        self.small = true;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.min_size.x = width;
        self
    }

    /// Turn off the frame
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = Some(frame);
        self
    }

    pub(crate) fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
}

impl Widget for FatButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let FatButton {
            text,
            wrap,
            fill,
            stroke,
            small,
            frame,
            min_size,
        } = self;

        let frame = frame.unwrap_or_else(|| ui.visuals().button_frame);

        let mut button_padding = ui.spacing().button_padding;
        if small {
            button_padding.y = 0.0;
        }
        let total_extra = button_padding + button_padding;

        let wrap_width = ui.available_width() - total_extra.x;
        let text = text.into_galley(ui, wrap, wrap_width, TextStyle::Button);

        let mut desired_size = text.size() + 2.0 * button_padding;
        if !small {
            desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        }
        desired_size = desired_size.at_least(min_size);

        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, text.text()));

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                .min;

            if frame {
                let fill = fill.unwrap_or(visuals.bg_fill);
                let stroke = stroke.unwrap_or(visuals.bg_stroke);
                ui.painter().rect(
                    rect.expand(visuals.expansion),
                    visuals.rounding,
                    fill,
                    stroke,
                );
            }

            text.paint_with_visuals(ui.painter(), text_pos, visuals);
        }

        response
    }
}
