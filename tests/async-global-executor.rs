mod tester;

#[test]
fn test_async_global_executor() {
    tester::test_global_executor(async_global_executor_trait::AsyncGlobalExecutor);
}
