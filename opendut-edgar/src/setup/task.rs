pub trait Task {
    fn description(&self) -> String;
    fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled>;
    fn execute(&self) -> anyhow::Result<Success>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum TaskFulfilled {
    Yes,
    No,
    ///Always run this task. Do not fail, if this is the result after execution.
    Unchecked,
}

#[derive(Default)]
pub struct Success {
    pub message: Option<String>,
}
impl Success {
    pub fn message(message: impl Into<String>) -> Self {
        Self { message: Some(message.into()) }
    }
}
