mod tester;

#[test]
fn test_smol() {
    // FIXME: make smol spawn_local work
    tester::test_global_executor(smol_executor_trait::Smol, false);
}
