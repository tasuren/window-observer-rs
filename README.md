# window-observer-rs
This crate provides an observer that receives events such as window movement and resizing.
It is designed to receive window events on Windows and macOS for cross-platform applications.

**⚠️ It is not production ready yet. Expect extremely disruptive changes.**

## Example
```rust
use window_observer::{self, Event, WindowObserver};

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

    while let Some((_window, event)) = event_rx.recv().await {
        println!("{event:?}");
    }
}
```

## ToDo
First release todos:
- [x] macOS
  - [x] Resize detection
  - [x] Move detection
  - [x] Foreground detection
  - [x] Resize detection with information such as width, height
  - [x] Move detection with information such as x, y.
  - [x] Get the window title
- [x] Windows
  - [x] Resize detection
  - [x] Move detectio
  - [x] Foreground detectionn
  - [x] Resize detection with information such as width, height
  - [x] Move detection with information such as x, y.
  - [x] Get the window title
- [ ] Linux?
  I have no plans to make this at this time due to my inexperienced knowledge about Linux.  
  But I'd be happy to receive pull requests.

## Acknowledgements
- Windows event handler: [wineventhook-rs](https://github.com/OpenByteDev/wineventhook-rs/)

## License
This project is licensed under the [MIT License](./LICENSE).
