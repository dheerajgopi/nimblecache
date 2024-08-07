use bytes::{Buf, BufMut};
use core::fmt;
use std::io::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::resp::types::RespType;

use super::RespError;

/// A tokio_utils Frame codec for working with TCP streams as a `Sink` and `Stream` of `RespType`.
///
/// This codec specifically handles Nimblecache commands, which are always represented
/// as array of bulk strings in the RESP (REdis Serialization Protocol) protocol.
///
/// The codec uses a `CommandBuilder` internally to construct the array of bulk strings
/// that make up a Nimblecache command.
///
/// # Examples
///
/// ```
/// use tokio::net::TcpStream;
/// use tokio_util::codec::Framed;
/// use crate::resp::frame::RespCommandFrame;
///
/// async fn handle_connection(stream: TcpStream) {
///     let mut framed = Framed::new(stream, RespCommandFrame::new());
///     // Now you can use `framed` to send and receive Nimblecache commands as `RespType` objects
/// }
/// ```
pub struct RespCommandFrame {
    /// Builder for appending the bulk strings in the command array.
    cmd_builder: Option<CommandBuilder>,
}

impl RespCommandFrame {
    /// Creates a new `RespCommandFrame`.
    ///
    /// # Returns
    ///
    /// A new instance of `RespCommandFrame` with no command builder initialized.
    pub fn new() -> RespCommandFrame {
        RespCommandFrame { cmd_builder: None }
    }
}

impl Decoder for RespCommandFrame {
    type Item = Vec<RespType>;

    type Error = std::io::Error;

    /// Decodes bytes from the input stream into a `Vec<RespType>` representing a Nimblecache command.
    ///
    /// This method implements the RESP protocol decoding logic, specifically handling
    /// arrays of bulk strings which represent Nimblecache commands. It uses a `CommandBuilder`
    /// to accumulate the parts of the command as they are received.
    ///
    /// # Arguments
    ///
    /// * `src` - A mutable reference to the input buffer containing bytes to decode.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<RespType>))` if a complete command (array of bulk strings) was successfully decoded.
    /// * `Ok(None)` if more data is needed to complete the command.
    /// * `Err(std::io::Error)` if an error occurred during decoding.
    fn decode(
        &mut self,
        src: &mut bytes::BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        // A command in RESP protocol should always be an array of Bulk Strings.
        // Check the first 2 bytes to validate if its a RESP array.
        if self.cmd_builder.is_none() {
            let (cmd_len, bytes_read) = match RespType::parse_array_len(src.clone()) {
                Ok(arr_len) => match arr_len {
                    Some((len, bytes_read)) => (len, bytes_read),
                    None => return Ok(None),
                },
                Err(e) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        FrameError::from(e),
                    ));
                }
            };

            // initilize command builder, if its a valid RESP array.
            self.cmd_builder = Some(CommandBuilder::new(cmd_len));

            // advance buffer
            src.advance(bytes_read);
        }

        // Read all bytes in buffer
        while src.len() != 0 {
            // Validate and check the length of next bulk string
            let (bulkstr_len, bytes_read) = match RespType::parse_bulk_string_len(src.clone()) {
                Ok(bulkstr_len) => match bulkstr_len {
                    Some((len, bytes_read)) => (len, bytes_read),
                    None => return Ok(None),
                },
                Err(e) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        FrameError::from(e),
                    ));
                }
            };

            // A bulk string has the below format
            //
            // `${string length in bytes }\r\n{string value}\r\n`
            //
            // Check if the buffer contains the required number of bytes to parse
            // the bulk string (including the CRLF at the end)
            let bulkstr_bytes = bulkstr_len + bytes_read + 2;
            if src.len() < bulkstr_bytes {
                return Ok(None);
            }

            // now that its sure the buffer has all the bytes required to parse the bulk string, parse it.
            let (bulkstr, bytes_read) = match RespType::new_bulk_string(src.clone()) {
                Ok((resp_type, bytes_read)) => (resp_type, bytes_read),
                Err(e) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        FrameError::from(e),
                    ));
                }
            };

            // append the bulk string to the command builder
            self.cmd_builder.as_mut().unwrap().add_part(bulkstr);

            // advance buffer
            src.advance(bytes_read);

            // if the command builder has all the parts, return it, else check buffer again
            let cmd_builder = self.cmd_builder.as_ref().unwrap();
            if cmd_builder.all_parts_received() {
                let cmd = cmd_builder.build();
                self.cmd_builder = None;
                return Ok(Some(cmd));
            }
        }

        Ok(None)
    }
}

impl Encoder<RespType> for RespCommandFrame {
    type Error = std::io::Error;

    /// Encodes a `RespType` into bytes and writes them to the output buffer.
    ///
    /// It's primarily used for sending responses to Nimblecache commands.
    ///
    /// # Arguments
    ///
    /// * `item` - The `RespType` to encode.
    /// * `dst` - The output buffer to write the encoded bytes to.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the encoding was successful.
    /// * `Err(std::io::Error)` if an error occurred during encoding.
    fn encode(&mut self, item: RespType, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.put_slice(&item.to_bytes());

        Ok(())
    }
}

/// This struct is used to accumulate the parts of a Nimblecache command, which are
/// typically represented as an array of bulk strings in the RESP protocol.
struct CommandBuilder {
    parts: Vec<RespType>,
    num_parts: usize,
    parts_parsed: usize,
}

impl CommandBuilder {
    /// Creates a new `CommandBuilder` with the specified number of parts.
    pub fn new(num_parts: usize) -> CommandBuilder {
        CommandBuilder {
            parts: vec![],
            num_parts,
            parts_parsed: 0,
        }
    }

    /// Adds a part to the command being built and increments the count of parsed parts.
    ///
    /// # Arguments
    ///
    /// * `part` - A `RespType` representing a part of the command.
    pub fn add_part(&mut self, part: RespType) {
        self.parts.push(part);
        self.parts_parsed += 1;
    }

    /// Checks if all expected parts of the command have been received.
    ///
    /// # Returns
    ///
    /// `true` if the number of parsed parts equals the expected number of parts,
    /// `false` otherwise.
    pub fn all_parts_received(&self) -> bool {
        self.num_parts == self.parts_parsed
    }

    /// Builds and returns the complete command as a vector of RESP values.
    ///
    /// # Returns
    ///
    /// A vector of `RespType` containing all the parts of the command.
    pub fn build(&self) -> Vec<RespType> {
        self.parts.clone()
    }
}

/// Represents error that can occur during RESP command frame parsing.
#[derive(Debug)]
pub struct FrameError {
    err: RespError,
}

impl FrameError {
    pub fn from(err: RespError) -> FrameError {
        FrameError { err }
    }
}

impl std::error::Error for FrameError {}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}
