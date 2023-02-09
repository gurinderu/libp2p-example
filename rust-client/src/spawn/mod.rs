use std::future::Future;

#[cfg(feature = "wasm")]
#[path = "rt_wasm_bindgen/mod.rs"]
mod imp;
#[cfg(not(feature = "wasm"))]
#[path = "rt_tokio/mod.rs"]
mod imp;

#[inline(always)]
pub fn spawn_local<F>(f: F)
    where
        F: Future<Output = ()> + 'static,
{
    imp::spawn_local(f);
}
