
use maplit::hashmap;

use noir::alias::Alias;
use noir::expander::Expander;
use noir::expression::RawQuery;


fn r(expression: &str) -> RawQuery {
    RawQuery::new(expression.to_owned())
}


#[test]
fn test_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{}
    );

    assert_eq!(e.expand_str("begin hoge end").unwrap(), r("begin fuga end"));
    assert_eq!(e.expand_str("hoge end").unwrap(), r("fuga end"));
    assert_eq!(e.expand_str("begin hoge").unwrap(), r("begin fuga"));
    assert_eq!(e.expand_str("hoge").unwrap(), r("fuga"));
    assert_eq!(e.expand_str("<hoge>").unwrap(), r("<fuga>"));
}

#[test]
fn test_tag_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{}
    );

    assert_eq!(
        e.expand_str("begin #moge end").unwrap(),
        r("begin (path in (SELECT path FROM tags WHERE tag = 'moge')) end"));
    assert_eq!(
        e.expand_str("begin #moge").unwrap(),
        r("begin (path in (SELECT path FROM tags WHERE tag = 'moge'))"));

    assert_eq!(
        e.expand_str("begin #bang! X").unwrap(),
        r("begin (path in (SELECT path FROM tags WHERE tag = 'bang!')) X"));
}

#[test]
fn test_non_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{},
    );

    assert_eq!(e.expand_str("beginhogeend").unwrap(), r("beginhogeend"));
    assert_eq!(e.expand_str("a").unwrap(), r("a"));
    assert_eq!(e.expand_str("1").unwrap(), r("1"));
}

#[test]
fn test_tag_non_expandable() {
    let e = Expander::new(hashmap!{}, hashmap!{});

    assert_eq!(
        e.expand_str("begin #hoge end").unwrap(),
        r("begin (path in (SELECT path FROM tags WHERE tag = 'hoge')) end"));
}

#[test]
fn test_recursive() {
    let e = Expander::new(
        hashmap!{
            "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: true },
            "fuga".to_owned() => Alias { expression: "meow".to_owned(), recursive: false },
        },
        hashmap!{},
    );

    assert_eq!(
        e.expand_str("begin hoge end").unwrap(),
        r("begin meow end"));
}

#[test]
fn test_nonrecursive() {
    let e = Expander::new(
        hashmap!{
            "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false },
            "fuga".to_owned() => Alias { expression: "meow".to_owned(), recursive: false },
        },
        hashmap!{},
    );

    assert_eq!(
        e.expand_str("begin hoge end").unwrap(),
        r("begin fuga end"));
}
