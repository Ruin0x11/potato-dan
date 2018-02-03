mod atlas;
mod render;
mod util;

mod interop;

pub use self::render::{RenderContext};
pub use self::interop::RenderUpdate;

make_global!(RENDERER, RenderContext, RenderContext::new());

pub use self::instance::*;
