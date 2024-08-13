#[derive(Debug, Clone)]
pub struct Replication {
    id: String,
    offset: u64,
    master_host: Option<String>,
    master_port: Option<u16>,
}

impl Replication {
    pub fn new(id: String, master_host_port: Option<(String, u16)>) -> Replication {
        let (master_host, master_port) = match master_host_port {
            Some((h, p)) => (Some(h), Some(p)),
            None => (None, None),
        };
        Replication {
            id,
            offset: 0,
            master_host,
            master_port,
        }
    }

    pub fn is_slave(&self) -> bool {
        self.master_host.is_some()
    }

    /// Returns the replication info in `<key>:<value>` format.
    pub fn info_str(&self) -> String {
        let mut s = String::new();
        s.push_str("role:");

        if self.is_slave() {
            s.push_str("slave");
        } else {
            s.push_str("master\n");
            s.push_str(format!("master_replid:{}\n", self.id).as_str());
            s.push_str(format!("master_repl_offset:{}\n", self.offset).as_str())
        }

        s.to_string()
    }
}
