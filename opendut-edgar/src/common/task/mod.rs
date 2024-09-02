pub mod runner;

pub trait Task: Send + Sync {
    /// Short description of the task, which is shown to the user.
    fn description(&self) -> String;

    /// Called before task execution, to check whether the task needs to be executed.
    /// And called after task execution, to check whether the task execution succeeded.
    fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled>;

    /// Make changes to the host system.
    fn execute(&self) -> anyhow::Result<Success>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum TaskFulfilled {
    ///Task does not need to be executed, or successfully changed the host system during its execution.
    Yes,
    ///Task needs to be executed, or did not successfully change the host system during its execution.
    No,
    ///Always run this task, and do not fail, if this is (still) the result after execution.
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
