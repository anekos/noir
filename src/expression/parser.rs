extern crate nom;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char as cchar, none_of, alphanumeric1, multispace1, one_of};
use nom::multi::{many0, many1, fold_many0, fold_many1};
use nom::sequence::{preceded, terminated};

use super::{Expression as E};


fn any(input: &str) -> IResult<&str, E> {
    let (rest, x) = anychar(input)?;
    Ok((rest, E::Any(x)))
}

fn ctor_str(mut acc: String, it: char) -> String {
    acc.push(it);
    acc
}

fn delimiter(input: &str) -> IResult<&str, E> {
    let (rest, x) = multispace1(input)?;
    Ok((rest, E::Delimiter(x.to_owned())))
}

#[test]
fn parse_delimiter() {
    use E::{Delimiter as D};

    assert_eq!(
        delimiter(" \t"),
        Ok(("", D(" \t".to_owned())))
    );

    assert_eq!(
        delimiter(" \t\r\n123"),
        Ok(("123", D(" \t\r\n".to_owned())))
    );
}

fn noir_tag(input: &str) -> IResult<&str, E> {
    let (rest, _) = cchar('#')(input)?;
    let (rest, y) = many1(none_of("\"() \t\r\n"))(rest)?;
    Ok((rest, E::NoirTag(y.iter().collect())))
}

#[test]
fn parse_noir_tag() {
    use E::{NoirTag as T};

    assert_eq!(
        noir_tag(r#"#foo"#),
        Ok(("", T("foo".to_owned())))
    );
}

fn term(input: &str) -> IResult<&str, E> {
    let (rest, y) = alphanumeric1(input)?;
    Ok((rest, E::Term(y.to_owned())))
}

#[test]
fn parse_term() {
    use E::{Term as T};

    assert_eq!(
        term(r#"cat"#),
        Ok(("", T("cat".to_owned())))
    );
    assert_eq!(
        term(r#"cat and dog"#),
        Ok((" and dog", T("cat".to_owned())))
    );
}

fn string_literal(input: &str) -> IResult<&str, E> {
    let ch = alt((preceded(cchar('\''), cchar('\'')), none_of("'")));

    let (rest, result) = terminated(
        preceded(
            tag("'"),
            fold_many0(ch, String::new, ctor_str)
        ),
        tag("'")
    )(input)?;
    Ok((rest, E::StringLiteral(result)))
}

#[test]
fn parse_string_literal() {
    use E::{StringLiteral as S};

    assert_eq!(
        string_literal(r#"'cat'"#),
        Ok(("", S("cat".to_owned())))
    );
    assert_eq!(
        string_literal(r#"'A'"#),
        Ok(("", S("A".to_owned())))
    );

    assert_eq!(
        string_literal(r#"'A'''"#),
        Ok(("", S("A'".to_owned())))
    );
    assert_eq!(
        string_literal(r#"'A''B'"#),
        Ok(("", S("A'B".to_owned())))
    );
    assert_eq!(
        string_literal(r#"'A''B''C'"#),
        Ok(("", S("A'B'C".to_owned())))
    );
    assert_eq!(
        string_literal(r#"'A''B''''C'"#),
        Ok(("", S("A'B''C".to_owned())))
    );
    assert_eq!(
        string_literal(r#"'A''B''''C'RR"#),
        Ok(("RR", S("A'B''C".to_owned())))
    );
}

fn symbol(input: &str) -> IResult<&str, E> {
    let p = one_of("()<>=");
    let (rest, x) = fold_many1(p, String::new, ctor_str)(input)?;
    Ok((rest, E::Symbol(x)))
}

pub fn parse(input: &str) -> IResult<&str, Vec<E>> {
    let p = alt((noir_tag, term, symbol, string_literal, delimiter, any));
    many0(p)(input)
}

#[test]
fn test_parse() {
    assert_eq!(
        parse(r#"()"#),
        Ok(
            ("",
             vec![
                 E::Symbol("()".to_owned())
             ])));
}
