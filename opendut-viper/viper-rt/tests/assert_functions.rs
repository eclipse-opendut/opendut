use googletest::prelude::*;
use indoc::indoc;
use viper_rt::compile::{Compilation, CompileResult, IdentifierFilter};
use viper_rt::events::emitter;
use viper_rt::run::{Outcome, ParameterBindings, Report};
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

async fn compile_test(runtime: &ViperRuntime, source: &Source) -> CompileResult<Compilation> {
    runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
}

#[tokio::test]
async fn test_assert_equals() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertEquals(self):
                    self.assertEquals("[1,2,3]", "[1,2,3]")
                    self.assertEquals("a", "a")
                    self.assertEquals(1, True)
                    self.assertEquals(None, None)
                    self.assertEquals([1,2,3], [1,2,3])
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));
    
    Ok(())
}

#[tokio::test]
async fn test_assert_equals_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertEquals_with_strings(self):
                    self.assertEquals("[1,2,3]", "[A,B,C]")

                def test_assertEquals_with_bool(self):
                    self.assertEquals(1, False)

                def test_assertEquals_with_numbers(self):
                    self.assertEquals(3, 5)

                def test_assertNotEquals_with_lists(self):
                    self.assertEquals([1,2,3], [1,2,5])
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });
    
    Ok(())
}

#[tokio::test]
async fn test_assert_not_equals() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertNotEquals(self):
                    self.assertNotEquals("[1,2,3]", "[1,2,4]")
                    self.assertNotEquals(5, True)
                    self.assertNotEquals([1,2,3], [1,2,5])
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_not_equals_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertNotEquals_with_strings(self):
                    self.assertNotEquals("[1,2,3]", "[1,2,3]")

                def test_assertNotEquals_with_bool(self):
                    self.assertNotEquals(1, True)

                def test_assertNotEquals_with_list(self):
                    self.assertNotEquals([1,2,3], [1,2,3])
        "#)
    )).await?.into_suite();
    
    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });
    
    Ok(())
}

#[tokio::test]
async fn test_assert_true() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertTrue(self):
                    self.assertTrue(True)
                    self.assertTrue(1 == 1)
                    self.assertTrue(1)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_true_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertTrue_failure(self):
                    self.assertTrue(False)

                def test_assertTrue_failure_with_expression(self):
                    self.assertTrue(1 == 2)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_false() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertFalse(self):
                    self.assertFalse(False)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_false_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertFalse_failure(self):
                    self.assertFalse(True)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_is() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is(self):
                    self.assertIs(1, 1)
                    self.assertIs(None, None)
                    self.assertIs("Hallo", "Hallo")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_is_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_failure_different_types(self):
                    self.assertIs(1, "1")

                def test_assert_is_failure_with_bool(self):
                    self.assertIs(1, True)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}


