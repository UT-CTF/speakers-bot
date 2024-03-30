use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use tokio::time::sleep;

pub(crate) async fn play_doorbell() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    stream_handle
        .play_raw(
            Decoder::new(BufReader::new(File::open("doorbell.ogg").unwrap()))
                .unwrap()
                .convert_samples(),
        )
        .unwrap();
    sleep(Duration::from_secs(5)).await;
}
