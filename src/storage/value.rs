use std::time::{SystemTime, UNIX_EPOCH};

/// Wrapper for string values stored in Nimblecache.
/// Apart from the actual string value, it contains additional info
/// like the expiry time etc.
#[derive(Debug, Clone)]
pub struct StringValue {
    // actual string value
    val: String,
    // expiry time in milliseconds. _None_ means no expiry.
    exp: Option<u128>,
}

impl StringValue {
    /// Returns a new [StringValue].
    pub fn new(s: String, expiry: Option<u128>) -> StringValue {
        StringValue {
            val: s,
            exp: expiry,
        }
    }

    /// Returns the actual string value.
    pub fn val(&self) -> &str {
        self.val.as_str()
    }

    // Set TTL in milliseconds
    pub fn set_ttl(&mut self, ms: u128) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.exp = Some(now + ms);
    }

    // Check if the value has reached its expiry
    pub fn has_expired(&self) -> bool {
        if self.exp.is_none() {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.exp.unwrap() < now
    }
}
