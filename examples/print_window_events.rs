use window_observer::{self, Event, MaybeWindowAvailable, WindowObserver};

fn print_event(event: MaybeWindowAvailable) {
    match event {
        MaybeWindowAvailable::Available { window, event } => {
            println!("\n{event:?}");
            println!("\tWindow title: {:?}", window.title());

            match event {
                Event::Moved => {
                    println!("\tWindow position: {:?}", window.position());
                }
                Event::Resized => {
                    println!("\tWindow size: {:?}", window.size());
                }
                _ => {}
            }
        }
        MaybeWindowAvailable::NotAvailable { event } => {
            println!("\n{event:?}");
        }
    };
}

#[tokio::main]
async fn main() {
    let pid = std::env::var("PID")
        .map(|v| v.parse().unwrap())
        .expect("Please give me the env `PID` of application that has window.");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_filter = window_observer::EventFilter::all();

    let _window_observer = WindowObserver::start(pid, event_tx, event_filter)
        .await
        .unwrap();

    while let Some(event) = event_rx.recv().await {
        match event {
            Ok(event) => print_event(event),
            Err(e) => eprintln!("Error occurred during handling event: {e:#?}"),
        }
    }
}
