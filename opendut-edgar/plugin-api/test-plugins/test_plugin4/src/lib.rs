use opendut_edgar_plugin_api::plugin::export;
use opendut_edgar_plugin_api::plugin::task::{Guest, Success, TaskFulfilled};

struct TestPlugin4;

impl Guest for TestPlugin4 {
    fn description() -> String {
        String::from("Test Plugin 4 - Taskfulfilled Err")
    }

    fn check_fulfilled() -> Result<TaskFulfilled, ()> {
        Err(())
    }

    fn execute() -> Result<Success, ()> {
        Ok(Success::message("This should never be reached!"))
    }
}

export!(TestPlugin4 with_types_in opendut_edgar_plugin_api::plugin::bindings);
