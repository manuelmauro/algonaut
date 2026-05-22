// The dryrun printer renders an algod `DryrunResponse`, so it needs the algod client.
#[cfg(feature = "algod")]
pub mod dryrun_printer;

#[cfg(target_arch = "wasm32")]
pub async fn sleep(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(ms: u32) {
    futures_timer::Delay::new(std::time::Duration::from_millis(ms as u64)).await;
}
