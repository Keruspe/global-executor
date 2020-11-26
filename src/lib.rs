use async_channel::Receiver;
use core::future::Future;
use executor_trait::Executor;
use once_cell::sync::OnceCell;

static EXECUTOR: OnceCell<Box<dyn Executor + Send + Sync>> = OnceCell::new();

pub fn init(executor: impl Executor + Send + Sync + 'static) {
    EXECUTOR.set(Box::new(executor)).map_err(|_| ()).unwrap();
}

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    let (send, recv) = async_channel::bounded(1);
    let inner = EXECUTOR.get().unwrap().spawn(Box::pin(async move {
        drop(send.send(future.await).await);
    }));
    Task { inner, recv }
}

pub fn spawn_local<T: 'static>(future: impl Future<Output = T> + 'static) -> Task<T> {
    let (send, recv) = async_channel::bounded(1);
    let inner = EXECUTOR.get().unwrap().spawn_local(Box::pin(async move {
        drop(send.send(future.await).await);
    }));
    Task { inner, recv }
}

pub async fn spawn_blocking(f: impl FnOnce() + Send + 'static) {
    EXECUTOR.get().unwrap().spawn_blocking(Box::new(f)).await
}

pub struct Task<T> {
    inner: Box<dyn executor_trait::Task>,
    recv: Receiver<T>,
}

impl<T> Task<T> {
    pub fn detach(self) {
        self.inner.detach();
    }

    pub async fn cancel(self) -> Option<T> {
        self.inner.cancel().await?;
        self.recv.recv().await.ok()
    }
}
