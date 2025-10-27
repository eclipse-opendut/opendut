#![cfg(feature = "http-source")]
#![allow(non_snake_case)]

use googletest::prelude::*;
use httpmock::prelude::*;
use httpmock::MockServer;
use indoc::indoc;
use viper_rt::common::TestSuiteIdentifier;
use viper_rt::compile::IdentifierFilter;
use viper_rt::events::emitter;
use viper_rt::run::ParameterBindings;
use viper_rt::source::loaders::HttpSourceLoader;
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

#[tokio::test]
async fn test_that_HttpSourceLoader_fetches_a_testsuite_via_http() -> Result<()> {

    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/testsuite.py");
        then.status(200)
            .body(
                indoc!(r#"
                    # VIPER_VERSION = 1.0
                    from viper import unittest
                    
                    class SomeClass(unittest.TestCase):
                        def test_awesomeness(self):
                            print("Awesome!")
                "#)
            );
    });

    let runtime = ViperRuntime::builder()
        .with_source_loader(HttpSourceLoader)
        .build()?;

    let source = Source::try_from_url_str(
        TestSuiteIdentifier::try_from("my-testsuite.py")?,
        &server.url("/testsuite.py")
    )?;

    let suite = runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default())
        .await?.into_suite();

    assert_that!(suite.test_cases(), len(eq(1)));
    assert_that!(suite.test_cases()[0].tests(), len(eq(1)));

    let run = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await;

    assert_that!(run, ok(anything()));

    mock.assert();

    Ok(())
}

#[tokio::test]
async fn test_that_HttpSourceLoader_does_not_support_none_http_urls() -> Result<()> {

    let runtime = ViperRuntime::builder()
        .with_source_loader(HttpSourceLoader)
        .build()?;

    assert_that!(runtime.compile(
        &Source::try_from_url_str(
            TestSuiteIdentifier::try_from("awesome_tests")?,
            "ftp://example.com/testsuite.py"
        )?,
        &mut emitter::drain(),
        &IdentifierFilter::default()
    ).await, err(anything()));

    assert_that!(runtime.compile(
        &Source::try_from_url_str(
            TestSuiteIdentifier::try_from("testsuite")?,
            "file:///testsuite.py"
        )?,
        &mut emitter::drain(),
        &IdentifierFilter::default()
    ).await, err(anything()));

    Ok(())
}