#[tokio::test]
async fn test_assert_is_not() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_not(self):
                    self.assertIsNot(1, 2)
                    self.assertIsNot(1, True)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_is_not_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_not_failure(self):
                    self.assertIsNot(1, 1)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_is_none() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_none(self):
                    self.assertIsNone(None)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_is_none_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_none_failure(self):
                    self.assertIsNone(0)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_is_not_none() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_not_none(self):
                    self.assertIsNotNone(0)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_is_not_none_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_not_none_failure(self):
                    self.assertIsNotNone(None)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_in() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_in(self):
                    self.assertIn("name", {"name": "Alice", "age": 25})
                    self.assertIn("Hello", "Hello World")
                    self.assertIn(1, [1,2,3])
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_in_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_in_failure_with_dict(self):
                    self.assertIn("pet", {"name": "Alice", "age": 25})

                def test_assert_in_failure_with_strings(self):
                    self.assertIn("Hi", "Hello World")

                def test_assert_in_failure_with_list(self):
                    self.assertIn(4, [1,2,3])
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_not_in() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_not_in(self):
                    self.assertNotIn(7, [1,2,3]);
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_not_in_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_not_in_failure(self):
                    self.assertNotIn(1, [1,2,3]);
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_is_instance() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_instance(self):
                    self.assertIsInstance(1, int)
                    self.assertIsInstance("Test", str)
                    self.assertIsInstance(3.14, float)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_is_instance_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_instance_failure_int_vs_str(self):
                    self.assertIsInstance(1, str)

                def test_assert_is_instance_failure_str_vs_int(self):
                    self.assertIsInstance("Test", int)

                def test_assert_is_instance_failure_float_vs_int(self):
                    self.assertIsInstance(3.14, int)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_is_not_instance() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_not_instance(self):
                    self.assertIsNotInstance(1, str)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_is_not_instance_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_is_not_instance_failure(self):
                    self.assertIsNotInstance(1, int)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_greater() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assertGreater(self):
                    self.assertGreater(2, 1)
                    self.assertGreater('B', 'A')
                    self.assertGreater(2.4, 2.3)
                    self.assertGreater("12345", "1234")

        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_greater_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
                
            class MyTestCase(unittest.TestCase):
                def test_assert_greater_failure_equals(self):
                    self.assertGreater(2, 2)

                def test_assert_greater_failure_with_chars(self):
                    self.assertGreater('a', 'b')

                def test_assert_greater_failure_with_floats(self):
                    self.assertGreater(2.1, 2.3)

                def test_assert_greater_failure_with_str(self):
                    self.assertGreater("123", "1234")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_less() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_less(self):
                    self.assertLess(1, 2)
                    self.assertLess('A', 'B')
                    self.assertLess(2.3, 2.4)
                    self.assertLess("123", "1234")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_less_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_less_failure_with_int(self):
                    self.assertLess(3, 2)

                def test_assert_less_failure_with_chars(self):
                    self.assertLess('C', 'B')

                def test_assert_less_failure_with_floats(self):
                    self.assertLess(2.5, 2.4)

                def test_assert_less_failure_with_str(self):
                    self.assertLess("12345", "1234")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_greater_or_equal() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_greater_or_equal(self):
                    self.assertGreaterOrEqual(2, 2)
                    self.assertGreaterOrEqual('B', 'A')
                    self.assertGreaterOrEqual(2.4, 2.4)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_greater_or_equal_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest
                
            class MyTestCase(unittest.TestCase):
                def test_assert_greater_or_equal_failure_with_chars(self):
                    self.assertGreaterOrEqual('a', 'b')

                def test_assert_greater_or_equal_failure_with_floats(self):
                    self.assertGreaterOrEqual(2.1, 2.3)

                def test_assert_greater_or_equal_failure_with_str(self):
                    self.assertGreaterOrEqual("123", "1234")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}

#[tokio::test]
async fn test_assert_less_or_equal() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_less_or_equal(self):
                    self.assertLessOrEqual(2.1, 2.1)
                    self.assertLessOrEqual(2, 2)
                    self.assertLessOrEqual('A', 'B')
                    self.assertLessOrEqual("123", "1234")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_assert_less_or_equal_failure() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):
                def test_assert_less_or_equal_failure_with_int(self):
                    self.assertLessOrEqual(2, 1)

                def test_assert_less_or_equal_failure_with_chars(self):
                    self.assertLessOrEqual('C', 'B')

                def test_assert_less_or_equal_failure_with_str(self):
                    self.assertLessOrEqual("12345", "1234")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    report.cases.into_iter()
        .flat_map(|case| case.tests)
        .for_each(|test_report| {
            assert_that!(test_report.outcome, eq(Outcome::Failure));
        });

    Ok(())
}


#[tokio::test]
async fn test_fail() -> Result<()> {
    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import unittest

            class MyTestCase(unittest.TestCase):

                def test_fail(self):
                    unittest.fail("Hello, I'm an error!")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Failure));

    Ok(())
}
