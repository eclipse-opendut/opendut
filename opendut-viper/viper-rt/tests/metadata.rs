use googletest::prelude::*;
use indoc::indoc;
use opendut_viper_rt::compile::{Compilation, CompileResult, IdentifierFilter, Metadata};
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;

async fn compile_test(runtime: &ViperRuntime, source: &Source) -> CompileResult<Compilation> {
    runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
}

#[tokio::test]
async fn test_metadata() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (metadata, _, _) = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata

            METADATA = metadata.Metadata(
                description = "Just a test",
            )
        "#)
    )).await?.split();

    assert_that!(metadata, matches_pattern!(
        Metadata {
            description: &Some(String::from("Just a test")),
            ..
        }
    ));

    Ok(())
}

#[tokio::test]
async fn test_metadata_is_empty_or_absent() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (metadata, _, _) = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata
            
            METADATA = metadata.Metadata()
        "#)
    )).await?.split();

    assert_that!(metadata, eq(&Metadata::default()));

    let (metadata, _, _) = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata
            
        "#)
    )).await?.split();

    assert_that!(metadata, eq(&Metadata::default()));

    Ok(())
}

#[tokio::test]
async fn test_compilation_fails_due_to_wrong_metadata_attributes() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let result = compile_test(&runtime, &Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata

            METADATA = metadata.Metadata(
                fubar = "BOOOM",
            )
        "#)
    )).await;

    assert_that!(result, err(anything()));

    Ok(())
}
