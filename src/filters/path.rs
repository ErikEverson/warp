//! Path Filters

use std::str::FromStr;

use ::filter::{Cons, Filter, filter_fn, HCons, HList};
use ::reject::{self, Rejection};


/// Create an exact match path `Filter`.
///
/// This will try to match exactly to the current request path segment.
///
/// # Note
///
/// Exact path filters cannot be empty, or contain slashes.
pub fn path(p: &'static str) -> impl Filter<Extract=(), Error=Rejection> + Copy {
    assert!(!p.is_empty(), "exact path segments should not be empty");
    assert!(!p.contains('/'), "exact path segments should not contain a slash: {:?}", p);

    segment(move |seg| {
        trace!("{:?}?: {:?}", p, seg);
        if seg == p {
            Ok(())
        } else {
            Err(reject::not_found())
        }
    })
}

/// Matches the end of a route.
pub fn index() -> impl Filter<Extract=(), Error=Rejection> + Copy {
    filter_fn(move |route| {
        if route.path().is_empty() {
            Ok(())
        } else {
            Err(reject::not_found())
        }
    })
}

/// Create an `Extract` path filter.
///
/// An `Extract` will try to parse a value from the current request path
/// segment, and if successful, the value is returned as the `Filter`'s
/// "extracted" value.
pub fn param<T: FromStr + Send>() -> impl Filter<Extract=Cons<T>, Error=Rejection> + Copy {
    segment(|seg| {
        trace!("param?: {:?}", seg);
        T::from_str(seg)
            .map(|t| HCons(t, ()))
            .map_err(|_| reject::not_found())
    })
}

fn segment<F, U>(func: F) -> impl Filter<Extract=U, Error=Rejection> + Copy
where
    F: Fn(&str) -> Result<U, Rejection> + Copy,
    U: HList + Send,
{
    filter_fn(move |route| {
        let (u, idx) = {
            let seg = route.path()
                .splitn(2, '/')
                .next()
                .expect("split always has at least 1");
            (func(seg)?, seg.len())
        };
        route.set_unmatched_path(idx);
        Ok(u)
    })
}

#[macro_export]
macro_rules! path {
    (@start $first:tt $(/ $tail:tt)*) => ({
        let __p = path!(@segment $first);
        $(
        let __p = $crate::Filter::and(__p, path!(@segment $tail));
        )*
        __p
    });
    (@segment $param:ty) => (
        $crate::path::param::<$param>()
    );
    (@segment $s:expr) => (
        $crate::path($s)
    );
    ($($pieces:tt)*) => (
        path!(@start $($pieces)*)
    );
}
