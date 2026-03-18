use crate::services::hasher::sha1;

#[test]
fn it_should_hash_an_string() {
    assert_eq!(sha1("hello world"), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
}
