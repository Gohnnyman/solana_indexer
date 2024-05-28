#[macro_export]
macro_rules! repeat_until_ok {
    ( $func:expr, $sleep_time:expr ) => {{
        loop {
            match $func {
                Ok(result) => break result,
                Err(err) => {
                    log::error!("Error in func {}: {}", stringify!($func), err);
                    tokio::time::sleep(std::time::Duration::from_secs($sleep_time)).await;
                }
            }
        }
    }};
}
