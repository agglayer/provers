use std::panic::UnwindSafe;

use eyre::Context as _;
use futures::{Future, FutureExt};
use tracing::error;

#[derive(Debug, thiserror::Error)]
#[error("Panic occurred during SP1 call: {message}")]
pub struct Sp1PanicError {
    message: String,
}

impl From<Box<dyn std::any::Any + Send>> for Sp1PanicError {
    fn from(original_error: Box<dyn std::any::Any + Send>) -> Self {
        let message = match original_error.downcast::<String>() {
            Ok(message) => *message,
            Err(error) => match error.downcast::<&str>() {
                Ok(message) => message.to_string(),
                Err(_) => "<unknown message type>".to_string(),
            },
        };
        Sp1PanicError { message }
    }
}

/// Run a fast sp1 function call, catching panics.
///
/// If `f` is slow, then it will block the async runtime.
/// In that case, consider using `sp1_blocking` or `sp1_block_in_place` instead.
pub fn sp1_fast<R>(f: impl UnwindSafe + FnOnce() -> R) -> eyre::Result<R> {
    std::panic::catch_unwind(f).map_err(|error| {
        let error = eyre::Report::from(Sp1PanicError::from(error));
        tracing::error!(?error, "SP1 call panicked");
        error
    })
}

/// Run a slow sp1 function call, catching panics.
///
/// This will run `f` on the blocking thread pool, to avoid blocking the async
/// runtime. If `f` is fast, consider using `sp1_fast` instead.
pub async fn sp1_blocking<F, R>(f: F) -> eyre::Result<R>
where
    F: 'static + Send + UnwindSafe + FnOnce() -> R,
    R: 'static + Send,
{
    tokio::task::spawn_blocking(|| sp1_fast(f))
        .await
        .context("Failed running blocking task for SP1 call")?
}

/// Run a slow sp1 function call, catching panics.
///
/// This will run `f` on this thread and move the tasks to another async thread,
/// to avoid blocking the async runtime. If you have the required `Send +
/// 'static` bounds, consider using `sp1_blocking` instead.
pub fn sp1_block_in_place<F, R>(f: F) -> eyre::Result<R>
where
    F: UnwindSafe + FnOnce() -> R,
{
    tokio::task::block_in_place(|| sp1_fast(f))
        .context("Failed running blocking task in place for SP1 call")
}

/// Run an async sp1 function call, catching panics.
pub async fn sp1_async<F, R>(f: F) -> eyre::Result<R>
where
    F: UnwindSafe + Future<Output = R>,
{
    f.catch_unwind().await.map_err(|error| {
        let error = eyre::Report::from(Sp1PanicError::from(error));
        tracing::error!(?error, "SP1 call panicked");
        error
    })
}
