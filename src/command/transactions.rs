// src/command/transactions.rs

use crate::{resp::types::RespType, storage::db::DB};

use super::Command;

/// Represents a Redis transaction that can be executed atomically (MULTI and EXEC).
pub struct Transaction {
    /// The queue of commands to be executed.
    commands: Vec<Command>,
    /// Indicates whether a transaction is currently active.
    is_active: bool,
}

impl Transaction {
    /// Creates a new `Transaction` instance.
    pub fn new() -> Transaction {
        Transaction {
            commands: vec![],
            is_active: false,
        }
    }

    /// Initializes a new transaction (MULTI command).
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the transaction was successfully initialized.
    /// * `Err(TransactionError::CannotNestMulti)` if a transaction is already active.
    pub fn init(&mut self) -> Result<(), TransactionError> {
        if self.is_active {
            return Err(TransactionError::CannotNestMulti);
        }
        self.is_active = true;

        Ok(())
    }

    /// Adds a command to the transaction.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to be added to the transaction.
    pub fn add_command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    /// Checks if a transaction is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Executes the commands in the transaction and returns the array of responses.
    ///
    /// This method will execute all the commands in the transaction and return the
    /// responses as a `RespType::Array`. After the execution, the transaction is
    /// automatically discarded.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// A `RespType::Array` containing the responses for each command in the transaction.
    pub async fn exec(&mut self, db: &DB) -> RespType {
        let mut responses: Vec<RespType> = vec![];

        for cmd in self.commands.iter() {
            // execute the command
            let res = cmd.execute(db);

            responses.push(res);
        }

        // discard txn after executing all commands
        self.discard();

        RespType::Array(responses)
    }

    /// Discards the current transaction.
    ///
    /// This method clears the queue of commands and resets the `is_active` flag.
    pub fn discard(&mut self) {
        self.commands = vec![];
        self.is_active = false;
    }
}

/// Represents errors that can occur during transaction operations.
#[derive(Debug)]
pub enum TransactionError {
    /// Indicates that a MULTI command cannot be nested within another active transaction.
    CannotNestMulti,
}

impl std::error::Error for TransactionError {}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::CannotNestMulti => "MULTI calls cannot be nested".fmt(f),
        }
    }
}
