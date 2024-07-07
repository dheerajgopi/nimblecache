use core::fmt;

/// Struct to store server information
pub struct ServerInfo {
    pub role: Role,
}

impl ServerInfo {
    pub fn new(role: Role) -> ServerInfo {
        ServerInfo { role: role }
    }
}

/// Role assumed by the server
pub enum Role {
    /// Master role.
    MASTER,
    /// Slave role. Contains the master name of whom the instance is a replica of.
    SLAVE(String),
}

impl Role {
    /// Parse role based on the string value.
    /// Anything other than "master" (case insensitive) will be considered as slave.
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
            Role::MASTER => {
                s.push_str("master");
            }
            Role::SLAVE(_) => {
                s.push_str("slave");
            }
        };

        s.to_string()
    }

    fn as_master() -> Role {
        Role::MASTER
    }

    fn as_slave_of(master: String) -> Role {
        Role::SLAVE(master)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::MASTER => f.write_str("master"),
            Role::SLAVE(m) => f.write_fmt(format_args!("slave of {}", m)),
        }
    }
}
