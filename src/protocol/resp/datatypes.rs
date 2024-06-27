use bytes::BytesMut;
use anyhow::{anyhow, Result};

#[derive(Clone, Debug)]
pub enum DataType {
    SimpleString(String),
    BulkString(String),
    Array(Vec<DataType>),
}

impl DataType {
    pub fn parse(buffer: BytesMut) -> Result<(DataType, usize)> {
        let c = buffer[0] as char;
        return match c {
            '+' => {
                Self::new_simple_string(buffer)
            }
            '$' => {
                Self::new_bulk_string(buffer)
            }
            '*' => {
                Self::new_array(buffer)
            }
            _ => {
                Err(anyhow!("Invalid RESP data type {:?}", buffer))
            }
        }
    }

    pub fn new_simple_string(buffer: BytesMut) -> Result<(DataType, usize)> {
        if let Some((buf_data, len)) = Self::read_till_clrf(&buffer[1..]) {
            let utf8_str = String::from_utf8(buf_data.to_vec());

            return match utf8_str {
                Ok(simple_str) => {
                    Ok((DataType::SimpleString(simple_str), len + 1))
                }
                Err(_) => {
                    Err(anyhow!("Invalid UTF-8 string {:?}", buffer))
                }
            }
        }

        return Err(anyhow!("Invalid value for simple string {:?}", buffer))
    }

    pub fn new_bulk_string(buffer: BytesMut) -> Result<(DataType, usize)> {
        let (bulk_str_len, bytes_consumed) = if let Some((buf_data, len)) = Self::read_till_clrf(&buffer[1..]) {
            let bulk_str_len = Self::parse_int_from_buf(buf_data)?;
            (bulk_str_len, len + 1)
        } else {
            return Err(anyhow!("Invalid value for bulk string {:?}", buffer))
        };

        let bulk_str_end_idx = bytes_consumed + bulk_str_len as usize;
        let bulk_str = String::from_utf8(buffer[bytes_consumed..bulk_str_end_idx].to_vec());

        match bulk_str {
            Ok(bs) => {
                Ok((DataType::BulkString(bs), bulk_str_end_idx + 2))
            }
            Err(_) => Err(anyhow!("Invalid UTF-8 string {:?}", buffer))
        }
    }

    pub fn new_array(buffer: BytesMut) -> Result<(DataType, usize)> {
        let (arr_len, mut bytes_consumed) = if let Some((buf_data, len)) = Self::read_till_clrf(&buffer[1..]) {
            let arr_len = Self::parse_int_from_buf(buf_data)?;
            (arr_len, len + 1)
        } else {
            return Err(anyhow!("Invalid value for array {:?}", buffer))
        };

        let mut items: Vec<DataType> = vec![];
        for _ in 0..arr_len {
            let item = Self::parse(BytesMut::from(&buffer[bytes_consumed..]));
            match item {
                Ok((data, bytes_read)) => {
                    items.push(data);
                    bytes_consumed += bytes_read;
                }
                Err(e) => {
                    return Err(e)
                }
            }
        }

        return Ok((DataType::Array(items), bytes_consumed));
    }

    pub fn serialize(&self) -> String {
        return match self {
            DataType::SimpleString(ss) => format!("+{}\r\n", ss),
            DataType::BulkString(bs) => format!("${}\r\n{}\r\n", bs.chars().count(), bs),
            DataType::Array(arr) => {
                let mut ser_array = String::from(format!("*{}\r\n", arr.len()));
                ser_array.push_str(arr.iter().map(|v| v.serialize()).collect::<String>().as_str());

                ser_array
            }
        }
    }

    // Read the bytes till reaching "\r\n"
    fn read_till_clrf(buf: &[u8]) -> Option<(&[u8], usize)> {
        for i in 1..buf.len() {
            if buf[i-1] == b'\r' && buf[i] == b'\n' {
                return Some((&buf[0..(i-1)], i+1));
            }
        }

        None
    }

    // Parse an integer from bytes
    fn parse_int_from_buf(buf: &[u8]) -> Result<i64> {
        let utf8_str = String::from_utf8(buf.to_vec());
        let parsed_int = match utf8_str {
            Ok(s) => {
                let int = s.parse::<i64>();
                match int {
                    Ok(n) => {
                        Ok(n)
                    }
                    Err(_) => {
                        Err(anyhow!("Invalid value for an integer {:?}", s))
                    }
                }
            }
            Err(_) => Err(anyhow!("Invalid UTF-8 string {:?}", buf))
        };

        parsed_int
    }
}