mod tester;

#[test]
fn test_tokio() {
    // FIXME: make tokio spawn_local work
    tester::test_global_executor(tokio_executor_trait::Tokio::default(), false);
}
