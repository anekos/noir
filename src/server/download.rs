use std::path::Path;
use std::{fs::OpenOptions, io::Write};

use curl::easy::Easy as EasyCurl;

use crate::errors::AppResultU;


pub fn download<T: AsRef<Path>>(url: &str, download_to: T) -> AppResultU {
    let mut file = OpenOptions::new()
        .read(false)
        .write(true).
        append(false)
        .create(true)
        .open(download_to).unwrap();

    let mut curl = EasyCurl::new();
    curl.url(url).unwrap();

    curl.write_function(move |data| {
        file.write_all(&data).unwrap();
        Ok(data.len())
    }).unwrap();

    curl.perform().unwrap();

    let transfer = curl.transfer();
    transfer.perform()?;

    Ok(())
}
