use futures::Future;

#[inline(always)]
pub(super) fn spawn_local<F>(f: F)
    where
        F: Future<Output=()> + Send + 'static,
{
    //tokio::spawn(f);
    tokio::task::spawn(f);
}