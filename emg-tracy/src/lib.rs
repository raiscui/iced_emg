#[cfg(feature = "tracy")]
pub use tracing_tracy;

#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! non_continuous_frame {
    ($name: literal) => {{
        $crate::tracing_tracy::client::Client::running()
            .expect("non_continuous_frame! without a running Client")
            .non_continuous_frame($crate::tracing_tracy::client::frame_name!($name))
    }};
}

#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! non_continuous_frame {
    ($name: literal) => {{}};
}

#[inline]
pub fn frame_mark() {
    #[cfg(feature = "tracy")]
    tracing_tracy::client::frame_mark();
}
#[inline]
pub fn start() {
    #[cfg(feature = "tracy")]
    let _client = tracing_tracy::client::Client::start();
}
