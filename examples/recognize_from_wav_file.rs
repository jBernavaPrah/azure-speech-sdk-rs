use std::env;
use azure_speech::{Auth, AzureSpeech, Device, Event, EventBase, EventSpeech, RecognizerConfig};

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
async fn main() {
    init_logging();


    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );

    let recognizer
        //:AzureSpeech<Recognizer, Connected> 
        = AzureSpeech::new(auth, Device::default())
        .recognizer(RecognizerConfig::default());
    //.connect().await.expect("connect failed");

    recognizer.on_raw(|raw| async move {
        tracing::info!("Received raw: {:?}", raw);
    })
        .on_recognized(|event| async move {
            tracing::info!("recognized: {event:?}");
        })
        .on_recongizing(|event| async move {
            tracing::info!("recognizing: {event:?}");
        })
        .on_canceled(|event| async move {
            tracing::info!("cancelled: {event:?}");
        })
        .on_session_started(|event| async move {
            tracing::info!("session started: {event:?}");
        });

    let stream = recognizer
        .recognize_from_default_microphone()
        .start()
        .await.expect("Start failed");

    recognizer.stop().await.expect("stop failed");

    recognizer.recognize_from_file("tests/audios/whatstheweatherlike.wav").await.expect("recognize failed");


    let mut receiver = recognizer
        // .on_raw(|raw| async move {
        //     tracing::info!("Received raw: {:?}", raw);
        // })
        // .on_recognized(|event| async move {
        //     tracing::info!("recognized: {event:?}");
        // })
        // .on_recongizing(|event| async move {
        //     tracing::info!("recognizing: {event:?}");
        // })
        // .on_canceled(|event| async move {
        //     tracing::info!("cancelled: {event:?}");
        // })
        // .on_session_started(|event| async move {
        //     tracing::info!("session started: {event:?}");
        // })

        .recognize_from_file("tests/audios/whatstheweatherlike.wav").await.expect("recognize failed");

    while let Some(event) = receiver.recv().await {
        match event {
            Event::Specific(EventSpeech::Recognized { text, .. }) => {
                tracing::info!("recognized: {text}");
            }
            Event::Base(EventBase::Cancelled { reason }) => {
                tracing::info!("cancelled: {reason}");
                break;
            }
            _ => {}
        }
    }

    tracing::info!("Completed!");
}