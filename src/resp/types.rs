use bytes::{Bytes, BytesMut};

use super::RespError;

/// Nimblecache supports Redis Serialization Protocol or RESP.
/// This enum is a wrapper for the different RESP types.
/// Please refer <https://redis.io/docs/latest/develop/reference/protocol-spec/> for more info
/// on the RESP protocol.
#[derive(Clone, Debug)]
pub enum RespType {
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-strings>
    SimpleString(String),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#bulk-strings>
    BulkString(String),
    /// Null representation in RESP2. It's simply a BulkString with length of negative one (-1).
    NullBulkString,
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#arrays>
    Array(Vec<RespType>),
    /// Refer <https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-errors>
    SimpleError(String),
}

impl RespType {
    /// Parse the given bytes into its respective RESP type and return the parsed RESP value and
    /// the number of bytes read from the buffer.
    ///
    /// More details on the parsing logic is available at
    /// <https://redis.io/docs/latest/develop/reference/protocol-spec/#resp-protocol-description>.
    ///
    /// # Errors
    ///
    /// Error will be returned in the following scenarios:
    /// - If first byte is an invalid character.
    /// - If the parsing fails due to encoding issues etc.
    pub fn parse(buffer: BytesMut) -> Result<(RespType, usize), RespError> {
        let c = buffer[0] as char;
        return match c {
            '$' => Self::new_bulk_string(buffer),
            '*' => Self::new_array(buffer),
            _ => Err(RespError::InvalidDataTypePrefix),
        };
    }
    /// Parse the given bytes into a BulkString RESP value. This will return the parsed RESP
    /// value and the number of bytes read from the buffer.
    ///
    /// Example BulkString: `$5\r\nhello\r\n`
    ///
    /// # BulkString Parts:
    /// ```
    ///     $      |            5           | \r\n |    hello     | \r\n
    /// identifier | string length in bytes | CRLF | string value | CRLF
    /// ```
    ///
    /// # Parsing Logic:
    /// - The buffer is read until CRLF characters ("\r\n") are encountered.
    /// - That slice of bytes are then parsed into an int. That will be the string length in bytes (let's say `bulkstr_len`)
    /// - `bulkstr_len` number of bytes are read from the buffer again from where it was stopped previously.
    /// - This 2nd slice of bytes is then parsed into an UTF-8 string.
    ///
    /// Note: The first byte in the buffer is skipped since it's just an identifier for the
    /// RESP type and is not the part of the actual value itself.
    pub fn new_bulk_string(buffer: BytesMut) -> Result<(RespType, usize), RespError> {
        let (bulkstr_len, bytes_consumed) =
            if let Some((buf_data, len)) = Self::read_till_crlf(&buffer[1..]) {
                let bulkstr_len = Self::parse_usize_from_buf(buf_data)?;
                (bulkstr_len, len + 1)
            } else {
                return Err(RespError::InvalidBulkString(String::from(
                    "Invalid value for bulk string",
                )));
            };

        let bulkstr_end_idx = bytes_consumed + bulkstr_len as usize;
        if bulkstr_end_idx >= buffer.len() {
            return Err(RespError::InvalidBulkString(String::from(
                "Invalid value for bulk string length",
            )));
        }
        let bulkstr = String::from_utf8(buffer[bytes_consumed..bulkstr_end_idx].to_vec());

        match bulkstr {
            Ok(bs) => Ok((RespType::BulkString(bs), bulkstr_end_idx + 2)),
            Err(_) => Err(RespError::InvalidBulkString(String::from(
                "Bulk string value is not a valid UTF-8 string",
            ))),
        }
    }

    /// Parse the given bytes into an Array RESP value. This will return the parsed RESP
    /// value and the number of bytes read from the buffer.
    ///
    /// Example Array: `*2\r\n$3\r\nSan\r\n$9\r\nFrancisco\r\n`
    ///
    /// The above array is of length 2, and contains 2 BulkStrings.
    ///
    /// # Array Parts:
    /// ```
    ///     *      |      2       | \r\n |      $3\r\nSan\r\n      |    $9\r\nFrancisco\r\n
    /// identifier | array length | CRLF | first item in the array | second item in the array
    /// ```
    ///
    /// # Parsing Logic:
    /// - The buffer is read until CRLF characters ("\r\n") are encountered.
    /// - That slice of bytes are then parsed into an int. That will be the array length (let's say `arr_len`)
    /// - [Self::parse] is called `arr_len` number of times on the remaining bytes of the buffer to parse each array item.
    ///
    /// Note: The first byte in the buffer is skipped since it's just an identifier for the
    /// RESP type and is not the part of the actual value itself.
    pub fn new_array(buffer: BytesMut) -> Result<(RespType, usize), RespError> {
        let (arr_len, mut bytes_consumed) =
            if let Some((buf_data, len)) = Self::read_till_crlf(&buffer[1..]) {
                let arr_len = Self::parse_usize_from_buf(buf_data)?;
                (arr_len, len + 1)
            } else {
                return Err(RespError::InvalidArray(String::from(
                    "Invalid value for array",
                )));
            };

        let mut items: Vec<RespType> = vec![];
        for _ in 0..arr_len {
            if bytes_consumed >= buffer.len() {
                return Err(RespError::InvalidArray(String::from(
                    "Invalid value for array length",
                )));
            }
            let item = Self::parse(BytesMut::from(&buffer[bytes_consumed..]));
            match item {
                Ok((data, bytes_read)) => {
                    items.push(data);
                    bytes_consumed += bytes_read;
                }
                Err(e) => return Err(e),
            }
        }

        return Ok((RespType::Array(items), bytes_consumed));
    }

