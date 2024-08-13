use crate::{replication::Replication, resp::types::RespType, storage::db::DB};

use super::Command;

/// Represents a Redis command pipeline that can be executed atomically (MULTI and EXEC).
pub struct MultiCommand {
    /// The queue of commands to be executed.
    commands: Vec<Command>,
    /// Indicates whether a pipeline is currently active.
    is_active: bool,
}

impl MultiCommand {
    /// Creates a new `MultiCommand` instance.
    pub fn new() -> MultiCommand {
        MultiCommand {
            commands: vec![],
            is_active: false,
        }
    }

    /// Initializes a new pipeline (MULTI command).
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the transaction was successfully initialized.
    /// * `Err(PipelineError::CannotNestMulti)` if a pipeline is already active.
    pub fn init(&mut self) -> Result<(), PipelineError> {
        if self.is_active {
            return Err(PipelineError::CannotNestMulti);
        }
        self.is_active = true;

        Ok(())
    }

    /// Adds a command to the pipeline.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to be added to the pipeline.
    pub fn add_command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    /// Checks if a pipeline is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Executes the commands in the pipeline and returns the array of responses.
    ///
    /// This method will execute all the commands in the pipeline and return the
    /// responses as a `RespType::Array`. After the execution, the pipeline is
    /// automatically discarded.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// * `replication` - Server replication.
    ///
    /// # Returns
    ///
    /// A `RespType::Array` containing the responses for each command in the pipeline.
    pub fn exec(&mut self, db: &DB, replication: &Replication) -> RespType {
        let responses = self
            .commands
            .iter()
            .map(|cmd| cmd.execute(db, replication))
            .collect::<Vec<RespType>>();

        self.discard();

        RespType::Array(responses)
    }

    /// Discards the current pipeline.
    ///
    /// This method clears the queue of commands and resets the `is_active` flag.
    pub fn discard(&mut self) {
        self.commands = vec![];
        self.is_active = false;
    }
}

/// Represents errors that can occur during pipeline operations.
#[derive(Debug)]
pub enum PipelineError {
    /// Indicates that a MULTI command cannot be nested within another active pipeline.
    CannotNestMulti,
}

impl std::error::Error for PipelineError {}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::CannotNestMulti => "MULTI calls cannot be nested".fmt(f),
        }
    }
}
