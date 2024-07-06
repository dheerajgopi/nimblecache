/// Wrapper for string values stored in Nimblecache.
/// Apart from the actual string value, it contains additional info
/// like the expiry time etc.
#[derive(Debug, Clone)]
pub struct StringValue {
    // actual string value
    val: String,
    // expiry time in milliseconds. _None_ means no expiry.
    exp: Option<u64>,
}

impl StringValue {
    /// Returns a new [StringValue].
    pub fn new(s: String, expiry: Option<u64>) -> StringValue {
        StringValue {
            val: s,
            exp: expiry,
        }
    }

    /// Returns the actual string value.
    pub fn val(&self) -> &str {
        self.val.as_str()
    }
}
