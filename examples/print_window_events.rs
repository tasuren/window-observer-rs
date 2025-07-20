use window_observer::{self, Event, WindowObserver};

fn print_event(event: Event) {
    match event {
        Event::Foregrounded { window } => {
            println!("Window {:?}: Foregrounded", window.title())
        }
        Event::Focused { window } => {
            println!("Window {:?}: Focused", window.title())
        }
        Event::Unfocused { window } => {
            println!("Window {:?}: Unfocused", window.title())
        }
        Event::Moved { window } => {
            println!("Window {:?}: Moved {:?}", window.title(), window.position())
        }
        Event::Resized { window } => {
            println!("Window {:?}: Resized {:?}", window.title(), window.size())
        }
        Event::Backgrounded { window } => {
            println!("Window {:?}: Backgrounded", window.title())
        }
        Event::Created { window } => {
            println!("Window {:?}: Created", window.title())
        }
        Event::Closed { window_id } => {
            println!("Window {window_id:?}: Closed")
        }
        _ => {}
    };
}

#[tokio::main]
async fn main() {
    let pid = std::env::var("PID")
        .map(|v| v.parse().unwrap())
        .expect("Please give me the env `PID` of application that has window.");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut event_filter = window_observer::EventFilter::empty();
    event_filter.focused = true;
    event_filter.unfocused = true;

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
