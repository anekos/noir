
use maplit::hashmap;

use noir::alias::Alias;
use noir::expander::Expander;



#[test]
fn test_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{}
    );

    assert_eq!(e.expand("begin hoge end").unwrap(), "begin fuga end".to_owned());
    assert_eq!(e.expand("hoge end").unwrap(), "fuga end".to_owned());
    assert_eq!(e.expand("begin hoge").unwrap(), "begin fuga".to_owned());
    assert_eq!(e.expand("hoge").unwrap(), "fuga".to_owned());
    assert_eq!(e.expand("<hoge>").unwrap(), "<fuga>".to_owned());
}

#[test]
fn test_tag_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{}
    );

    assert_eq!(
        e.expand("begin #moge end").unwrap(),
        "begin (path in (SELECT path FROM tags WHERE tag = 'moge')) end".to_owned());
    assert_eq!(
        e.expand("begin #moge").unwrap(),
        "begin (path in (SELECT path FROM tags WHERE tag = 'moge'))".to_owned());

    assert_eq!(
        e.expand("begin #bang! X").unwrap(),
        "begin (path in (SELECT path FROM tags WHERE tag = 'bang!')) X".to_owned());
}

#[test]
fn test_non_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{},
    );

    assert_eq!(e.expand("beginhogeend").unwrap(), "beginhogeend".to_owned());
    assert_eq!(e.expand("a").unwrap(), "a".to_owned());
    assert_eq!(e.expand("1").unwrap(), "1".to_owned());
}

#[test]
fn test_tag_non_expandable() {
    let e = Expander::new(hashmap!{}, hashmap!{});

    assert_eq!(
        e.expand("begin #hoge end").unwrap(),
        "begin (path in (SELECT path FROM tags WHERE tag = 'hoge')) end".to_owned());
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
        e.expand("begin hoge end").unwrap(),
        "begin meow end".to_owned());
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
        e.expand("begin hoge end").unwrap(),
        "begin fuga end".to_owned());
}
