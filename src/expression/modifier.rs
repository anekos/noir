
use crate::errors::AppResult;

use super::{Expression as E, NoirQuery};



pub fn replace_tag(query: NoirQuery, tag: &str) -> AppResult<NoirQuery> {
    let mut elements: Vec<E> = vec![];
    let mut at_first = true;

    for e in query.elements {
        if at_first {
            if let E::NoirTag(_) = e {
                elements.push(E::NoirTag(tag.to_owned()));
                at_first = false;
                continue;
            }
        }
        elements.push(e);
    }

    Ok(NoirQuery { elements })
}
