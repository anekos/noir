use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use std::{thread, time};

use curl::easy::{Easy as EasyCurl, WriteError};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::errors::AppResultU;
use crate::loader;
use crate::tag::Tag;


#[derive(Debug, Serialize)]
pub struct Job {
    pub tags: Option<Tags>,
    pub to: PathBuf,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tags {
    pub items: Vec<String>,
    pub source: String
}

#[derive(Clone)]
pub struct Manager {
    tx: Sender<Job>
}

impl Manager {
    pub fn new(db: Database) -> Self {
        let (tx, rx) = channel::<Job>();

        thread::spawn(move || {
            let mut pool: VecDeque<Job> = VecDeque::new();
            let mut errors: usize = 0;

            loop {
                let before = pool.len();

                while let Ok(job) = rx.try_recv() {
                    pool.push_back(job);
                }

                let after = pool.len();
                let delta = after - before;
                if 0 < delta {
                    info!("Download: Queue: count={} delta={}", after, delta);
                }

                if let Some(job) = pool.pop_front() {
                    info!("Download: {:?}", job);
                    if let Err(err) = job.process(&db) {
                        errors += 1;
                        error!("Download: NG: {:?}", err);
                        if let Ok(json) = serde_json::to_string(&job) {
                            error!("NG-JOB: {}", json);
                        } else {
                            error!("NG-JOB: {:?}", job);
                        }
                    } else {
                        info!("Download: OK: {:?}", job.url);
                    }
                    info!("Download: Queue: count={}, errors={}", pool.len(), errors);
                }

                thread::sleep(time::Duration::from_secs(3));
            }
        });

        Self {tx}
    }

    pub fn download(&self, job: Job) {
        self.tx.send(job).unwrap();
    }
}

impl Job {
    fn process(&self, db: &Database) -> AppResultU {
        download(&self.url, &self.to)?;
        write_record(db, self)?;
        Ok(())
    }
}

fn download<T: AsRef<Path>>(url: &str, download_to: T) -> AppResultU {
    if let Some(parent) = download_to.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new()
        .read(false)
        .write(true).
        append(false)
        .create(true)
        .open(download_to)?;

    let mut curl = EasyCurl::new();

    curl.timeout(Duration::from_secs(60 * 5))?;
    curl.connect_timeout(Duration::from_secs(10))?;
    curl.low_speed_time(Duration::from_secs(30))?;
    curl.low_speed_limit(1024)?;
    curl.http_version(curl::easy::HttpVersion::V11)?;

    curl.url(url)?;

    curl.write_function(move |data| {
        if let Err(err) = file.write_all(data) {
            error!("Write error: {:?}", err);
            return Err(WriteError::Pause);
        }
        Ok(data.len())
    })?;

    curl.perform()?;

    let transfer = curl.transfer();
    transfer.perform()?;

    Ok(())
}

fn write_record(db: &Database, job: &Job) -> AppResultU {
    let config = loader::Config { compute_dhash: true, ..Default::default() };
    let mut loader = loader::Loader::new(db, config);
    let _tx = sql_retry!(db.transaction())?;
    loader.load_file(&job.to)?;
    if let Some(ref tags) = job.tags {
        let mut _tags = vec![];
        for tag in &tags.items {
            _tags.push(Tag::from_str(&tag)?);
        }
        let to = job.to.to_str().unwrap();
        db.add_tags(to, &_tags, &tags.source)?;
    }
    Ok(())
}
