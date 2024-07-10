use anyhow::{anyhow, Result};
use core::fmt;
use rand::distributions::{Alphanumeric, DistString};

/// Struct to store server information
pub struct ServerInfo {
    pub role: Role,
    pub port: u16,
}

impl ServerInfo {
    pub fn new(role: Role, port: u16) -> ServerInfo {
        ServerInfo { role, port }
    }
}

/// Wrapper for storing master replication id and offset.
/// When a replica connects to a master it uses the replication_id and offset value to
/// check if its completely in sync with the data in master.
pub struct Master {
    /// Pseudorandom alphanumeric string with length of 40 characters.
    pub replication_id: String,
    /// Offset is incremented for each byte of replication stream that is sent to replicas.
    pub replication_offset: u64,
}

pub struct Slave {
    pub master_host: String,
    pub master_port: u16,
}

/// Role assumed by the server
pub enum Role {
    /// Master role.
    MASTER(Master),
    /// Slave role. Contains the master name of whom the instance is a replica of.
    SLAVE(Slave),
}

impl Role {
    /// Parse and create the role based on the string value.
    /// Anything other than "master" (case-insensitive) will be considered as slave.
    ///
    /// # Errors
    /// If `replicaof` cli arg is invalid for slave role. Correct format is `<MASTER_HOST> <MASTER_PORT>`.
    pub fn from_str(s: &str) -> Result<Role> {
        match s.trim().to_uppercase().as_str() {
            "MASTER" => Ok(Self::as_master()),
            _ => match Self::as_slave_of(s.to_string()) {
                Ok(slave) => Ok(slave),
                Err(e) => Err(e),
            },
        }
    }

    /// Returns a string in "role:<value>" format.
    /// This can be used in INFO command.
    pub fn info_str(&self) -> String {
        let mut s = String::new();
        s.push_str("role:");

        match self {
            Role::MASTER(m) => {
                s.push_str("master\n");
                s.push_str(format!("master_replid:{}\n", m.replication_id).as_str());
                s.push_str(format!("master_repl_offset:{}\n", m.replication_offset).as_str())
            }
            Role::SLAVE(_) => {
                s.push_str("slave");
            }
        };

        s.to_string()
    }

    /// Create master role, generate a pseudo-random alphanumeric replication id, and set that to the role.
    /// replication offset will be set as 0.
    fn as_master() -> Role {
        let replication_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 40);
        Role::MASTER(Master {
            replication_id,
            replication_offset: 0,
        })
    }

    /// Create slave role by parsing `replicaof` cli arg and extracting master host and port.
    ///
    /// # Validations
    /// Check format of `replicaof`. Correct format is `<MASTER_HOST> <MASTER_PORT>`.
    fn as_slave_of(master_host_and_port: String) -> Result<Role> {
        let host_port = Self::parse_host_port(master_host_and_port);
        match host_port {
            Ok((h, p)) => Ok(Role::SLAVE(Slave {
                master_host: h,
                master_port: p,
            })),
            Err(e) => Err(e),
        }
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
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::MASTER(_) => f.write_str("master"),
            Role::SLAVE(m) => {
                f.write_fmt(format_args!("slave of {}:{}", m.master_host, m.master_port))
            }
        }
    }
}
