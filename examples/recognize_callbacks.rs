use azure_speech::Auth;
use azure_speech::{recognizer, StreamExt};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "INFO");
    }
    tracing_subscriber::fmt::init();

    // Check on the example recognize_simple.rs for more details on how to set the recognizer.
    let auth = Auth::from_subscription(
        env::var("AZURE_REGION").expect("Region set on AZURE_REGION env"),
        env::var("AZURE_SUBSCRIPTION_KEY").expect("Subscription set on AZURE_SUBSCRIPTION_KEY env"),
    );
    let config = recognizer::Config::default();

    let client = recognizer::Client::connect(auth, config)
        .await
        .expect("to connect to azure");

    // Create a callbacks for the recognizer.
    // The callbacks are used to get information about the recognition process.
    let callbacks = recognizer::Callback::default()
        .on_start_detected(|id, offset| async move {
            tracing::info!("Start detected: {:?} - {:?}", id, offset);
        })
        .on_recognized(|id, result, _offset, _duration, _raw| async move {
            tracing::info!("Recognized: {:?} - {:?}", id, result);
        })
        .on_session_end(|id| async move {
            tracing::info!("Session end: {:?}", id);
        });
    //.on_... // check the other callbacks available.

    client
        .recognize_file("tests/audios/examples_sample_files_turn_on_the_lamp.wav")
        .await
        .expect("to recognize")
        // When you set the callbacks, the events will be sent to the callbacks and not to the stream.
        .use_callbacks(callbacks)
        .await; // it's important to await here.

    tracing::info!("Completed!");

    Ok(())
}
