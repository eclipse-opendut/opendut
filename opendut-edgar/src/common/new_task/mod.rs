use async_trait::async_trait;
use crate::common::task::{Success, TaskFulfilled};

#[async_trait]
pub trait NewTask: Send + Sync {
    /// Short description of the task, which is shown to the user.
    fn description(&self) -> String;

    /// Called before task execution, to check whether the task needs to be executed.
    /// And called after task execution, to check whether the task execution succeeded.
    async fn check_present(&self) -> anyhow::Result<TaskFulfilled>;
    async fn check_absent(&self) -> anyhow::Result<TaskFulfilled>;

    /// Make changes to the host system.
    async fn make_absent(&self) -> anyhow::Result<Success>;
    async fn make_present(&self) -> anyhow::Result<Success>;
}