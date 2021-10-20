
use std::str;

const MAX_NAME: usize = 255;


pub fn shorten_for(name: &str, n: usize) -> String {
    if name.len() < n {
        return name.to_owned();
    }
    let mut r: usize = n;
    let bs = name.as_bytes();
    while 0 < r {
        if let Ok(result) = str::from_utf8(&bs[0..r]) {
            return result.to_owned();
        }
        r -= 1;
    }
    panic!("WTF: no characters, name={:?}, n={:?}", name, n)
}

pub fn shorten_name_for(name: &str, n: usize) -> String {
    if let Some((l, r)) = name.rsplit_once('.') {
        if 5 < r.len() { // too long file exntesion, so it is not file extension
            return shorten_for(name, n)
        }
        let l = shorten_for(l, n - r.len() - 1);
        return format!("{}.{}", l, r)
    }

    shorten_for(name, n)
}


pub fn shorten_path_for(path: &str, n: usize) -> String {
    let mut result = "".to_owned();
    let mut at_first = true;
    for seg in path.split('/') {
        if at_first {
            at_first = false;
        } else {
            result.push('/');
        }
        result.push_str(&shorten_name_for(seg, n));
    }
    result
}

pub fn shorten_path(path: &str) -> String {
    shorten_path_for(path, MAX_NAME)
}
