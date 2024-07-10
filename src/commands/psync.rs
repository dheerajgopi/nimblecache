use crate::commands::traits::{CommandBuilder, CommandExecutor};
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError, SimpleString};
use crate::server::info::{Role, ServerInfo};

/// Struct for the PSYNC command.
pub struct Psync<'a> {
    /// Used to fetch server info
    server: &'a ServerInfo,
}

impl<'a> Psync<'a> {
    pub fn new(server: &ServerInfo) -> Psync {
        Psync { server }
    }

    fn is_unknown_replication_id(repl_id: &str) -> bool {
        repl_id == "?"
    }

    fn is_null_offset(offset: &str) -> bool {
        offset == "-1"
    }
}

impl<'a> CommandExecutor for Psync<'a> {
    /// Blindly return a FULLRESYNC response for now.
    /// Supports only first-time replica connection as of now.
    /// TODO: Actual replication configuration
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        let master_role = match &self.server.role {
            Role::MASTER(m) => m,
            Role::SLAVE(_) => {
                return SimpleError("PSYNC cannot be performed by a slave server".into())
            }
        };

        if args.len() < 2 {
            return SimpleError("ERR insufficient arguments for command".into());
        }

        // parse replication_id
        let replication_id = args[0];
        let replication_id = match replication_id {
            BulkString(k) => k,
            _ => {
                return SimpleError(
                    "ERR Invalid argument. replication_id must be a bulk string".into(),
                )
            }
        };

        // parse offset
        let offset = args[1];
        let offset = match offset {
            BulkString(v) => v,
            _ => return SimpleError("ERR Invalid argument. offset must be a bulk string".into()),
        };

        // when slave is connecting first time replication_id should be `?` and offset should be `-1`
        if Self::is_unknown_replication_id(replication_id.as_str())
            && Self::is_null_offset(offset.as_str())
        {
            return SimpleString(format!("FULLRESYNC {} 0", master_role.replication_id));
        }

        return SimpleError(
            "ERR - Supports FULLRESYNC for first-time replica connection only".into(),
        );
    }
}

impl<'a> CommandBuilder for Psync<'a> {
    /// Returns a PSYNC command in RESP array format.
    fn build(args: Option<&[&RespType]>) -> RespType {
        let mut cmd = vec![RespType::BulkString("PSYNC".to_string())];
        if args.is_some() {
            let cmd_args = args
                .unwrap()
                .iter()
                .map(|&r| r.clone())
                .collect::<Vec<RespType>>();
            cmd.extend(cmd_args);
        }

        RespType::Array(cmd)
    }
}
