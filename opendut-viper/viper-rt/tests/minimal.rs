use googletest::prelude::*;
use indoc::indoc;
use viper_rt::common::TestSuiteIdentifier;
use viper_rt::events::emitter;
use viper_rt::run::{Outcome, ParameterBindings, Report};
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

#[tokio::test]
async fn test_that_compile_and_run_properly_work() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
            
            class MyTestCase(unittest.TestCase):
                """This is my minimal test case."""
                def test_awesomeness(self):
                    print("Awesome!")
        "#)
    ), &mut emitter::drain()).await?.into_suite();

    assert_that!(suite.test_cases(), len(eq(1)));
    assert_that!(suite.test_cases()[0].name(), eq(String::from("MyTestCase")));
    assert_that!(suite.test_cases()[0].description(), some(eq(&String::from("This is my minimal test case."))));
    assert_that!(suite.test_cases()[0].tests(), len(eq(1)));
    assert_that!(suite.test_cases()[0].tests()[0].name(), eq(String::from("test_awesomeness")));

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.cases, len(eq(1)));
    assert_that!(report.cases[0].tests, len(eq(1)));

    assert_that!(report.cases[0].tests[0].outcome(), eq(Outcome::Success));
    assert_that!(report.cases[0].outcome(), eq(Outcome::Success));
    assert_that!(report.outcome(), eq(Outcome::Success));

    assert_that!(report.name, eq(&String::from("<embedded>")));
    assert_that!(report.cases[0].name, eq(&String::from("<embedded>::MyTestCase")));
    assert_that!(report.cases[0].tests[0].identifier, eq(&String::from("<embedded>::MyTestCase::test_awesomeness")));

    Ok(())
}

#[cfg(feature = "file-source")]
#[tokio::test]
async fn test_that_compile_and_run_work_for_a_file_source() -> Result<()> {

    let runtime = ViperRuntime::builder()
        .with_source_loader(viper_rt::source::loaders::SimpleFileSourceLoader)
        .build()?;

    let path = std::path::absolute(std::path::PathBuf::from("tests/minimal.py"))?;
    let source = Source::try_from_path(TestSuiteIdentifier::try_from("minimal")?, &path)?;

    let suite = runtime.compile(&source, &mut emitter::drain()).await?.into_suite();

    assert_that!(runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await, ok(anything()));

    Ok(())
}

#[tokio::test]
async fn test_that_classes_are_ignored_derived_from_test_case() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
            
            class SomeClass(unittest.TestCase):
                def test_awesomeness(self):
                    print("Awesome!")
                    
            class AnotherClass():
                def test_awesomeness(self):
                    print("Awesome!")
        "#)
    ), &mut emitter::drain()).await?.into_suite();

    assert_that!(suite.test_cases(), len(eq(1)));
    assert_that!(suite.test_cases()[0].tests(), len(eq(1)));
    
    assert_that!(runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await, ok(anything()));

    Ok(())
}

#[tokio::test]
async fn test_that_functions_not_prefixed_with_test_are_ignored() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
            
            class MyTestCase(unittest.TestCase):
        
                def test_awesomeness(self):
                    print("Awesome!")
                    
                def util(self):
                    pass
                    
                def test_sadness(self):
                    print("Awesome!")
        "#)
    ), &mut emitter::drain()).await?.into_suite();

    assert_that!(suite.test_cases(), len(eq(1)));
    assert_that!(suite.test_cases()[0].tests(), len(eq(2)));

    assert_that!(runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await, ok(anything()));

    Ok(())
}

#[tokio::test]
async fn test_that_results_reflect_the_outcome_of_a_test() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
            
            class MyTestCase(unittest.TestCase):
                def test_failure(self):
                    raise Exception("Boom!")
        "#)
    ), &mut emitter::drain()).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Failure));
    assert_that!(report.cases[0].tests[0].outcome, eq(Outcome::Failure));

    Ok(())
}

#[tokio::test]
async fn test_that_stdout_and_stderr_are_captured() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
            import sys
            
            class MyTestCase(unittest.TestCase): 
                def test_hello(self):
                    print("Hello World")
                    print("Bye Bye")
                    print("Boom!", file=sys.stderr)
                    
                def test_nothing(self):
                    pass
        "#)
    ), &mut emitter::drain()).await?.into_suite();

    let result  = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(result.cases[0].tests[0].output, container_eq([
        String::from("Hello World"),
        String::from("\n"),
        String::from("Bye Bye"),
        String::from("\n"),
        String::from("Boom!"),
        String::from("\n"),
    ]));

    assert_that!(result.cases[0].tests[1].output, is_empty());

    Ok(())
}

#[tokio::test]
async fn test_that_compilation_fails_if_no_version_is_given() -> Result<()> {

    let runtime = ViperRuntime::default();

    let result = runtime.compile(&Source::embedded(
        indoc!(r#"
            from viper import unittest
            
            class MyTestCase(unittest.TestCase): 
                def test_hello(self):
                    pass
        "#)
    ), &mut emitter::drain()).await;

    assert_that!(result.is_err(), eq(true));

    Ok(())
}
