use googletest::prelude::*;
use indoc::indoc;
use viper_rt::compile::{Compilation, CompileResult, IdentifierFilter, ParameterDescriptor, ParameterInfo, ParameterName};
use viper_rt::events::emitter;
use viper_rt::run::{BindingValue, ParameterBindings, Report};
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

async fn compile_test(runtime: &ViperRuntime, source: &Source) -> CompileResult<Compilation> {
    runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
}

#[tokio::test]
async fn test_boolean_parameters() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (_, parameters, suite) = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import *
            
            FOO = parameters.BooleanParameter("foo")
            BAR = parameters.BooleanParameter("bar")
            FUBAR = parameters.BooleanParameter("fubar", default=True, display_name="The FUBAR", description="This is an awesome boolean parameter.")
            
            class MyTestCase(unittest.TestCase):
                def test_by_string(self):
                    self.assertTrue(self.parameters.get("foo"))
                    self.assertFalse(self.parameters.get("bar"))
                    self.assertTrue(self.parameters.get("fubar"))
                def test_by_descriptor(self):
                    self.assertTrue(self.parameters.get(FOO))
                    self.assertFalse(self.parameters.get(BAR))
                    self.assertTrue(self.parameters.get(FUBAR))
        "#)
    )).await?.split();

    {
        let parameters = parameters.iter().cloned().collect::<Vec<_>>();
        assert_that!(parameters, container_eq([
            ParameterDescriptor::BooleanParameter {
                key: String::from("FOO"),
                name: ParameterName::try_from("foo")?,
                info: ParameterInfo::default(),
                default: None,
            },
            ParameterDescriptor::BooleanParameter {
                key: String::from("BAR"),
                name: ParameterName::try_from("bar")?,
                info: ParameterInfo::default(),
                default: None,
            },
            ParameterDescriptor::BooleanParameter {
                key: String::from("FUBAR"),
                name: ParameterName::try_from("fubar")?,
                info: ParameterInfo { display_name: Some(String::from("The FUBAR")), description: Some(String::from("This is an awesome boolean parameter.")) },
                default: Some(true),
            }
        ]));
    }

    let mut bindings = ParameterBindings::from(parameters);

    bindings.bind(&ParameterName::try_from("foo")?, BindingValue::BooleanValue(true))?;
    bindings.bind(&ParameterName::try_from("bar")?, BindingValue::BooleanValue(false))?;

    let bindings = bindings.complete()?;

    let report = runtime.run(suite, bindings, &mut emitter::drain()).await?;

    assert!(report.is_success());

    Ok(())
}

#[tokio::test]
async fn test_number_parameters() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (_, parameters, suite) = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import *
            
            FOO = parameters.NumberParameter("foo")
            BAR = parameters.NumberParameter("bar")
            FUBAR = parameters.NumberParameter(
                "fubar",
                default=1207,
                max=42,
                min=-42,
                display_name="The FUBAR",
                description="This is an awesome number parameter."
            )

            class MyTestCase(unittest.TestCase):
                def test_by_string(self):
                    self.assertEquals(self.parameters.get("foo"), 8121)
                    self.assertEquals(self.parameters.get("bar"), 2703)
                    self.assertEquals(self.parameters.get("fubar"), 1207)
                def test_by_descriptor(self):
                    self.assertEquals(self.parameters.get(FOO), 8121)
                    self.assertEquals(self.parameters.get(BAR), 2703)
                    self.assertEquals(self.parameters.get(FUBAR), 1207)
        "#)
    )).await?.split();

    {
        let parameters = parameters.iter().cloned().collect::<Vec<_>>();
        assert_that!(parameters, container_eq([
            ParameterDescriptor::NumberParameter {
                key: String::from("FOO"),
                name: ParameterName::try_from("foo")?,
                info: ParameterInfo::default(),
                default: None,
                min: i64::MIN,
                max: i64::MAX,
            },
            ParameterDescriptor::NumberParameter {
                key: String::from("BAR"),
                name: ParameterName::try_from("bar")?,
                info: ParameterInfo::default(),
                default: None,
                min: i64::MIN,
                max: i64::MAX,
            },
            ParameterDescriptor::NumberParameter {
                key: String::from("FUBAR"),
                name: ParameterName::try_from("fubar")?,
                info: ParameterInfo { display_name: Some(String::from("The FUBAR")), description: Some(String::from("This is an awesome number parameter.")) },
                default: Some(1207),
                min: -42,
                max: 42,
            }
        ]));
    }

    let mut bindings = ParameterBindings::from(parameters);

    bindings.bind(&ParameterName::try_from("foo")?, BindingValue::NumberValue(8121))?;
    bindings.bind(&ParameterName::try_from("bar")?, BindingValue::NumberValue(2703))?;

    let bindings = bindings.complete()?;

    let report = runtime.run(suite, bindings, &mut emitter::drain()).await?;

    assert!(report.is_success());

    Ok(())
}

#[tokio::test]
async fn test_text_parameters() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (_, parameters, suite) = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import *
            
            NAME = parameters.TextParameter("name")
            FUBAR = parameters.TextParameter(
                "fubar",
                default="Jessica",
                max=7,
                display_name="The FUBAR",
                description="This is an awesome text parameter."
            )
            
            class MyTestCase(unittest.TestCase):
                def test_by_string(self):
                    self.assertEquals(self.parameters.get("name"), "Elmar")
                    self.assertEquals(self.parameters.get("fubar"), "Jessica")
                def test_by_descriptor(self):
                    self.assertEquals(self.parameters.get(NAME), "Elmar")
                    self.assertEquals(self.parameters.get(FUBAR), "Jessica")
        "#)
    )).await?.split();

    {
        let parameters = parameters.iter().cloned().collect::<Vec<_>>();
        assert_that!(parameters, container_eq([
            ParameterDescriptor::TextParameter {
                key: String::from("NAME"),
                name: ParameterName::try_from("name")?,
                info: ParameterInfo::default(),
                default: None,
                max: u32::MAX,
            },
            ParameterDescriptor::TextParameter {
                key: String::from("FUBAR"),
                name: ParameterName::try_from("fubar")?,
                info: ParameterInfo { display_name: Some(String::from("The FUBAR")), description: Some(String::from("This is an awesome text parameter.")) },
                default: Some(String::from("Jessica")),
                max: 7,
            },
        ]));
    }

    let mut bindings = ParameterBindings::from(parameters);

    bindings.bind(&ParameterName::try_from("name")?, BindingValue::TextValue(String::from("Elmar")))?;

    let bindings = bindings.complete()?;

    let report = runtime.run(suite, bindings, &mut emitter::drain()).await?;

    assert!(report.is_success());

    Ok(())
}
