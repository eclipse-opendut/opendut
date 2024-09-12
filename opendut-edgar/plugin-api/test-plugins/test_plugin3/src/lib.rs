use opendut_edgar_plugin_api::plugin::task::{Guest, Success, TaskFulfilled};
use opendut_edgar_plugin_api::plugin::export;

struct TestPlugin3;

static mut FIRST_EXECUTION: bool = true;

impl Guest for TestPlugin3 {
    fn description() -> String {
        String::from("Test Plugin 3 - Tasfulfilled Yes and No")
    }

    fn check_fulfilled() -> Result<TaskFulfilled, ()> {
        unsafe {
            if FIRST_EXECUTION {
                Ok(TaskFulfilled::No)
            } else {
                Ok(TaskFulfilled::Yes)
            }
        }
    }

    fn execute() -> Result<Success, ()> {
        unsafe {
            FIRST_EXECUTION = false;
        }
        Ok(Success::message("Now this will be fulfilled"))
    }
}

export!(TestPlugin3 with_types_in opendut_edgar_plugin_api::plugin::bindings);
