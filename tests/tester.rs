use executor_trait::Executor;

pub fn test_global_executor(executor: impl Executor + Send + Sync + 'static) {
    global_executor::init(executor);
    let res = global_executor::block_on(async {
        let r1 = global_executor::spawn(async { 1 + 2 }).await;
        let r2 = global_executor::spawn_local(async { 3 + 4 }).await;
        let r3 = global_executor::spawn_blocking(|| 5 + 6).await;
        r1 + r2 + r3
    });
    assert_eq!(res, 21);
}
