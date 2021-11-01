
use maplit::hashmap;

use noir::alias::Alias;
use noir::expander::Expander;



#[test]
fn test_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{},
        vec![]);

    assert_eq!(e.expand("begin hoge end"), "begin fuga end".to_owned());
    assert_eq!(e.expand("hoge end"), "fuga end".to_owned());
    assert_eq!(e.expand("begin hoge"), "begin fuga".to_owned());
    assert_eq!(e.expand("hoge"), "fuga".to_owned());
    assert_eq!(e.expand("<hoge>"), "<fuga>".to_owned());
}

#[test]
fn test_tag_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{},
        vec!["moge".to_owned(), "bang!".to_owned()]);

    assert_eq!(
        e.expand("begin #moge end"),
        "begin (path in (SELECT path FROM tags WHERE tag = 'moge')) end".to_owned());
    assert_eq!(
        e.expand("begin #moge"),
        "begin (path in (SELECT path FROM tags WHERE tag = 'moge'))".to_owned());

    assert_eq!(
        e.expand("begin #bang! X"),
        "begin (path in (SELECT path FROM tags WHERE tag = 'bang!')) X".to_owned());
}

#[test]
fn test_non_expandable() {
    let e = Expander::new(
        hashmap!{ "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false }},
        hashmap!{},
        vec![]);

    assert_eq!(e.expand("beginhogeend"), "beginhogeend".to_owned());
    assert_eq!(e.expand("a"), "a".to_owned());
    assert_eq!(e.expand("1"), "1".to_owned());
}

#[test]
fn test_tag_non_expandable() {
    let e = Expander::new(
        hashmap!{},
        hashmap!{},
        vec![]);

    assert_eq!(e.expand("begin #hoge end"), "begin #hoge end".to_owned());
}

#[test]
fn test_recursive() {
    let e = Expander::new(
        hashmap!{
            "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: true },
            "fuga".to_owned() => Alias { expression: "meow".to_owned(), recursive: false },
        },
        hashmap!{},
        vec![]);

    assert_eq!(e.expand("begin hoge end"), "begin meow end".to_owned());
}

#[test]
fn test_nonrecursive() {
    let e = Expander::new(
        hashmap!{
            "hoge".to_owned() => Alias { expression: "fuga".to_owned(), recursive: false },
            "fuga".to_owned() => Alias { expression: "meow".to_owned(), recursive: false },
        },
        hashmap!{},
        vec![]);

    assert_eq!(e.expand("begin hoge end"), "begin fuga end".to_owned());
}
