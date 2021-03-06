mod bar;
mod message;
mod list;
mod window;
mod text;
mod pixmap;

pub use self::message::UiMessageLog;
pub use self::bar::UiBar;
pub use self::list::UiList;
pub use self::window::UiWindow;
pub use self::text::UiText;
pub use self::pixmap::UiPixmap;

pub use super::subrenderer::UiSubRenderer;

use point::Point2d;

pub trait UiElement {
    fn draw<'a>(&self, renderer: &UiSubRenderer<'a>);
    fn required_size(&self, _constraint: Point2d) -> Point2d {
        Point2d::new(1, 1)
    }
    fn layout(&mut self, _constraint: Point2d) {}
}
