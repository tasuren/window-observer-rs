# window-observer-rs

This crate provides an observer that receives events such as window movement and resizing.
It is designed to receive window events on Windows and macOS for cross-platform applications.

[![Crates.io Version](https://img.shields.io/crates/v/window-observer)](https://crates.io/crates/window-observer)
[![docs.rs](https://img.shields.io/docsrs/window-observer)](https://docs.rs/window-observer/latest/window_observer/)

## Example

```rust
use window_observer::{EventFilter, WindowObserver};

#[tokio::main]
async fn main() {
    let pid = std::env::var("PID")
        .map(|v| v.parse().unwrap())
        .expect("Please give me the env `PID` of application that has window.");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_filter = EventFilter::all();

    let _window_observer = WindowObserver::start(pid, event_tx, event_filter)
        .await
        .unwrap();

    while let Some(event) = event_rx.recv().await {
        match event {
            Ok(event) => println!("new event: {event:#?}"),
            Err(e) => eprintln!("Error occurred during handling event: {e:#?}"),
        }
    }
}
```

## Platform supports

- [x] macOS
- [x] Windows
- [ ] Linux?

I have no plans to make this at this time due to my inexperienced knowledge about Linux.
But I'd be happy to receive pull requests.

## Acknowledgements

- Windows event handler: [wineventhook-rs](https://github.com/OpenByteDev/wineventhook-rs/)

## License

This project is licensed under the [MIT License](https://github.com/tasuren/window-observer-rs/blob/main/LICENSE).
