use googletest::prelude::*;
use indoc::indoc;
use viper_rt::compile::IdentifierFilter;
use viper_rt::events::emitter;
use viper_rt::run::{Outcome, ParameterBindings, Report};
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

#[tokio::test]
async fn test_running_specific_case() -> Result<()> {
    let runtime = ViperRuntime::default();

    let filter = "<embedded>::MySucceedingTestCase";
    let identifier_filter = IdentifierFilter::parse(filter);

    let suite = runtime.compile(
        &Source::embedded(
            indoc!(r#"
                # VIPER_VERSION = 1.0
                from viper import unittest

                class MyFailingTestCase(unittest.TestCase):
                    def test_failure(self):
                        self.fail("BOOM")

                class MySucceedingTestCase(unittest.TestCase):
                    def test_success(self):
                        print("Success!")
            "#)
        ),
        &mut emitter::drain(),
        &identifier_filter,
    ).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_running_specific_test() -> Result<()> {
    let runtime = ViperRuntime::default();

    let filter = "<embedded>::MyTestCase::test_success";
    let identifier_filter = IdentifierFilter::parse(filter);

    let suite = runtime.compile(
        &Source::embedded(
            indoc!(r#"
                # VIPER_VERSION = 1.0
                from viper import unittest

                class MyTestCase(unittest.TestCase):
                    def test_failure(self):
                        self.fail("BOOM")

                    def test_success(self):
                        print("Success!")
            "#)
        ),
        &mut emitter::drain(),
        &identifier_filter,
    ).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}
