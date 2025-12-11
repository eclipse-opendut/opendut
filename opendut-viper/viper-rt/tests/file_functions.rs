#![cfg(feature = "events")]

use googletest::prelude::*;
use indoc::indoc;
use std::fs;
use tempfile::NamedTempFile;
use opendut_viper_rt::compile::{Compilation, CompileResult, IdentifierFilter};
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::run::{Outcome, ParameterBindings, Report};
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;

async fn compile_test(runtime: &ViperRuntime, source: &Source) -> CompileResult<Compilation> {
    runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
}

#[tokio::test]
async fn test_open_file() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import file, unittest

            class MyTestCase(unittest.TestCase):

                def test_file_open(self):
                    f = open("tests/test.txt")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_open_file_that_does_not_exist() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import file, unittest

            class MyTestCase(unittest.TestCase):

                def test_file_open_fail(self):
                    f = open("tests/i_dont_exist.txt")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Failure));

    Ok(())
}


#[tokio::test]
async fn test_iter_file() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import file, unittest

            class MyTestCase(unittest.TestCase):

                def test_file_open(self):
                    with open("tests/test.txt") as file:
                        lines = [line.rstrip() for line in file]

                def test_for_loop(self):
                    f = open("tests/test.txt")
                    for line in f:
                        print(line)
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
    
    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_read_write() -> Result<()> {

    let runtime = ViperRuntime::default();

    let tmp_file = NamedTempFile::new()?;
    let path = tmp_file.path().to_str().unwrap();

    let suite = compile_test(&runtime, &Source::embedded(
        format!(r#"# VIPER_VERSION = 1.0
from viper import unittest

class MyTestCase(unittest.TestCase):

    def test_read_and_write(self):
        f = open("tests/test.txt")
        content = f.read()
        
        tmp_file = open("{path}", "w+")
        tmp_file.write(content)
"#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    let content_in_tmp_file = fs::read_to_string(path)?;
    let expected = String::from("1\n22\n333\n");

    assert_eq!(content_in_tmp_file, expected);
    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_readline_write() -> Result<()> {

    let runtime = ViperRuntime::default();

    let tmp_file = NamedTempFile::new()?;
    let path = tmp_file.path().to_str().unwrap();

    let suite = compile_test(&runtime, &Source::embedded(
        format!(r#"# VIPER_VERSION = 1.0
from viper import unittest

class MyTestCase(unittest.TestCase):

    def test_readline(self):
        f = open("tests/test.txt")
        firstReadline = f.readline()
        secondReadline = f.readline()
        
        tmp_file = open("{path}", "w+")
        tmp_file.write(firstReadline)
        tmp_file.write(secondReadline)

"#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    let content_in_tmp_file = fs::read_to_string(path)?;
    let expected = String::from("1\n22\n");

    assert_eq!(content_in_tmp_file, expected);
    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_readlines_write() -> Result<()> {

    let runtime = ViperRuntime::default();

    let tmp_file = NamedTempFile::new()?;
    let path = tmp_file.path().to_str().unwrap();

    let suite = compile_test(&runtime, &Source::embedded(
        format!(r#"# VIPER_VERSION = 1.0
from viper import unittest

class MyTestCase(unittest.TestCase):

    def test_readlines(self):
        f = open("tests/test.txt")
        content = f.readlines()
        
        tmp_file = open("{path}", "w+")
        tmp_file.writelines(content)

"#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    let content_in_tmp_file = fs::read_to_string(path)?;
    let expected = String::from("1\n22\n333\n");

    assert_eq!(content_in_tmp_file, expected);
    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_write() -> Result<()> {

    let runtime = ViperRuntime::default();

    let tmp_file = NamedTempFile::new()?;
    let path = tmp_file.path().to_str().unwrap();
    
    let suite = compile_test(&runtime, &Source::embedded(
        format!(r#"# VIPER_VERSION = 1.0
from viper import unittest

class MyTestCase(unittest.TestCase): 

    def test_file_write(self):
        f = open("{path}", "w+")
        f.write("Hello, I'm a test input! :D")
"#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    let input_after_write = fs::read_to_string(path)?;
    let expected = String::from("Hello, I'm a test input! :D");

    assert_eq!(input_after_write, expected);
    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_writelines() -> Result<()> {

    let runtime = ViperRuntime::default();

    let tmp_file = NamedTempFile::new()?;
    let path = tmp_file.path().to_str().unwrap();

    let suite = compile_test(&runtime, &Source::embedded(
        format!(r#"# VIPER_VERSION = 1.0
from viper import unittest

class MyTestCase(unittest.TestCase):
    
    def test_writelines(self):
        path = "{path}"
        f = open(path, "w+")
        f.writelines(["ABC", "DEF"])
"#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    let input_after_write = fs::read_to_string(path)?;
    let expected = String::from("ABC\nDEF\n");

    assert_eq!(input_after_write, expected);
    assert_that!(report.outcome(), eq(Outcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_write_in_read_mode() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import file, unittest

            class MyTestCase(unittest.TestCase):

                def test_write_fail(self):
                    f = open("tests/test.txt")
                    f.write("Test")
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Failure));

    Ok(())
}

#[tokio::test]
async fn test_read_in_write_mode() -> Result<()> {

    let runtime = ViperRuntime::default();

    let suite = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import file, unittest

            class MyTestCase(unittest.TestCase):

                def test_write_fail(self):
                    f = open("tests/test.txt", "w")
                    f.read()
        "#)
    )).await?.into_suite();

    let report = runtime.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;

    assert_that!(report.outcome(), eq(Outcome::Failure));

    Ok(())
}
