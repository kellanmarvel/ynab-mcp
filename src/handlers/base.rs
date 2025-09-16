use crate::handlers::common::CommonUtils;

/// Base trait for all handlers that need access to CommonUtils
#[allow(dead_code)]
pub trait Handler<'a> {
    fn common(&self) -> &'a CommonUtils;
}

/// Macro to automatically implement the standard handler pattern
#[macro_export]
macro_rules! impl_handler {
    ($handler_type:ty) => {
        impl<'a> $handler_type {
            pub fn new(common: &'a CommonUtils) -> Self {
                Self { common }
            }
        }

        impl<'a> $crate::handlers::base::Handler<'a> for $handler_type {
            fn common(&self) -> &'a $crate::handlers::common::CommonUtils {
                self.common
            }
        }
    };
}

/// Standard handler struct pattern
#[macro_export]
macro_rules! define_handler {
    ($name:ident) => {
        pub struct $name<'a> {
            pub common: &'a $crate::handlers::common::CommonUtils,
        }

        $crate::impl_handler!($name<'a>);
    };
}
