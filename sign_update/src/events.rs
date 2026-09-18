#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Frontend,
    Backend,
    Hash,
    Archive,
}

impl Task {
    pub const ALL: [Task; 4] = [Task::Frontend, Task::Backend, Task::Hash, Task::Archive];

    pub fn name(self) -> &'static str {
        match self {
            Task::Frontend => "Build frontend",
            Task::Backend => "Build backend",
            Task::Hash => "Create hash",
            Task::Archive => "Create archive",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
}

pub enum AppEvent {
    TaskStarted(Task),
    Output(String),
    TaskFinished(Task, Result<(), String>),
    ParallelStarted,
    ParallelOutput(Task, String),
    ParallelTaskFinished(Task, Result<(), String>),
    ParallelFinished,
    GitPushFinished(Result<(), String>),
}
