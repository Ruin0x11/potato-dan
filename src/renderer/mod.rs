mod atlas;
mod render;
mod util;
mod traits;
pub mod ui;

pub use self::render::{RenderContext};
pub use self::traits::RenderUpdate;

make_global!(RENDERER, RenderContext, RenderContext::new());

pub use self::instance::*;
