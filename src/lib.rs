pub use axum_distributed_routing_macros::*;
pub use inventory;

use axum::routing::Router;

/// A trait for defining a route. All routes must implement this trait
///
/// A route is not necessarily an HTTP route, it could be anything that can be
/// attached to a router (for example, a nested router).
///
/// You typically use the `route_group!` macro to define a route struct that
/// implements this trait, but you can also do it manually.
/// Be sure to create a function `new` (you can find the signature in the
/// `route_group!` macro)
pub trait Route {
    type State: Clone + Send + Sync + 'static;

    fn attach(&self, router: Router<Self::State>, level: usize) -> Router<Self::State>;

    fn path(&self) -> &'static str;
}

/// Define a route group
///
/// A route group is used to group routes together. It is then used in the
/// `create_router!` macro. It is effectively a type that implements the
/// `Route` trait.
///
/// You can also use this macro to define a nested route group, simply add the
/// parent group and the subpath as the third and fourth arguments
#[macro_export]
macro_rules! route_group {
    ($name:ident, $type:ty, $parent:ident, $path:literal) => {
        $crate::route_group!($name, $type);
        $crate::inventory::submit!($parent::new($path, |router, level| {
            router.nest($path, $crate::create_router::<$name>(level + 4))
        }));
    };
    ($name:ident, $type:ty) => {
        #[derive(Copy, Clone, Debug)]
        struct $name {
            path: &'static str,
            handler: fn(axum::routing::Router<$type>, usize) -> axum::routing::Router<$type>,
        }

        impl $name {
            pub const fn new(
                path: &'static str,
                handler: fn(axum::routing::Router<$type>, usize) -> axum::routing::Router<$type>,
            ) -> Self {
                Self { path, handler }
            }
        }

        impl $crate::Route for $name {
            type State = $type;

            fn attach(
                &self,
                router: axum::routing::Router<$type>,
                level: usize,
            ) -> axum::routing::Router<$type> {
                (self.handler)(router, level)
            }

            fn path(&self) -> &'static str {
                self.path
            }
        }

        $crate::inventory::collect!($name);
    };
}

/// Returns an iterator over the routes of the provided group
#[macro_export]
macro_rules! routes {
    ($type:ty) => {
        $crate::inventory::iter::<$type>
    };
}

/// Creates a router from the provided group
#[macro_export]
macro_rules! create_router {
    ($type:ty) => {
        $crate::create_router::<$type>(0)
    };
}

#[doc(hidden)]
pub fn create_router<T: Route + 'static>(level: usize) -> Router<T::State>
where
    inventory::iter<T>: IntoIterator<Item = &'static T>,
{
    let mut router = Router::new();
    for route in inventory::iter::<T> {
        router = route.attach(router, level);
    }
    router
}

// TODO: tests
