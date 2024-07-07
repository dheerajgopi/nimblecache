use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Port to be bound to Nimblecache
    #[arg(short, long, default_value_t = 6379)]
    pub port: u16,
    /// Specify which role is to be assumed by the server (master/slave)
    #[arg(long = "replicaof", default_value = "master")]
    pub replica_of: String,
}
