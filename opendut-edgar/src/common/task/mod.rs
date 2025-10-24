use async_trait::async_trait;

pub mod runner;
mod progress_bar;
pub mod dependency;
pub mod task_resolver;

#[async_trait]
pub trait Task: Send + Sync {
    /// Short description of the task, which is shown to the user.
    fn description(&self) -> String;

    /// Used to check whether calling [Self::make_present] is necessary.
    /// And called afterward, to check whether [Self::make_present] was successful.
    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled>;

    /// Roll out the configuration described by this task to the host system.
    async fn make_present(&self) -> anyhow::Result<Success>;
}

// to avoid error that 'the trait `common::task::TaskAbsent` is not dyn compatible'
#[async_trait]
pub trait TaskAbsent: Task {
    /// Used to check whether calling [Self::make_absent] is necessary.
    /// And called afterward, to check whether [Self::make_absent] was successful.
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled>;

    /// Clean up the configuration described by this task from the host system.
    async fn make_absent(&self) -> anyhow::Result<Success>;
}


#[derive(Debug, PartialEq, Eq)]
pub enum TaskStateFulfilled {
    ///Task does not need to be executed, or successfully changed the host system during its execution.
    Yes,
    ///Task needs to be executed, or did not successfully change the host system during its execution.
    No,
    ///Always run this task, and do not fail, if this is (still) the result after execution.
    Unchecked,
}

#[derive(Default, Debug)]
pub struct Success {
    pub message: Option<String>,
}
impl Success {
    pub fn message(message: impl Into<String>) -> Self {
        Self { message: Some(message.into()) }
    }
}
