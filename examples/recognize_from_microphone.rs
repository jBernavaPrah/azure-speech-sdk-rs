use std::env;
use cpal::traits::StreamTrait;
use azure_speech::{Auth, AzureSpeech, Device, RecognizerConfig, Error, LanguageDetectMode};


fn init_logging() {
    let dir = tracing_subscriber::filter::Directive::from(tracing::Level::INFO);

    use std::io::stderr;
    use std::io::IsTerminal;
    use tracing_glog::Glog;
    use tracing_glog::GlogFields;
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let fmt = tracing_subscriber::fmt::Layer::default()
        .with_ansi(stderr().is_terminal())
        .with_writer(std::io::stderr)
        .event_format(Glog::default().with_timer(tracing_glog::LocalTime::default()))
        .fmt_fields(GlogFields::default().compact());

    let filter = vec![dir]
        .into_iter()
        .fold(EnvFilter::from_default_env(), |filter, directive| {
            filter.add_directive(directive)
        });

    let subscriber = Registry::default().with(filter).with(fmt);
    tracing::subscriber::set_global_default(subscriber).expect("to set global subscriber");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    
    init_logging();
    
    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let recognizer = AzureSpeech::new(auth, Device::default())
        .recognizer(RecognizerConfig::default()
            .set_detect_languages(vec![String::from("it-IT")], LanguageDetectMode::Continuous)
            
        );

    let (mut receiver, stream) = recognizer.recognize_from_default_microphone().await.expect("recognize failed");

    stream.play().expect("play failed");
    
    while let Some(event) = receiver.recv().await {
        tracing::info!("recognized: {event:?}");
        
        if let azure_speech::Event::Specific(azure_speech::EventSpeech::Recognized { text, .. }) = event {
            tracing::info!("recognized: {text}");
            if text.to_lowercase().contains("stop") {
                stream.pause().expect("pause failed");
                break;
            }
        }
        
    }
    
    tracing::info!("Completed!");
    
    
    
    
    Ok(())
}
