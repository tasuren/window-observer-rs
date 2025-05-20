use window_observer::{self, Event, WindowObserver};

fn print_event(window: window_observer::Window, event: Event) {
    match event {
        Event::Activated => println!("Is window main: {}", window.is_active().unwrap()),
        Event::Moved => println!("Window current position: {:?}", window.get_position()),
        Event::Resized => println!("Window current size: {:?}", window.get_size()),
    };
}

#[tokio::main]
async fn main() {
    let pid = std::env::var("PID")
        .map(|v| v.parse().unwrap())
        .expect("Please give me the env `PID` of application that has window.");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_filter = window_observer::smallvec![Event::Activated, Event::Moved, Event::Resized];

    let _window_observer = WindowObserver::start(pid, event_tx, event_filter)
        .await
        .unwrap();

    while let Some((window, event)) = event_rx.recv().await {
        print_event(window, event);
    }
}
