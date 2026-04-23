use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ciborium::Value;
use hegel::{
    backend::{DataSource, DataSourceError, TestCaseResult, TestRunResult, TestRunner},
    Hegel, Mode, Settings, TestCase, Verbosity,
};

struct NoopDataSource;

impl DataSource for NoopDataSource {
    fn generate(&self, _schema: &Value) -> Result<Value, DataSourceError> {
        Err(DataSourceError::StopTest)
    }

    fn start_span(&self, _label: u64) -> Result<(), DataSourceError> {
        Ok(())
    }

    fn stop_span(&self, _discard: bool) -> Result<(), DataSourceError> {
        Ok(())
    }

    fn new_collection(
        &self,
        _min_size: u64,
        _max_size: Option<u64>,
    ) -> Result<String, DataSourceError> {
        Ok("0".to_string())
    }

    fn collection_more(&self, _collection: &str) -> Result<bool, DataSourceError> {
        Ok(false)
    }

    fn collection_reject(
        &self,
        _collection: &str,
        _why: Option<&str>,
    ) -> Result<(), DataSourceError> {
        Ok(())
    }

    fn new_pool(&self) -> Result<i128, DataSourceError> {
        Ok(0)
    }

    fn pool_add(&self, _pool_id: i128) -> Result<i128, DataSourceError> {
        Ok(0)
    }

    fn pool_generate(&self, _pool_id: i128, _consume: bool) -> Result<i128, DataSourceError> {
        Ok(0)
    }

    fn mark_complete(&self, _status: &str, _origin: Option<&str>) {}

    fn test_aborted(&self) -> bool {
        false
    }
}

struct AssertingRunner {
    saw_run_case: Arc<AtomicBool>,
}

impl TestRunner for AssertingRunner {
    fn run(
        &self,
        settings: &Settings,
        database_key: Option<&str>,
        run_case: &mut dyn FnMut(Box<dyn DataSource>, bool) -> TestCaseResult,
    ) -> TestRunResult {
        assert_eq!(settings.mode_value(), Mode::SingleTestCase);
        assert_eq!(settings.test_cases_value(), 7);
        assert_eq!(settings.verbosity_value(), Verbosity::Debug);
        assert_eq!(settings.seed_value(), Some(9));
        assert!(settings.derandomize_value());
        assert_eq!(database_key, Some("custom-runner-test"));

        self.saw_run_case.store(true, Ordering::SeqCst);
        let result = run_case(Box::new(NoopDataSource), false);
        assert!(matches!(result, TestCaseResult::Valid));

        TestRunResult {
            passed: true,
            failure_message: None,
        }
    }
}

#[test]
fn hegel_can_run_with_custom_runner() {
    let test_body_ran = Arc::new(AtomicBool::new(false));
    let saw_run_case = Arc::new(AtomicBool::new(false));

    Hegel::new({
        let test_body_ran = Arc::clone(&test_body_ran);
        move |_tc: TestCase| {
            test_body_ran.store(true, Ordering::SeqCst);
        }
    })
    .settings(
        Settings::new()
            .mode(Mode::SingleTestCase)
            .test_cases(7)
            .verbosity(Verbosity::Debug)
            .seed(Some(9))
            .derandomize(true),
    )
    .__database_key("custom-runner-test".to_string())
    .run_with_runner(AssertingRunner {
        saw_run_case: Arc::clone(&saw_run_case),
    });

    assert!(saw_run_case.load(Ordering::SeqCst));
    assert!(test_body_ran.load(Ordering::SeqCst));
}
