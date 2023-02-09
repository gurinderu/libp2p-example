#[inline(always)]
pub(super) fn spawn_local<F>(f: F)
    where
        F: Future<Output=()> + Send + 'static,
{
    tokio::spawn(f);
}