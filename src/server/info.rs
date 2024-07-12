use crate::cli::args::Args;
use anyhow::{anyhow, Result};
use rand::distributions::{Alphanumeric, DistString};

/// Struct to store server information
#[derive(Clone)]
pub struct ServerConfig {
    pub role: Role,
    pub port: u16,
    pub replication: Replication,
    pub master: Option<Master>,
}

impl ServerConfig {
    /// Create a new instance of ServerConfig.
    /// Role: Anything other than "master" (case-insensitive) will be considered as slave.
    ///
    /// # Errors
    /// If `replicaof` cli arg is invalid for slave role. Correct format is `<MASTER_HOST> <MASTER_PORT>`.
    pub fn new(args: &Args) -> Result<ServerConfig> {
        // set replication id
        let replication_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 40);
        // set role based on `replicaof` cli arg
        let role = Role::from_str(args.replica_of.as_str());
        let master: anyhow::Result<Option<Master>> = match role {
            Role::MASTER => Ok(None),
            Role::SLAVE => {
                // set master host and port if role is slave
                match Self::parse_host_port(args.replica_of.clone()) {
                    Ok((h, p)) => Ok(Some(Master { host: h, port: p })),
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        };

        let master = master.unwrap();

        Ok(ServerConfig {
            role,
            port: args.port,
            replication: Replication {
                id: replication_id,
                offset: 0,
            },
            master,
        })
    }

    fn parse_host_port(host_port_str: String) -> Result<(String, u16)> {
        let mut split = host_port_str.split_whitespace();

        let host = match split.next() {
            Some(h) => h,
            None => {
                return Err(anyhow!("Invalid value for replicaof. replicaof should be in '<MASTER_HOST> <MASTER_PORT>' format"));
            }
        };

        let port = match split.next() {
            Some(p) => p,
            None => {
                return Err(anyhow!("Master port is not specified in replicaof"));
            }
        };

        let port = port.parse::<u16>();
        let port = match port {
            Ok(p) => p,
            Err(_) => {
                return Err(anyhow!("Invalid value for master port in replicaof"));
            }
        };

        Ok((host.to_string(), port))
    }

    /// Returns a string in "role:<value>" format.
    /// This can be used in INFO command.
    pub fn info_replication(&self) -> String {
        let mut s = String::new();
        s.push_str("role:");

        match self.role {
            Role::MASTER => {
                s.push_str("master\n");
                s.push_str(format!("master_replid:{}\n", self.replication.id).as_str());
                s.push_str(format!("master_repl_offset:{}\n", self.replication.offset).as_str())
            }
            Role::SLAVE => {
                s.push_str("slave");
            }
        };

        s.to_string()
    }
}

/// Wrapper for storing master replication id and offset.
/// When a replica connects to a master it uses the replication_id and offset value to
/// check if its completely in sync with the data in master.
#[derive(Clone)]
pub struct Replication {
    /// Pseudorandom alphanumeric string with length of 40 characters.
    pub id: String,
    /// Offset is incremented for each byte of replication stream that is sent to replicas.
    pub offset: u64,
}

/// Stores the master's host and port.
#[derive(Clone)]
pub struct Master {
    pub host: String,
    pub port: u16,
}

/// Role assumed by the server
#[derive(Clone)]
pub enum Role {
    /// Master role.
    MASTER,
    /// Slave role.
    SLAVE,
}

impl Role {
    /// Parse and create the role based on the string value.
    /// Anything other than "master" (case-insensitive) will be considered as slave.
    ///
    /// # Errors
    /// If `replicaof` cli arg is invalid for slave role. Correct format is `<MASTER_HOST> <MASTER_PORT>`.
    pub fn from_str(s: &str) -> Role {
        match s.trim().to_uppercase().as_str() {
            "MASTER" => Role::MASTER,
            _ => Role::SLAVE,
        }
    }
}
