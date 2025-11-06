//! # Writing tests
//!
//! ## Structure
//!
//! Tests in Viper are organized in the same way as with Python's [unittest](https://docs.python.org/3/library/unittest.html) library.
//!
//! ```text
//! ┌───────────────────────────────────┐
//! │ Test Suite                        │
//! │                                   │
//! │     ┌──────────────────────────┐  │
//! │     │ Test Case                │  │
//! │     │                          │  │
//! │     │     ┌─────────────────┐  │  │
//! │     │     │ Test            │  │  │
//! │     │     └─────────────────┘  │  │
//! │     └──────────────────────────┘  │
//! │     ┌──────────────────────────┐  │
//! │     │ Test Case                │  │
//! │     │                          │  │
//! │     │     ┌─────────────────┐  │  │
//! │     │     │ Test            │  │  │
//! │     │     └─────────────────┘  │  │
//! │     │     ┌─────────────────┐  │  │
//! │     │     │ Test            │  │  │
//! │     │     └─────────────────┘  │  │
//! │     └──────────────────────────┘  │
//! └───────────────────────────────────┘
//! ```
//!
//! Here's an example:
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0                          # (1)
//! from viper import *                            # (2)
//!
//! class MyFirstTestCase(unittest.TestCase):       # (3)
//!     def test_something(self):                   # (4)
//!         self.assertEquals(21 + 21, 42)          # (5)
//!
//! class AnotherTestCase(unittest.TestCase):       # (6)
//!     def test_something(self):
//!         self.assertTrue(True)
//!     def test_the_truth(self):                   # (7)
//!         self.assertTrue(self.compute_truth())   # (8)
//!     def compute_truth(self):                    # (9)
//!         return fetch_opinion()
//!
//! def fetch_opinion():                            # (10)
//!     return True
//! #     "#));
//! #     let (_, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//! 1. The first line is important for Viper to know which version of Viper's Python API you are
//!    using. Therefore, setting `VIPER_VERSION` to a valid version is mandatory.
//! 2. The second line imports Viper's Python library.
//! 3. To declare a test case, write a Python class and inherit from `unittest.TestCase`.
//! 4. To define a test, add a method to the class prefixed with `test_`.
//! 5. Write your test logic within the test method.
//! 6. You can define as many test cases as you want.
//! 7. You can define as many tests as you want within a test case.
//! 8. You can use the `self` object to access the test case's attributes and methods.
//! 9. You can define utility methods within the test case to encapsulate some logic.
//! 10. You can define any other Python code you want to structure your test logic.
//!
//! ### Setup & Teardown
//!
//! In addition to the test methods, you can also define `setUp`/`setUpClass` and `tearDown`/`tearDownClass` methods.
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! class MyTestCase(unittest.TestCase):
//!     def setUp(self):        # (1)
//!         pass
//!     def tearDown(self):     # (2)
//!         pass
//!     @classmethod
//!     def setUpClass(cls):    # (3)
//!         pass
//!     @classmethod
//!     def tearDownClass(cls): # (4)
//!         pass
//! #     "#));
//! #     let (_, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//! 1. `setUp` is called before each test method - useful for initializing test resources like opening temporary files.
//! 2. `tearDown` is called after each test method - useful for cleaning up resources created during a test.
//! 3. `setUpClass` is called before the first test method in the class - good for one-time setup across all tests like starting a server.
//! 4. `tearDownClass` is called after the last test method in the class - good for cleaning up class-level resources like shutting down servers.
//!
//! ## Assertions
//!
//! Viper's Python API provides a set of assertions that you can use to write your test logic.
//!
//! The example blow shows the common assertions:
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! class MyTestCase(unittest.TestCase):
//!     def test_something(self):
//!         self.assertEquals(1 + 1, 2) # Check if values are equal
//!         self.assertTrue(True)       # Check if value is True
//!         self.assertFalse(False)     # Check if value is False
//!         self.assertIs(1, 1)         # Check if objects are the same
//!         self.assertIn(1, [1, 2, 3]) # Check if item is in container
//!         self.assertIsNone(None)     # Check if value is None
//! #     "#));
//! #
//! #     let (_, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//! ## Metadata
//!
//! Metadata allows you to attach additional descriptive information to your test suite. Using the `metadata`
//! module, you can provide context about the test suite, such as its purpose, requirements, or any other
//! relevant documentation. See [here](crate::compile::Metadata) for a full list of fields.
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::compile::Metadata;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use indoc::indoc;
//! # use googletest::prelude::*;
//! #
//! # #[tokio::main]
//! # async fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! METADATA = metadata.Metadata(
//!     display_name="My Awesome Test Suite",
//!     description="Verifies the awesomeness of my software.",
//! )
//! #     "#));
//! #     let (metadata, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     assert_that!(metadata, matches_pattern!(
//! #         Metadata {
//! #             display_name: &Some(String::from("My Awesome Test Suite")),
//! #             description: &Some(String::from("Verifies the awesomeness of my software.")),
//! #         }
//! #     ));
//! #     assert!(report.is_success());
//! #
//! #     Ok(())
//! # }
//! ```
//! 
//! ## Parameters
//!
//! Parameters allow you to define test inputs that can be specified outside the test, making it
//! possible to run the same test logic across different configurations.
//!
//! They are declared at the top level using the `parameters` module and accessed within test
//! methods using `self.parameters.get()`.
//!
//! Each parameter requires a unique name and can have:
//! - A default value.
//! - A display name and description for UI purposes.
//! - Type-specific constraints like min/max.
//!
//! Here's an example:
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use indoc::indoc;
//! # use opendut_viper_rt::compile::ParameterName;
//! # use opendut_viper_rt::run::{BindingValue , ParameterBindings, Report};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! NAME = parameters.TextParameter("name")
//! AGE = parameters.NumberParameter("age")
//! IS_DEVELOPER = parameters.BooleanParameter("is_developer")
//!
//! class MyTestCase(unittest.TestCase):
//!     def test_something(self):
//!         name = self.parameters.get(NAME)
//!         age = self.parameters.get(AGE)
//!         is_developer = self.parameters.get(IS_DEVELOPER)
//!         if is_developer and age < 20:
//!             print(f"Hello young Developer!")
//!         else:
//!             print(f"Hello {name}!")
//! #     "#));
//! #
//! #     let (_, parameters, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let mut bindings = ParameterBindings::from(parameters);
//! #     bindings.bind(&ParameterName::try_from("name")?, BindingValue::TextValue(String::from("Alice")))?;
//! #     bindings.bind(&ParameterName::try_from("age")?, BindingValue::NumberValue(34))?;
//! #     bindings.bind(&ParameterName::try_from("is_developer")?, BindingValue::BooleanValue(true))?;
//! #     let report = viper.run(suite, bindings.complete()?, &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//!
//! ### Parameter Types
//!
//! Viper supports different parameter types to handle various kinds of test inputs:
//!
//! #### TextParameter ([Descriptor](viper_py::parameters::parameters::PyTextParameterDescriptor))
//!
//! For text input values. Useful for configuring strings like names, messages, or identifiers in tests.
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings, BindingValue};
//! # use opendut_viper_rt::compile::ParameterName;
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! # from viper import *
//!
//! NAME = parameters.TextParameter(
//!     "name",
//!     default="Jessica",
//!     max=7,
//!     display_name="Username",
//!     description="Login-Name of the user to test with."
//! )
//! #
//! # class MyTestCase(unittest.TestCase):
//! #    def test_something(self):
//! #        self.assertEquals(self.parameters.get(NAME), "Vivian")
//! #     "#));
//! #     let (_, parameters, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let mut bindings = ParameterBindings::from(parameters);
//! #     bindings.bind(&ParameterName::try_from("name")?, BindingValue::TextValue(String::from("Vivian")))?;
//! #     let report = viper.run(suite, bindings.complete()?, &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//!
//! #### NumberParameter ([Descriptor](viper_py::parameters::parameters::PyNumberParameterDescriptor))
//!
//! For numeric input values. Commonly used to configure numeric settings like ports, counts, or numeric thresholds in tests.
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings, BindingValue};
//! # use opendut_viper_rt::compile::ParameterName;
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! # from viper import *
//!
//! PORT = parameters.NumberParameter(
//!     "server-port",
//!     default=1207,
//!     max=65535,
//!     min=0,
//!     display_name="Server Port",
//!     description="Port of the server to test with."
//! )
//! #
//! # class MyTestCase(unittest.TestCase):
//! #    def test_something(self):
//! #        self.assertEquals(self.parameters.get(PORT), 8121)
//! #     "#));
//! #     let (_, parameters, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let mut bindings = ParameterBindings::from(parameters);
//! #     bindings.bind(&ParameterName::try_from("server-port")?, BindingValue::NumberValue(8121))?;
//! #     let report = viper.run(suite, bindings.complete()?, &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//!
//! #### BooleanParameter ([Descriptor](viper_py::parameters::parameters::PyBooleanParameterDescriptor))
//!
//! For defining boolean input values (true/false). Useful for controlling test behavior through flags and conditional test execution.
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings, BindingValue};
//! # use opendut_viper_rt::compile::ParameterName;
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! # from viper import *
//!
//! ENABLED = parameters.BooleanParameter(
//!     "spcial-tests",
//!     default=True,
//!     display_name="Test Enabled",
//!     description="Enables special tests."
//! )
//! #
//! # class MyTestCase(unittest.TestCase):
//! #    def test_something(self):
//! #        self.assertFalse(self.parameters.get(ENABLED))
//! #     "#));
//! #     let (_, parameters, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let mut bindings = ParameterBindings::from(parameters);
//! #     bindings.bind(&ParameterName::try_from("spcial-tests")?, BindingValue::BooleanValue(false))?;
//! #     let report = viper.run(suite, bindings.complete()?, &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! # }
//! ```
//!
//! ## Report Properties
//!
//! Properties can be set during test execution to provide additional information in the test report.
//! These properties are useful for storing test data, runtime values, or any other contextual information that
//! might be relevant for test analysis and reporting.
//!
//! ### Key-Value Properties
//!
//! You can attach key-value properties individually using `self.report.property()` or multiple properties
//! at once using `self.report.properties()` ([kwargs](https://book.pythontips.com/en/latest/args_and_kwargs.html)).
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! class MyTestCase(unittest.TestCase):
//!     def test_something(self):
//!         self.report.property("foo", "bar")
//!         self.report.property("baz", 42)
//!         self.report.properties(
//!             foo="bar",
//!             baz=42,
//!         )
//! #     "#));
//! #     let (_, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! }
//! ```
//! 
//! ### File Properties
//!
//! File properties allow you to attach files to your test report for further analysis or documentation. 
//! These can be any type of files such as logs, screenshots, generated reports, or data files. You can 
//! attach files individually using `self.report.file()` or multiple files at once using `self.report.files()`.
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! class MyTestCase(unittest.TestCase):
//!     def test_something(self):
//!         self.report.file("log.txt")
//!         self.report.files(
//!             "report/values.json",
//!             "report/report.pdf",
//!         )
//! #     "#));
//! #     let (_, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let report = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     assert!(report.is_success());
//! #     Ok(())
//! }
//! ```
//!
//! ## Containers
//!
//! <div class="warning">
//! The container API is a separate feature which must be enabled when compiling this crate.
//! Further, a container runtime must be selected when instantiating a <code>ViperRuntime</code>.
//! </div>
//!
//! Viper provides a container API that allows tests to create and interact with containers (e.g. [Docker](https://www.docker.com/))
//! during test execution. This is particularly useful when additional software or special environments are required for testing.
//! The API supports operations like creating containers with custom configurations (image, entrypoint, environment variables),
//! starting containers, and waiting for their completion. The container functionality is available through the `self.container`
//! object in test methods.
//!
//! Here's an example:
//!
//! ```
//! # use opendut_viper_rt::events::emitter;
//! # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
//! # use opendut_viper_rt::source::Source;
//! # use opendut_viper_rt::ViperRuntime;
//! # use opendut_viper_rt::compile::IdentifierFilter;
//! # use indoc::indoc;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let viper = ViperRuntime::default();
//! #     let source = Source::embedded(indoc!(r#"
//! ## VIPER_VERSION = 1.0
//! from viper import *
//!
//! class MyTestCase(unittest.TestCase):
//!     def test_with_container(self):
//!         container = self.container.create(
//!             "alpine:latest",
//!             ["Hello from container!"],
//!             entrypoint=["echo"],
//!             env=["DEBUG=true"],
//!             name="test-container",
//!             user="1000"
//!         )
//!         self.container.start(container)
//!         exit_code = self.container.wait(container)
//!         self.assertEquals(0, exit_code)
//! #     "#));
//! #     let (_, _, suite) = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.split();
//! #     let _ = viper.run(suite, ParameterBindings::new(), &mut emitter::drain()).await?;
//! #     Ok(())
//! # }
//! ```
//!
