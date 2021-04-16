use tui::layout::Rect;

pub trait RectExt {
    /// Calculates a rectangle centered on `self`, with the same size as `other`.
    ///
    /// Width and height of other rectangle are truncated to that of `self`
    fn centered(self, other: Rect) -> Rect;

    /// Calculates a rectangle with a new height and width
    fn scaled(self, x_scale: f64, y_scale: f64) -> Rect;
}

impl RectExt for Rect {
    fn centered(self, other: Rect) -> Rect {
        let width = self.width.min(other.width);
        let height = self.height.min(other.height);
        Rect {
            x: self.x + ((self.width - width) as f64 / 2.0) as u16,
            y: self.y + ((self.height - height) as f64 / 2.0) as u16,
            width,
            height,
        }
    }

    fn scaled(self, x_scale: f64, y_scale: f64) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: (self.width as f64 * x_scale) as u16,
            height: (self.height as f64 * y_scale) as u16,
        }
    }
}
