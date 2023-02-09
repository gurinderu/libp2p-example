

#[inline(always)]
pub(super) fn spawn_local<F>(f: F)
    where
        F: Future<Output = ()> + 'static,
{
    match LocalHandle::try_current() {
        Some(m) => {
            // If within a Yew runtime, use a local handle increases the local task count.
            m.spawn_local(f);
        }
        None => {
            tokio::task::spawn_local(f);
        }
    }
}