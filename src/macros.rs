macro_rules! make_global {
    ($name:ident, $global_ty:ty, $maker:expr) => {
        pub mod instance {
            use super::*;
            use std::cell::RefCell;
            thread_local!(static $name: RefCell<$global_ty> = RefCell::new($maker); );

            pub fn with<A, F>(f: F) -> A
                where F: FnOnce(&$global_ty) -> A {
                $name.with(|w| f(& *w.borrow()))
            }

            pub fn with_mut<A, F>(f: F) -> A
                where F: FnOnce(&mut $global_ty) -> A {
                $name.with(|w| f(&mut *w.borrow_mut()))
            }
        }
    }
}

#[macro_export]
macro_rules! log(
    ($tag:expr) => {
        ::debug::log(&format!($tag));
    };
    ($tag:expr, $($args:tt)+) => {
        ::debug::log(&format!($tag, $($args)+));
    };
);
