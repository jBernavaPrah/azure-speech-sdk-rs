use tokio::sync::mpsc::Receiver;
use crate::speech_to_text::config::ResolverConfig;
use crate::speech_to_text::event::{Event, TryFromMessage};

pub struct Recognizer;

impl Recognizer {
    pub async fn speech<T>(
        config: ResolverConfig,
    ) -> crate::errors::Result<Receiver<Event<T>>>
        where T: Send + TryFromMessage<T> + 'static
    {
        let resolver = crate::speech_to_text::client::RecognizerClient::connect(config).await?;

        Ok(resolver.event_rx)
    }

    // pub fn conversation(mut self) -> impl Future<Output=Result<Receiver<Event>>> {
    //     self.config.set_mode(RecognitionMode::Conversation);
    //     self.stream()
    // }
    // 
    // pub fn translate(mut self, _to_language: String) -> impl Future<Output=Result<Receiver<Event>>> {
    //     self.config.set_mode(RecognitionMode::Interactive);
    //     self.stream()
    // }
    // 
    // pub fn dictation(mut self) -> impl Future<Output=Result<Receiver<Event>>> {
    //     self.config.set_mode(RecognitionMode::Dictation);
    //     self.stream()
    // }

    // async fn connect_to_azure(&self, headers: &AudioHeaders) -> Result<Connector> {
    //     let mut connector = Connector::connect(generate_uri_for_stt_speech_azure(&self.config), Uuid::new_v4().to_string()).await?;
    // 
    //     connector.send(UpMessage::SpeechConfig(SpeechConfig::from_config(&self.config, headers))).await?;
    //     connector.send(UpMessage::SpeechContext(SpeechContext::from_config(self.config.clone()))).await?;
    // 
    //     connector.send(UpMessage::AudioHeaders {
    //         content_type: "audio/x-wav".to_string(),
    //         data: headers.to_vec(),
    //     }).await?;
    // 
    //     Ok(connector)
    // }
    // 
    // async fn stream(mut self) -> Result<Receiver<Event>> {
    //     let (tx, rx) = tokio::sync::mpsc::channel(1024);
    // 
    //     let connector = self.connect_to_azure(&self.source.headers()).await?;
    // 
    //     let (mut sender, mut receiver) = connector.split();
    // 
    //     tokio::spawn(async move {
    //         let mut buffer = Vec::new();
    //         while let Some(s) = &self.source.next().await {
    //             buffer.extend(s);
    //             if buffer.len() >= 4096 {
    //                 if let Err(_) = sender.send(UpMessage::Audio { data: buffer.clone() }).await {
    //                     error!("Failed to send buffer");
    //                 }
    //                 buffer.clear();
    //             }
    //         }
    // 
    //         // Send any remaining data in the buffer
    //         if !buffer.is_empty() && sender.send(UpMessage::Audio { data: buffer }).await.is_err() {
    //             error!("Failed to send buffer");
    //         }
    // 
    //         let _ = sender.send(UpMessage::EndAudio).await;
    //     });
    // 
    //     // Receive Events from the mod
    //     tokio::spawn(async move {
    //         while let Some(r) = receiver.next().await {
    //             match r {
    //                 Ok(message) => {
    //                     match tx.send(message.into()).await {
    //                         Ok(_) => (),
    //                         Err(e) => {
    //                             trace!("Failed to send response: {:?}", e);
    //                             break;
    //                         }
    //                     }
    //                 }
    //                 Err(_) => break
    //             }
    //         }
    // 
    //         debug!("Azure dropped stream.");
    // 
    //         drop(tx)
    //     });
    // 
    //     Ok(rx)
    // }
}
