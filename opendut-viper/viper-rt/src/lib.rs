//! This crate provides a runtime to use Python as a language to write tests but run them with Rust.
//!
//! # Appetizer
//! ```
//! use indoc::indoc;
//!
//! use viper_rt::events::emitter;
//! use viper_rt::run::{Report, Outcome, ParameterBindings};
//! use viper_rt::source::Source;
//! use viper_rt::ViperRuntime; 
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//!     let viper = ViperRuntime::default();
//!
//!     let source = Source::embedded(
//!         indoc!(r#"
//!             ## VIPER_VERSION = 1.0
//!             from viper import *
//!
//!             class MyTestCase(unittest.TestCase):
//!                 def test_something():
//!                     self.assertEquals(7+3, 10)
//!         "#)
//!     );
//!
//!     let (_, _, suite) = viper.compile(&source, &mut emitter::drain()).await?.split();
//!
//!     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//!
//!     match report.outcome() {
//!         Outcome::Success => println!("All tests passed."),
//!         Outcome::Failure => println!("Some tests failed."),
//!     }
//!
//!     Ok(())
//! }
//! ```
//! <sup><b>Note:</b> This example uses [tokio](https://tokio.rs/), but viper is not bound to a specific async-runtime.</sup>
//!
//! # Crate Features
//!
//! Enable or disable features according to your needs and to optimize for compile time and space.
//!
//! **Main Features**
//! | Feature     | Default  | Description                                                                                                                                                                                   |
//! | ----------- |:--------:| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
//! | compile     | &#x2717; | When enabled, all types and functions for compilation and introspection of python code are available.                                                                                         |
//! | run         | &#x2717; | When enabled, all types and functions for running tests are available.                                                                                                                        |
//!
//! <sup>&#x2714; enabled, &#x2717; disabled</sup>
//!
//! **Source Features**
//! | Feature     | Default  | Description                                                                                                                                                                                   |
//! | ----------- |:--------:| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
//! | file-source | &#x2717; | When enabled, this crate provides a [`SimpleFileSourceLoader`](crate::source::loaders::SimpleFileSourceLoader) to use local files as a source.                                                |
//! | http-source | &#x2717; | When enabled, this crate provides a [`HttpSourceLoader`](crate::source::loaders::HttpSourceLoader) to use HTTP resources as a source.                                                         |
// | git-source  | &#x2717; | When enabled, this crate provides a [SourceLoader](`crate::source::loaders::SourceLoader`) implementation to use [Git](https://git-scm.com/) repositories as a source.                         |
//!
//! <sup>&#x2714; enabled, &#x2717; disabled</sup>
//!
//! **Utility Features**
//! | Feature     | Default  | Description                                                                                                                                                                                   |
//! | ----------- |:--------:| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
//! | error       | &#x2714; | When enabled, all error types of this crate implement [`core::error::Error`].                                                                                                                 |
//! | containers  | &#x2717; | When enabled, this crate provides an API to use container runtimes like [Docker](https://www.docker.com/) in tests.                                                                           |
//!
//! <sup>&#x2714; enabled, &#x2717; disabled</sup>
//!
//! **Internal Features**
//!
//! These features are used internally and should not be enabled manually.
//!
//! | Feature     | Default  | Description                                                                                                                                                                                   |
//! | ----------- |:--------:| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
//! | types       | &#x2717; | This flag is required for the `compile` and `run` configurations and will be enabled automatically.                                                                                           |
//! | py          | &#x2717; | When enabled, certain types will contain python-specific data-structures and functions. This flag is required for the `compile` and `run` configurations and will be enabled automatically.   |
//!
//! <sup>&#x2714; enabled, &#x2717; disabled</sup>
//!
//! # Writing tests
//! Tests are written in Python and executed using Viper. For comprehensive guidance on writing effective tests,
//! including best practices and examples, please refer to the [Test Writer Guide](crate::doc::test_writer_guide).
//! 
mod runtime;

pub use runtime::{
    ViperRuntime,
    options::{
        ViperOptions,
        ViperBuilder
    },
};

#[cfg(feature = "events")]
pub mod events {
    pub use crate::runtime::emitter::{
        EventEmitter,
        EventEmissionError,
    };
    pub mod emitter {
        pub use crate::runtime::emitter::{
            drain,
            fail,
            sink,
        };
    }
}

pub mod common {
    pub use crate::runtime::types::naming::{
        Identifier,
        TestIdentifier,
        TestCaseIdentifier,
        TestSuiteIdentifier,
    };
    pub mod error {
        pub use crate::runtime::types::py::error::{
            PythonReflectionError,
            PythonRuntimeError,
        };
    }
}

pub mod compile {
    pub use crate::runtime::types::compile::{
        code::{
            ApiVersion,
            SourceCode,
        },
        error::{
            CompilationError,
            CompilationErrorKind,
            CompileResult,
        },
        compilation::{
            Compilation,
        },
        metadata::Metadata,
        parameters::{
            ParameterDescriptors,
            ParameterDescriptor,
            ParameterInfo,
            ParameterName,
            ParameterError,
            InvalidParameterNameError,
            InvalidParameterNameErrorKind,
        },
        suite::{
            Test,
            TestCase,
            TestSuite,
        },
    };
    #[cfg(feature = "events")]
    pub use crate::runtime::types::compile::event::{
        CompileEvent,
        CompilationSummary,
        CompiledTestSuite,
        CompiledTestCase,
        CompiledTest,
    };
}

pub mod run {
    pub use crate::runtime::types::run::{
        error::{
            RunError,
            RunErrorKind,
            RunResult,
        },
        parameters::{
            ParameterBindings,
            ParameterBinding,
            BindingValue,
            BindParameterError,
            IncompleteParameterBindingsError,
            Incomplete,
            Complete,
        },
        report::{
            TestSuiteReport,
            TestCaseReport,
            TestReport,
            Outcome,
            Report,
            ReportProperty,
            ReportPropertyValue
        }
    };
    #[cfg(feature = "events")]
    pub use crate::runtime::types::run::event::{
        RunEvent,
        RunState,
        TestSuiteRunState,
        TestCaseRunState,
        TestRunState,
    };
}

pub mod source {
    pub use crate::runtime::types::source::{
        Source,
        SourceLocation,
    };
    pub mod loaders {
        pub use crate::runtime::source::{
            SourceLoader,
            SourceLoaderResult,
        };
        pub use crate::runtime::source::embedded::EmbeddedSourceLoader;
        #[cfg(feature = "file-source")]
        pub use crate::runtime::source::file::SimpleFileSourceLoader;
        #[cfg(feature = "http-source")]
        pub use crate::runtime::source::http::{
            HttpSourceLoader,
            HttpSourceLoaderOptions
        };
        #[cfg(feature = "git-source")]
        pub use crate::runtime::source::git::GitSourceLoader;
    }
}

#[cfg(feature = "containers")]
pub mod containers {
    pub use viper_containers::{
        ContainerRuntime,
        ContainerRuntimeError,
    };
}

#[cfg(doc)]
pub mod doc;
