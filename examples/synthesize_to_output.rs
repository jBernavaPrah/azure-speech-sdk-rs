
#[tokio::main]
async fn main() {

    // pub async fn synthesize_to_output_device(&self, speaks_rx: Receiver<Speak>, _device: cpal::Device) -> crate::Result<()> {
    //     let stream = self.synthesize(speaks_rx).await?;
    // 
    //     tokio::task::spawn_blocking::<_, crate::Result<()>>(move || {
    //         let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    // 
    //         let sink = rodio::Sink::try_new(&handle).unwrap();
    // 
    //         //let file = std::fs::File::open("tests/whatstheweatherlike.wav").unwrap();
    // 
    //         sink.append(rodio::Decoder::new(StreamMediaSource::new(stream.audio())).unwrap());
    // 
    //         sink.sleep_until_end();
    // 
    //         Ok(())
    //     }).await.expect("to run blocking task")?;
    // 
    //     Ok(())
    // }
    // 
    // pub async fn synthesize_to_default_output_device(&self, speaks_rx: Receiver<Speak>) -> crate::Result<()> {
    //     let host = cpal::default_host();
    //     let device = host.default_output_device()
    //         .ok_or(crate::Error::InternalError("Failed to get default input device".to_string()))?;
    // 
    //     self.synthesize_to_output_device(speaks_rx, device).await
    // }
    
}