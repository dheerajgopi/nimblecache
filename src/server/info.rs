use core::fmt;
use rand::distributions::{Alphanumeric, DistString};

/// Struct to store server information
pub struct ServerInfo {
    pub role: Role,
}

impl ServerInfo {
    pub fn new(role: Role) -> ServerInfo {
        ServerInfo { role }
    }
}

/// Wrapper for storing master replication id and offset
pub struct Master {
    replication_id: String,
    replication_offset: u16,
}

pub struct Slave {
    replica_of: String,
}

/// Role assumed by the server
pub enum Role {
    /// Master role.
    MASTER(Master),
    /// Slave role. Contains the master name of whom the instance is a replica of.
    SLAVE(Slave),
}

impl Role {
    /// Parse role based on the string value.
    /// Anything other than "master" (case-insensitive) will be considered as slave.
    pub fn from_str(s: &str) -> Role {
        match s.to_uppercase().as_str() {
            "MASTER" => Self::as_master(),
            _ => Self::as_slave_of(s.to_string()),
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

    /// Create slave role.
    fn as_slave_of(master: String) -> Role {
        Role::SLAVE(Slave { replica_of: master })
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::MASTER(_) => f.write_str("master"),
            Role::SLAVE(m) => f.write_fmt(format_args!("slave of {}", m.replica_of)),
        }
    }
}
