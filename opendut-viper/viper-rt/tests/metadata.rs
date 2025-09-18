use googletest::prelude::*;
use indoc::indoc;
use viper_rt::compile::Metadata;
use viper_rt::events::emitter;
use viper_rt::source::Source;
use viper_rt::ViperRuntime;

#[tokio::test]
async fn test_metadata() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let (metadata, _, _) = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata

            METADATA = metadata.Metadata(
                description = "Just a test",
            )
        "#)
    ), &mut emitter::drain()).await?.split();

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

    let (metadata, _, _) = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata
            
            METADATA = metadata.Metadata()
        "#)
    ), &mut emitter::drain()).await?.split();

    assert_that!(metadata, eq(&Metadata::default()));

    let (metadata, _, _) = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata
            
        "#)
    ), &mut emitter::drain()).await?.split();

    assert_that!(metadata, eq(&Metadata::default()));

    Ok(())
}

#[tokio::test]
async fn test_compilation_fails_due_to_wrong_metadata_attributes() -> Result<()> {

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let runtime = ViperRuntime::default();

    let result = runtime.compile(&Source::embedded(
        indoc!(r#"
            # VIPER_VERSION = 1.0
            from viper import metadata

            METADATA = metadata.Metadata(
                fubar = "BOOOM",
            )
        "#)
    ), &mut emitter::drain()).await;

    assert_that!(result, err(anything()));

    Ok(())
}
