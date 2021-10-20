
use noir::server::util::{
    shorten_for as sf,
    shorten_name_for as snf,
    shorten_path_for as spf
};


#[test]
fn test_shorten_for() {
    assert_eq!(sf("a", 5), "a".to_owned());
    assert_eq!(sf("b", 5), "b".to_owned());
    assert_eq!(sf("abcdefg", 5), "abcde".to_owned());
    assert_eq!(sf("abcd", 5), "abcd".to_owned());
    assert_eq!(sf("あい", 5), "あ".to_owned());
    assert_eq!(sf("あい", 6), "あい".to_owned());
    assert_eq!(sf("あいう", 6), "あい".to_owned());
    assert_eq!(sf("あいう", 7), "あい".to_owned());
    assert_eq!(sf("あいう", 8), "あい".to_owned());
    assert_eq!(sf("あいう", 9), "あいう".to_owned());
}

#[test]
fn test_shorten_name_for() {
    assert_eq!(snf("あいう.jpg", 9), "あ.jpg".to_owned());
    assert_eq!(snf("あいう.jpg123", 9), "あいう".to_owned());
    assert_eq!(snf("あいう.jpg123", 10), "あいう.".to_owned());
}

#[test]
fn test_shorten_path_for() {
    assert_eq!(spf("a", 5), "a".to_owned());
    assert_eq!(spf("abcdefg", 5), "abcde".to_owned());
    assert_eq!(spf("abcd", 5), "abcd".to_owned());
    assert_eq!(spf("あい", 5), "あ".to_owned());
    assert_eq!(spf("あいう", 8), "あい".to_owned());
    assert_eq!(spf("あいう", 9), "あいう".to_owned());

    assert_eq!(spf("foo/bar/あい", 5), "foo/bar/あ".to_owned());
    assert_eq!(spf("/foo/bar/あい", 5), "/foo/bar/あ".to_owned());
    assert_eq!(spf("foo/bar/あい/", 5), "foo/bar/あ/".to_owned());

    assert_eq!(spf("foo/bar/あい/か.jpg123", 5), "foo/bar/あ/か.j".to_owned());
}

