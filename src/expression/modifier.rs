
use crate::errors::AppResult;

use super::{Expression as E, NoirQuery};



pub fn replace_tag(query: NoirQuery, tag: &str) -> AppResult<Option<NoirQuery>> {
    let mut elements: Vec<E> = vec![];
    let mut replaced = false;

    for e in query.elements {
        if !replaced {
            if let E::NoirTag(_) = e {
                elements.push(E::NoirTag(tag.to_owned()));
                replaced = true;
                continue;
            }
        }
        elements.push(e);
    }

    if replaced {
        return Ok(Some(NoirQuery { elements }))
    }

    Ok(None)
}
