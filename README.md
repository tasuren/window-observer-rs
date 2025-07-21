# window-observer-rs
This crate provides an observer that receives events such as window movement and resizing.
It is designed to receive window events on Windows and macOS for cross-platform applications.

**⚠️ It is not production ready yet. Expect extremely disruptive changes.**

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

## ToDo
First release todos:
- [x] macOS
  - [x] Resize detection
  - [x] Move detection
  - [x] Foreground detection
  - [x] Backgrounded detection
  - [x] Hidden detection
  - [x] Showed detection
  - [x] Focus switching detection
  - [x] Get the window title
- [x] Windows
  - [x] Resize detection
  - [x] Move detectio
  - [x] Foreground detectionn
  - [x] Backgrounded detection
  - [x] Hidden detection
  - [x] Showed detection
  - [x] Focus switching detection
  - [x] Get the window title
- [ ] Linux?
  I have no plans to make this at this time due to my inexperienced knowledge about Linux.  
  But I'd be happy to receive pull requests.

## Acknowledgements
- Windows event handler: [wineventhook-rs](https://github.com/OpenByteDev/wineventhook-rs/)

## License
This project is licensed under the [MIT License](./LICENSE).
