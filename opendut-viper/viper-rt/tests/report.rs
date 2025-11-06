use googletest::prelude::*;
use indoc::indoc;
use std::path::PathBuf;
use opendut_viper_rt::compile::IdentifierFilter;
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::run::{ParameterBindings, Report, ReportProperty, ReportPropertyValue};
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;

#[tokio::test]
async fn test_report_properties() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (_, _, suite) = runtime.compile(
        &Source::embedded(
            indoc!(r#"
                # VIPER_VERSION = 1.0
                from viper import *

                class MyTestCase(unittest.TestCase):
                    def test_report_properties(self):
                        self.report.property("number", 42)
                        self.report.property("string", "Hello World")
                        self.report.properties(
                            kw_number=42,
                            kw_string="Hello World"
                        )
                    def test_report_properties_2(self):
                        self.report.property("number", 73)
                        self.report.property("string", "Bye Bye")
            "#)
        ),
        &mut emitter::drain(),
        &IdentifierFilter::default(),
    ).await?.split();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.is_success(), eq(true));

    assert_that!(report.cases[0].tests[0].properties, len(eq(4)));
    assert_that!(report.cases[0].tests[1].properties, len(eq(2)));

    assert_that!(report.cases[0].tests[0].properties[0], eq(&ReportProperty { name: String::from("number"), value: ReportPropertyValue::Number(42) }));
    assert_that!(report.cases[0].tests[0].properties[1], eq(&ReportProperty { name: String::from("string"), value: ReportPropertyValue::String(String::from("Hello World")) }));
    assert_that!(report.cases[0].tests[0].properties[2], eq(&ReportProperty { name: String::from("kw_number"), value: ReportPropertyValue::Number(42) }));
    assert_that!(report.cases[0].tests[0].properties[3], eq(&ReportProperty { name: String::from("kw_string"), value: ReportPropertyValue::String(String::from("Hello World")) }));

    assert_that!(report.cases[0].tests[1].properties[0], eq(&ReportProperty { name: String::from("number"), value: ReportPropertyValue::Number(73) }));
    assert_that!(report.cases[0].tests[1].properties[1], eq(&ReportProperty { name: String::from("string"), value: ReportPropertyValue::String(String::from("Bye Bye")) }));

    Ok(())
}

#[tokio::test]
async fn test_report_file_properties() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (_, _, suite) = runtime.compile(
        &Source::embedded(
            indoc!(r#"
                # VIPER_VERSION = 1.0
                from viper import *

                class MyTestCase(unittest.TestCase):
                    def test_report_properties(self):
                        self.report.file("/a/b")
                        self.report.files("/c/d/e/f", "/g/h/i/j")
                    def test_report_properties_2(self):
                        self.report.file("/a/b")
                        self.report.file("/c/d/e/f")
            "#)
        ),
        &mut emitter::drain(),
        &IdentifierFilter::default(),
    ).await?.split();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.is_success(), eq(true));

    assert_that!(report.cases[0].tests[0].properties, len(eq(3)));
    assert_that!(report.cases[0].tests[1].properties, len(eq(2)));

    assert_that!(report.cases[0].tests[0].properties[0], eq(&ReportProperty { name: String::from("/a/b"), value: ReportPropertyValue::File(PathBuf::from("/a/b")) }));
    assert_that!(report.cases[0].tests[0].properties[1], eq(&ReportProperty { name: String::from("/c/d/e/f"), value: ReportPropertyValue::File(PathBuf::from("/c/d/e/f")) }));
    assert_that!(report.cases[0].tests[0].properties[2], eq(&ReportProperty { name: String::from("/g/h/i/j"), value: ReportPropertyValue::File(PathBuf::from("/g/h/i/j")) }));

    assert_that!(report.cases[0].tests[1].properties[0], eq(&ReportProperty { name: String::from("/a/b"), value: ReportPropertyValue::File(PathBuf::from("/a/b")) }));
    assert_that!(report.cases[0].tests[1].properties[1], eq(&ReportProperty { name: String::from("/c/d/e/f"), value: ReportPropertyValue::File(PathBuf::from("/c/d/e/f")) }));

    Ok(())
}
