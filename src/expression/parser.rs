extern crate nom;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char as cchar, none_of, one_of};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated};

use crate::errors::AppResult;

use super::{Expression as E, NoirQuery};


const DELIMITERS: &'static str = "\t \r\n()<>=";


fn any(input: &str) -> IResult<&str, E> {
    let (rest, x) = anychar(input)?;
    Ok((rest, E::Any(x)))
}

fn delimiter(input: &str) -> IResult<&str, E> {
    let (rest, x) = many1(one_of(DELIMITERS))(input)?;
    Ok((rest, E::Delimiter(x.iter().collect())))
}

fn noir_tag(input: &str) -> IResult<&str, E> {
    let (rest, _) = cchar('#')(input)?;
    let (rest, y) = many1(none_of("\"() \t\r\n"))(rest)?;
    Ok((rest, E::NoirTag(y.iter().collect())))
}

fn path_segment(input: &str) -> IResult<&str, E> {
    let (rest, x) = delimited(cchar('`'), many0(none_of("`")), cchar('`'))(input)?;
    Ok((rest, E::PathSegment(x.iter().collect())))
}

fn string_literal(input: &str) -> IResult<&str, E> {
    let ch = alt((preceded(cchar('\''), cchar('\'')), none_of("'")));

    let (rest, result) = terminated(
        preceded(
            tag("'"),
            many0(ch)
        ),
        tag("'")
    )(input)?;
    Ok((rest, E::StringLiteral(result.iter().collect())))
}

fn term(input: &str) -> IResult<&str, E> {
    let (rest, x) = many1(none_of(DELIMITERS))(input)?;
    Ok((rest, E::Term(x.iter().collect())))
}

pub fn parse(input: &str) -> AppResult<NoirQuery> {
    let p = alt((noir_tag, string_literal, path_segment, term, delimiter, any));
    let (_rest, elements) = many0(p)(input)?;
    Ok(NoirQuery { elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter() {
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

    #[test]
    fn test_noir_tag() {
        use E::{NoirTag as T};

        assert_eq!(
            noir_tag(r#"#foo"#),
            Ok(("", T("foo".to_owned())))
        );
    }

    #[test]
    fn test_path_segment() {
        use E::{PathSegment as P};

        assert_eq!(
            path_segment(r#"`hoge`"#),
            Ok(("", P("hoge".to_owned())))
        );
        assert_eq!(
            path_segment(r#"`ho'ge`"#),
            Ok(("", P("ho'ge".to_owned())))
        );
        assert_eq!(
            path_segment(r#"`ho''ge`"#),
            Ok(("", P("ho''ge".to_owned())))
        );
        assert_eq!(
            path_segment(r#"`ho(g)e`"#),
            Ok(("", P("ho(g)e".to_owned())))
        );
    }

    #[test]
    fn test_string_literal() {
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
            string_literal(r#"''''"#),
            Ok(("", S("'".to_owned())))
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

        assert_eq!(
            string_literal(r#"'()'"#),
            Ok(("", S("()".to_owned())))
        );
        assert_eq!(
            string_literal(r#"'('')'"#),
            Ok(("", S("(')".to_owned())))
        );
    }

    #[test]
    fn test_term() {
        use E::{Term as T};

        assert_eq!(
            term(r#"cat-dog"#),
            Ok(("", T("cat-dog".to_owned())))
        );
        assert_eq!(
            term(r#"cat"#),
            Ok(("", T("cat".to_owned())))
        );
        assert_eq!(
            term(r#"cat and dog"#),
            Ok((" and dog", T("cat".to_owned())))
        );
        assert_eq!(
            term(r#"cat-dog "#),
            Ok((" ", T("cat-dog".to_owned())))
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse(r#"()"#).unwrap(),
            NoirQuery {
                elements: vec![
                    E::Delimiter("()".to_owned())
                ]});

        assert_eq!(
            parse(r#"(')'"#).unwrap(),
            NoirQuery {
                elements: vec![
                    E::Delimiter("(".to_owned()),
                    E::StringLiteral(")".to_owned())
                ]});

        assert_eq!(
            parse(r#"(#tag)"#).unwrap(),
            NoirQuery {
                elements: vec![
                    E::Delimiter("(".to_owned()),
                    E::NoirTag("tag".to_owned()),
                    E::Delimiter(")".to_owned()),
                ]});

        assert_eq!(
            parse(r#"('#tag')"#).unwrap(),
            NoirQuery {
                elements: vec![
                    E::Delimiter("(".to_owned()),
                    E::StringLiteral("#tag".to_owned()),
                    E::Delimiter(")".to_owned()),
                ]});
    }
}
