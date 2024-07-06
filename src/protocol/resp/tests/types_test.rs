use crate::protocol::resp::types::RespType;
use bytes::BytesMut;

#[test]
fn test_valid_simple_string_creation() {
    let s = "+Hello\r\n";
    let bytes = BytesMut::from(s);
    let ss = RespType::new_simple_string(bytes);

    assert_eq!(false, ss.is_err());

    let (ss, size) = ss.unwrap();
    assert_eq!(s, ss.serialize());
    assert_eq!(s.len(), size);
}

#[test]
fn test_simple_string_creation_with_no_crlf() {
    let s = "+Hello";
    let bytes = BytesMut::from(s);
    let ss = RespType::new_simple_string(bytes);

    assert_eq!(true, ss.is_err());
}

#[test]
fn test_valid_bulk_string_creation() {
    let s = "$5\r\nHello\r\n";
    let bytes = BytesMut::from(s);
    let bs = RespType::new_bulk_string(bytes);

    assert_eq!(false, bs.is_err());

    let (bs, size) = bs.unwrap();
    assert_eq!(s, bs.serialize());
    assert_eq!(s.len(), size);
    assert!(matches!(bs, RespType::BulkString { .. }))
}

#[test]
fn test_bulk_string_creation_with_no_crlf() {
    let s = "$5Hello";
    let bytes = BytesMut::from(s);
    let bs = RespType::new_bulk_string(bytes);

    assert_eq!(true, bs.is_err());
}

#[test]
fn test_bulk_string_creation_with_no_string_length() {
    let s = "$\r\nHello\r\n";
    let bytes = BytesMut::from(s);
    let bs = RespType::new_bulk_string(bytes);

    assert_eq!(true, bs.is_err());
}

#[test]
fn test_bulk_string_creation_with_invalid_string_length() {
    let s = "$a\r\nHello\r\n";
    let bytes = BytesMut::from(s);
    let bs = RespType::new_bulk_string(bytes);

    assert_eq!(true, bs.is_err());
}

#[test]
fn test_bulk_string_creation_with_excess_string_length() {
    let s = "$10\r\nHello\r\n";
    let bytes = BytesMut::from(s);
    let bs = RespType::new_bulk_string(bytes);

    assert_eq!(true, bs.is_err());
}

#[test]
fn test_valid_array_creation() {
    let s = "*2\r\n$3\r\nSan\r\n$9\r\nFrancisco\r\n";
    let bytes = BytesMut::from(s);
    let arr = RespType::new_array(bytes);

    assert_eq!(false, arr.is_err());

    let (arr, size) = arr.unwrap();
    assert_eq!(s, arr.serialize());
    assert_eq!(s.len(), size);
    assert!(matches!(arr, RespType::Array { .. }));

    let vec = match arr {
        RespType::Array(v) => Some(v),
        _ => None,
    };

    assert_eq!(true, vec.is_some());

    let vec = vec.unwrap();
    assert_eq!(2, vec.len());
    assert!(matches!(vec[0], RespType::BulkString { .. }));
    assert_eq!("$3\r\nSan\r\n", vec[0].serialize());
    assert!(matches!(vec[1], RespType::BulkString { .. }));
    assert_eq!("$9\r\nFrancisco\r\n", vec[1].serialize());
}

#[test]
fn test_array_creation_with_invalid_length() {
    let s = "*3\r\n$3\r\nSan\r\n$9\r\nFrancisco\r\n";
    let bytes = BytesMut::from(s);
    let arr = RespType::new_array(bytes);

    assert_eq!(true, arr.is_err());
}

#[test]
fn test_array_creation_with_invalid_item() {
    let s = "*3\r\ninvalid3\r\nSan\r\n$9\r\nFrancisco\r\n";
    let bytes = BytesMut::from(s);
    let arr = RespType::new_array(bytes);

    assert_eq!(true, arr.is_err());
}

#[test]
fn test_valid_simple_error_creation() {
    let s = "-Error\r\n";
    let bytes = BytesMut::from(s);
    let se = RespType::new_simple_error(bytes);

    assert_eq!(false, se.is_err());

    let (ss, size) = se.unwrap();
    assert_eq!(s, ss.serialize());
    assert_eq!(s.len(), size);
}

#[test]
fn test_simple_error_creation_with_no_crlf() {
    let s = "-Error";
    let bytes = BytesMut::from(s);
    let se = RespType::new_simple_error(bytes);

    assert_eq!(true, se.is_err());
}

#[test]
fn test_valid_null_bulk_string_creation() {
    let s = "$-1\r\n";
    let nbs = RespType::null_bulk_string();
    assert_eq!(s, nbs.serialize());
}
