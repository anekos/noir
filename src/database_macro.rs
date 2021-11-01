#[macro_export]
macro_rules! sql_retry {
    ($body:expr) => {
        {
            use crate::errors::AppError;
            use std::thread;
            use std::time::Duration;
            use log;
            let rr;
            let mut tried = 0;
            loop {
                if 0 < tried {
                    log::error!("Start to retry: {}", tried);
                }
                let r = $body;
                if let Err(AppError::Sqlite(ref e)) = &r {
                    log::error!("{:?}", e);
                    if tried < 10 {
                        log::error!("Wait for retry: {}", tried);
                        tried += 1;
                        thread::sleep(Duration::from_secs(1 * tried));
                        continue;
                    } else {
                        log::error!("Give up: {}", tried);
                    }
                }
                rr = r;
                break;
            }
            rr
        }
    };
}



