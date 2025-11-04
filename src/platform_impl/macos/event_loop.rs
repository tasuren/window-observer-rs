use std::thread::{self, JoinHandle};

use objc2_core_foundation::{CFRetained, CFRunLoop, CFRunLoopSource, kCFRunLoopDefaultMode};
use tokio::sync::OnceCell;

use super::binding_ax_observer::AXObserver;

/// The wrapper of [`CFRunLoop`] for [`AXObserver`].
pub struct EventLoop {
    run_loop: CFRetained<CFRunLoop>,
    handle: JoinHandle<()>,
}
unsafe impl Send for EventLoop {}
unsafe impl Sync for EventLoop {}

impl EventLoop {
    pub async fn new() -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();

        #[derive(Debug)]
        struct SendRunLoop(CFRetained<CFRunLoop>);
        unsafe impl Send for SendRunLoop {}
        unsafe impl Sync for SendRunLoop {}

        let handle = std::thread::spawn(move || {
            let run_loop = CFRunLoop::current().unwrap();
            tx.send(SendRunLoop(CFRetained::clone(&run_loop))).unwrap();

            thread::park();

            CFRunLoop::run();
        });
        let run_loop = rx.await.unwrap().0;

        Self { run_loop, handle }
    }

    pub fn register(&self, source: CFRetained<CFRunLoopSource>) {
        let source = Some(source.as_ref());
        let mode = unsafe { kCFRunLoopDefaultMode };

        self.run_loop.add_source(source, mode);

        self.handle.thread().unpark();
    }

    pub fn unregister(&self, source: CFRetained<CFRunLoopSource>) {
        let source = Some(source.as_ref());
        let mode = unsafe { kCFRunLoopDefaultMode };

        self.run_loop.remove_source(source, mode);
    }
}

static EVENT_LOOP: OnceCell<EventLoop> = OnceCell::const_new();

/// Returns a reference to the global event loop for the observer.
/// This function initializes the event loop and starts it if it hasn't been initialized yet.
pub async fn event_loop() -> &'static EventLoop {
    EVENT_LOOP
        .get_or_init(|| async { EventLoop::new().await })
        .await
}

pub fn get_event_loop<'a>() -> Option<&'a EventLoop> {
    EVENT_LOOP.get()
}

/// A wrapper for [`CFRunLoopSource`] and [`AXObserver`].
/// This struct keeps [`AXObserver`] alive for preventing it from destroyed.
/// And it can be sended to other threads safely because [`AXObserver`] can't be touched.
/// Also, it provides a method to get the `CFRunLoopSource` of [`AXObserver`].
pub struct ObserverSource {
    source: CFRetained<CFRunLoopSource>,
    _observer: AXObserver, // This field is never touched.
}
unsafe impl Send for ObserverSource {}
unsafe impl Sync for ObserverSource {}

impl ObserverSource {
    pub fn new(observer: AXObserver) -> Self {
        let source = observer.get_run_loop_source();
        Self {
            source,
            _observer: observer,
        }
    }

    pub fn get(&self) -> CFRetained<CFRunLoopSource> {
        CFRetained::clone(&self.source)
    }
}
