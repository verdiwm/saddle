use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Result;
use saddle::Seat;
use tokio::{pin, time::sleep};
use tokio_stream::StreamExt;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let seat = Seat::new().await?;

    let has_control = Arc::new(RwLock::new(false));

    tokio::spawn({
        let seat = seat.clone();
        let has_control = has_control.clone();

        async move {
            let stream = seat.active_stream().await;

            pin!(stream);

            while let Some(is_active) = stream.try_next().await? {
                if is_active {
                    info!("Session became active, taking control");
                    seat.aquire_session().await?;
                    *has_control.write().unwrap() = true;
                } else {
                    info!("Session became inactive");
                    seat.release_session().await?;
                    *has_control.write().unwrap() = false;
                }
            }

            anyhow::Ok(())
        }
    });

    sleep(Duration::from_secs(5)).await;

    Ok(())
}
