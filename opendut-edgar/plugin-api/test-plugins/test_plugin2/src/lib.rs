use opendut_edgar_plugin_api::plugin::{export, info};
use opendut_edgar_plugin_api::plugin::host::call_command;
use opendut_edgar_plugin_api::plugin::task::{Guest, Success, TaskFulfilled};

struct TestPlugin2;

impl Guest for TestPlugin2 {
    fn description() -> String {
        String::from("Test Plugin 2 - Commands")
    }

    fn check_fulfilled() -> Result<TaskFulfilled, ()> {
        Ok(TaskFulfilled::Unchecked)
    }

    fn execute() -> Result<Success, ()> {
        let command_result = call_command("false", &vec![]);

        info(format!("Expecting to fail: {:?}", &command_result).as_str());

        let expect_fail = match command_result {
            Ok(_) => Err(()),
            Err(_) => Ok(Success::new()),
        };

        let command_result = call_command("true", &vec![]);

        info(format!("Expecting to succeed: {:?}", &command_result).as_str());

        let expect_success = match command_result {
            Ok(_) => Ok(Success::new()),
            Err(_) => Err(()),
        };

        if expect_fail.is_ok() && expect_success.is_ok(){
            Ok(Success::message("Failing and Succeding command returned successfuly"))
        }else{
            Err(())
        }

    }
}

export!(TestPlugin2 with_types_in opendut_edgar_plugin_api::plugin::bindings);
