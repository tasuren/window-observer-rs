use window_observer;

fn main() {
    let mut window_observer = window_observer::WindowObserver::new(
        std::env::var("PID")
            .map(|v| v.parse().unwrap())
            .expect("Please give me the env `PID` of application that has window."),
        Box::new(|| println!("Main window changed!")),
    )
    .unwrap();

    window_observer.run().unwrap();
}
