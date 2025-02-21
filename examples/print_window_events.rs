use window_observer::{self, Event};

fn callback(event: Event, window: window_observer::Window) {
    match event {
        Event::Activated => println!("Is window main: {}", window.is_active().unwrap()),
        Event::Moved => println!("Window current position: {:?}", window.get_position()),
        Event::Resized => println!("Window current size: {:?}", window.get_size()),
    };
}

fn main() {
    let mut window_observer = window_observer::WindowObserver::new(
        std::env::var("PID")
            .map(|v| v.parse().unwrap())
            .expect("Please give me the env `PID` of application that has window."),
        Box::new(callback),
    )
    .unwrap();

    window_observer.add_target_event(Event::Activated).unwrap();
    window_observer.add_target_event(Event::Moved).unwrap();
    window_observer.add_target_event(Event::Resized).unwrap();

    window_observer.run().unwrap();
}
