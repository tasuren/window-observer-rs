use window_observer::{self, Event};

fn print_event(window: window_observer::Window, event: Event) {
    match event {
        Event::Activated => println!("Is window main: {}", window.is_active().unwrap()),
        Event::Moved => println!("Window current position: {:?}", window.get_position()),
        Event::Resized => println!("Window current size: {:?}", window.get_size()),
    };
}

#[tokio::main]
async fn main() {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut window_observer = window_observer::WindowObserver::new(
        std::env::var("PID")
            .map(|v| v.parse().unwrap())
            .expect("Please give me the env `PID` of application that has window."),
        event_tx,
    )
    .unwrap();

    window_observer.add_target_event(Event::Activated).unwrap();
    window_observer.add_target_event(Event::Moved).unwrap();
    window_observer.add_target_event(Event::Resized).unwrap();

    tokio::spawn(async move {
        while let Some((window, event)) = event_rx.recv().await {
            print_event(window, event);
        }
    });

    window_observer.run().unwrap();
}
