use window_observer::{self, Event};

fn main() {
    let mut window_observer = window_observer::WindowObserver::new(
        std::env::var("PID")
            .map(|v| v.parse().unwrap())
            .expect("Please give me the env `PID` of application that has window."),
        Box::new(|event| match event {
            Event::Activated => println!("Window is opened now."),
            _ => (),
        }),
    )
    .unwrap();
    println!("aiueo");
    window_observer.add_target_event(Event::Activated);
    window_observer.run().unwrap();
}