    /// Convert the RESP value into its byte values.
    pub fn to_bytes(&self) -> Bytes {
        return match self {
            RespType::SimpleString(ss) => Bytes::from_iter(format!("+{}\r\n", ss).into_bytes()),
            RespType::BulkString(bs) => {
                let bulkstr_bytes = format!("${}\r\n{}\r\n", bs.chars().count(), bs).into_bytes();
                Bytes::from_iter(bulkstr_bytes)
            }
            RespType::NullBulkString => Bytes::from("$-1\r\n"),
            RespType::Array(arr) => {
                let mut arr_bytes = format!("*{}\r\n", arr.len()).into_bytes();
                arr.iter()
                    .map(|v| v.to_bytes())
                    .for_each(|b| arr_bytes.extend(b));

                Bytes::from_iter(arr_bytes)
            }
            RespType::SimpleError(es) => Bytes::from_iter(format!("-{}\r\n", es).into_bytes()),
            _ => unimplemented!(),
        };
    }

    /// Parses the length of a RESP array from the given byte buffer.
    ///
    /// This function attempts to read the first few bytes of a RESP array to determine its length.
    /// It expects the input to start with a '*' character followed by the length and terminated by CRLF.
    ///
    /// # Arguments
    ///
    /// * `src` - A `BytesMut` containing the bytes to parse.
    ///
    /// # Returns
    ///
    /// * `Ok(Some((usize, usize)))` - If successful, returns a tuple containing:
    ///   - The parsed length of the array
    ///   - The number of bytes read from the input
    /// * `Ok(None)` - If there's not enough data in the buffer to parse the length
    /// * `Err(RespError)` - If the input is not a valid RESP array prefix or if parsing fails
    pub fn parse_array_len(src: BytesMut) -> Result<Option<(usize, usize)>, RespError> {
        let (array_prefix_bytes, bytes_read) = match Self::read_till_crlf(&src[..]) {
            Some((b, size)) => (b, size),
            None => return Ok(None),
        };

        if bytes_read < 4 || array_prefix_bytes[0] as char != '*' {
            return Err(RespError::InvalidArray(String::from(
                "Not a valid RESP array",
            )));
        }

        match Self::parse_usize_from_buf(&array_prefix_bytes[1..]) {
            Ok(len) => Ok(Some((len, bytes_read))),
            Err(e) => Err(e),
        }
    }

    /// Parses the length of a RESP bulk string from the given byte buffer.
    ///
    /// This function attempts to read the first few bytes of a RESP bulk string to determine its length.
    /// It expects the input to start with a '$' character followed by the length and terminated by CRLF.
    ///
    /// # Arguments
    ///
    /// * `src` - A `BytesMut` containing the bytes to parse.
    ///
    /// # Returns
    ///
    /// * `Ok(Some((usize, usize)))` - If successful, returns a tuple containing:
    ///   - The parsed length of the bulk string
    ///   - The number of bytes read from the input
    /// * `Ok(None)` - If there's not enough data in the buffer to parse the length
    /// * `Err(RespError)` - If the input is not a valid RESP bulk string prefix or if parsing fails
    ///
    pub fn parse_bulk_string_len(src: BytesMut) -> Result<Option<(usize, usize)>, RespError> {
        let (bulkstr_prefix_bytes, bytes_read) = match Self::read_till_crlf(&src[..]) {
            Some((b, size)) => (b, size),
            None => return Ok(None),
        };

        if bytes_read < 4 || bulkstr_prefix_bytes[0] as char != '$' {
            return Err(RespError::InvalidBulkString(String::from(
                "Not a valid RESP bulk string",
            )));
        }

        match Self::parse_usize_from_buf(&bulkstr_prefix_bytes[1..]) {
            Ok(len) => Ok(Some((len, bytes_read))),
            Err(e) => Err(e),
        }
    }

    // Read the bytes till reaching CRLF ("\r\n")
    fn read_till_crlf(buf: &[u8]) -> Option<(&[u8], usize)> {
        for i in 1..buf.len() {
            if buf[i - 1] == b'\r' && buf[i] == b'\n' {
                return Some((&buf[0..(i - 1)], i + 1));
            }
        }

        None
    }

    // Parse an integer from bytes
    fn parse_usize_from_buf(buf: &[u8]) -> Result<usize, RespError> {
        let utf8_str = String::from_utf8(buf.to_vec());
        let parsed_int = match utf8_str {
            Ok(s) => {
                let int = s.parse::<usize>();
                match int {
                    Ok(n) => Ok(n),
                    Err(_) => Err(RespError::Other(String::from(
                        "Invalid value for an integer",
                    ))),
                }
            }
            Err(_) => Err(RespError::Other(String::from("Invalid UTF-8 string"))),
        };

        parsed_int
    }
}
