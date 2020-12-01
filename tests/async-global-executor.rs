use async_trait::async_trait;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use executor_trait::{Executor, Task};

struct AsyncGlobalExecutor;

struct AsyncGlobalTask(async_global_executor::Task<()>);

#[async_trait]
impl Executor for AsyncGlobalExecutor {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        async_global_executor::block_on(f)
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(AsyncGlobalTask(async_global_executor::spawn(f)))
    }

    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Box<dyn Task> {
        Box::new(AsyncGlobalTask(async_global_executor::spawn_local(f)))
    }

    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        blocking::unblock(f).await
    }
}

#[async_trait(?Send)]
impl Task for AsyncGlobalTask {
    fn detach(self: Box<Self>) {
        self.0.detach();
    }

    async fn cancel(self: Box<Self>) -> Option<()> {
        self.0.cancel().await
    }
}

impl Future for AsyncGlobalTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        Pin::new(&mut self.0).poll(cx)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_async_global_executor() {
        global_executor::init(super::AsyncGlobalExecutor);
        let res = global_executor::block_on(async {
            let r1 = global_executor::spawn(async { 1 + 2 }).await;
            let r2 = global_executor::spawn_local(async { 3 + 4 }).await;
            let r3 = global_executor::spawn_blocking(|| 5 + 6).await;
            r1 + r2 + r3
        });
        assert_eq!(res, 21);
    }
}
