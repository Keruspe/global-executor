mod tester;

#[test]
fn test_async_std() {
    tester::test_global_executor(async_executor_trait::AsyncStd, true);
}
