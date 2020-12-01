use async_trait::async_trait;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use executor_trait::{Executor, Task};

mod tester;

struct AsyncStd;

struct AsyncStdTask(async_std::task::JoinHandle<()>);

#[async_trait]
impl Executor for AsyncStd {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        async_std::task::block_on(f)
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(AsyncStdTask(async_std::task::spawn(f)))
    }

    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Box<dyn Task> {
        Box::new(AsyncStdTask(async_std::task::spawn_local(f)))
    }

    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        async_std::task::spawn_blocking(f).await
    }
}

#[async_trait(?Send)]
impl Task for AsyncStdTask {
    fn detach(self: Box<Self>) {
        // async-std detaches task on drop
        drop(self)
    }

    async fn cancel(self: Box<Self>) -> Option<()> {
        self.0.cancel().await
    }
}

impl Future for AsyncStdTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        Pin::new(&mut self.0).poll(cx)
    }
}

#[test]
fn test_async_global_executor() {
    tester::test_global_executor(AsyncStd);
}
