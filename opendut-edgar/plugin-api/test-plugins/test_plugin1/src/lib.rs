use opendut_edgar_plugin_api::plugin::{export, trace, debug, info, warn, error};
use opendut_edgar_plugin_api::plugin::task::{Guest, Success, TaskFulfilled};

struct TestPlugin1;

impl Guest for TestPlugin1 {
    fn description() -> String {
        trace("Testing trace-logging");
        debug("Testing debug-logging");
        info("Testing info-logging");
        warn("Testing warn-logging");
        error("Testing error-logging");     
                
        String::from("Test Plugin 1 - Logging - This is already fulfilled")

    }

    fn check_fulfilled() -> Result<TaskFulfilled, ()> {
        Ok(TaskFulfilled::Yes)
    }

    fn execute() -> Result<Success, ()> {
        Ok(Success::message("This should never be reached!"))
    }
}

export!(TestPlugin1 with_types_in opendut_edgar_plugin_api::plugin::bindings);
