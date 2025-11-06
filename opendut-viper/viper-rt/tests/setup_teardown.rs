use googletest::prelude::*;
use indoc::indoc;
use opendut_viper_rt::compile::{Compilation, CompileResult, IdentifierFilter};
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::run::{Outcome, ParameterBindings, Report};
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;

async fn compile_test(runtime: &ViperRuntime, source: &Source) -> CompileResult<Compilation> {
    runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
}

#[tokio::test]
async fn test_setup_class() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"# VIPER_VERSION = 1.0
            from viper import unittest
            
            class MyTestCase(unittest.TestCase): 
            
                @classmethod
                def setUpClass(cls):
                    cls.text = "Hello World!"
            
                def test_print(self):
                    print(self.text)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
    report.cases[0].tests.iter().for_each(|test| {
        test.output.iter().for_each(|line| { print!("{line}"); })
    });

    assert_that!(report.outcome(), eq(Outcome::Success));

    assert_that!(report.cases[0].tests[0].output, container_eq([
        String::from("Hello World!"),
        String::from("\n"),
    ]));

    Ok(())
}

#[tokio::test]
async fn test_teardown_class() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            calls = []

            class MyTestCase(unittest.TestCase):

                @classmethod
                def tearDownClass(cls):
                    calls.append("tearDownClass MyTestCase")

                def setUp(self):
                    pass

            class TestTeardown(unittest.TestCase):
                def test_teardown_called(self):
                    print(calls)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    assert_that!(report.cases[1].tests[0].output, container_eq([
        String::from("['tearDownClass MyTestCase']"),
        String::from("\n"),
    ]));

    Ok(())
}

#[tokio::test]
async fn test_setup() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):

                def setUp(self):
                    self.x = 3

                def test_assert_1(self):
                    self.x = self.x + 1
                    self.assertEquals(1, 1)

                def test_assert_2(self):
                    self.assertEquals(3, self.x)
        "#)
    )).await?.into_suite();

    let result = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(result.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_teardown() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def tearDown(self):
                    self.x = 3

                def test_assert_1(self):
                    self.x = 1
                    self.assertEquals(1, self.x)

                def test_assert_2(self):
                    self.assertEquals(3, self.x)
        "#)
    )).await?.into_suite();

    let result = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(result.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_calls_order() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"# VIPER_VERSION = 1.0
            from viper import unittest

            calls = []

            class MyTestCase(unittest.TestCase):

                @classmethod
                def setUpClass(cls):
                    calls.append("setUpClass MyTestCase")

                @classmethod
                def tearDownClass(cls):
                    calls.append("tearDownClass MyTestCase")

                def setUp(self):
                    calls.append("setUp MyTestCase")  # <-- NEU

                def tearDown(self):
                    calls.append("tearDown MyTestCase")  # <-- NEU

                def test_1(self):
                    calls.append("test_1")

                def test_2(self):
                    calls.append("test_2")


            class MyTestCase2(unittest.TestCase):

                @classmethod
                def setUpClass(cls):
                    calls.append("setUpClass MyTestCase2")

                @classmethod
                def tearDownClass(cls):
                    calls.append("tearDownClass MyTestCase2")

                def setUp(self):
                    calls.append("setUp MyTestCase2")  # <-- NEU

                def tearDown(self):
                    calls.append("tearDown MyTestCase2")  # <-- NEU

                def test_3(self):
                    calls.append("test_3")


            class TestOrder(unittest.TestCase):

                def test_result(self):
                    print(calls)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    assert_that!(report.cases[2].tests[0].output, container_eq([
        String::from("\
            ['setUpClass MyTestCase', \
            'setUp MyTestCase', \
            'test_1', \
            'tearDown MyTestCase', \
            'setUp MyTestCase', \
            'test_2', \
            'tearDown MyTestCase', \
            'tearDownClass MyTestCase', \
            'setUpClass MyTestCase2', \
            'setUp MyTestCase2', \
            'test_3', \
            'tearDown MyTestCase2', \
            'tearDownClass MyTestCase2']\
        "),
        String::from("\n"),
    ]));

    Ok(())
}
