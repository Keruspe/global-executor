//! Configure a global executor you can reuse everywhere

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use async_channel::Receiver;
use core::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
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
    Task {
        inner,
        recv: recv.into(),
    }
}

pub fn spawn_local<T: 'static>(future: impl Future<Output = T> + 'static) -> Task<T> {
    let (send, recv) = async_channel::bounded(1);
    let inner = EXECUTOR.get().unwrap().spawn_local(Box::pin(async move {
        drop(send.send(future.await).await);
    }));
    Task {
        inner,
        recv: recv.into(),
    }
}

pub async fn spawn_blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (send, recv) = async_channel::bounded(1);
    EXECUTOR
        .get()
        .unwrap()
        .spawn_blocking(Box::new(|| {
            let res = f();
            crate::spawn(async move {
                drop(send.send(res).await);
            })
            .detach();
        }))
        .await;
    recv.recv().await.unwrap()
}

pub struct Task<T> {
    inner: Box<dyn executor_trait::Task>,
    recv: ReceiverWrapper<T>,
}

impl<T: 'static> Task<T> {
    pub fn detach(self) {
        self.inner.detach();
    }

    pub async fn cancel(self) -> Option<T> {
        self.inner.cancel().await?;
        Some(self.recv.await)
    }
}

impl<T> fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task").finish()
    }
}

impl<T: 'static> Future for Task<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        Pin::new(&mut self.recv).poll(cx)
    }
}

struct ReceiverWrapper<T> {
    recv: Receiver<T>,
    recv_fut: Option<Pin<Box<dyn Future<Output = T>>>>,
}

impl<T: 'static> Future for ReceiverWrapper<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        if self.recv_fut.is_none() {
            let recv = self.recv.clone();
            self.recv_fut = Some(Box::pin(async move { recv.recv().await.unwrap() }));
        }
        match self.recv_fut.as_mut().unwrap().as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => {
                self.recv_fut = None;
                Poll::Ready(t)
            }
        }
    }
}

impl<T> From<Receiver<T>> for ReceiverWrapper<T> {
    fn from(recv: Receiver<T>) -> Self {
        Self {
            recv,
            recv_fut: None,
        }
    }
}
