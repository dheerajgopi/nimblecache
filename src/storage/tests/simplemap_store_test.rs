use crate::storage::store::Store;

#[test]
fn test_simplemap_for_key_with_some_value() {
    let store = Store::new_simple_map();
    let k = "a";
    let v = "b";
    store.put(k.into(), v.into());

    let fetched_val = store.get(k);
    assert_eq!(true, fetched_val.is_some());
    assert_eq!(v, fetched_val.unwrap());
}

#[test]
fn test_simplemap_for_key_with_no_value() {
    let store = Store::new_simple_map();
    let fetched_val = store.get("a");
    assert_eq!(true, fetched_val.is_none());
}

#[test]
fn test_simplemap_for_key_with_updated_value() {
    let store = Store::new_simple_map();
    let k = "a";
    let v = "b";
    store.put(k.into(), v.into());
    let updated_v = "c";
    store.put(k.into(), updated_v.into());

    let fetched_val = store.get(k);
    assert_eq!(true, fetched_val.is_some());
    assert_eq!(updated_v, fetched_val.unwrap());
}
