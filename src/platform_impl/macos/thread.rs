use std::{ptr::NonNull, rc::Rc, sync::atomic::AtomicPtr};

use accessibility::{AXAttribute, AXUIElement};
use objc2_core_foundation::{CFRetained, CFRunLoop};

use crate::{Event, EventFilter, EventTx};

use super::{ax_observer::AXObserver, event_helper::EventMacOSExt, window::MacOSWindow};

/// Run `CFRunLoop`. It will set off the observer.
fn run_loop(observer: AXObserver, controller_tx: EventLoopControllerTx) {
    let run_loop = CFRunLoop::current().unwrap();

    let source = observer.get_run_loop_source();
    let source = Some(source.as_ref());
    let mode = unsafe { objc2_core_foundation::kCFRunLoopDefaultMode };
    run_loop.add_source(source, mode);

    controller_tx
        .send(EventLoopController::new(CFRetained::clone(&run_loop)))
        .unwrap();
    CFRunLoop::run();
}

fn observer_callback(ax_ui_element: Rc<AXUIElement>, event_tx: EventTx, notification: String) {
    let Some(event) = Event::from_ax_notification(&notification) else {
        return;
    };

    let window_element = ax_ui_element
        .attribute(&AXAttribute::focused_window())
        .unwrap();
    let window = MacOSWindow::new(window_element);

    event_tx.send((crate::Window(window), event)).unwrap();
}

fn observe(
    pid: i32,
    event_tx: EventTx,
    event_filter: EventFilter,
    stopper_rx: EventLoopControllerTx,
) {
    // Instantiate `AXObserver`.
    let ax_ui_element = Rc::new(AXUIElement::application(pid));
    let callback = {
        let ax_ui_element = Rc::clone(&ax_ui_element);

        move |notification: String| {
            observer_callback(Rc::clone(&ax_ui_element), event_tx.clone(), notification);
        }
    };

    let observer = AXObserver::create(pid, Box::new(callback));

    // Add the event filter to the observer.
    for event in event_filter {
        observer
            .add_notification(&ax_ui_element, event.ax_notification())
            .expect("Failed to add notification to `AXObserver`");
    }

    // Start the observer.
    run_loop(observer, stopper_rx);
}

pub fn make_observe_thread(
    pid: i32,
    event_tx: EventTx,
    event_filter: EventFilter,
    controller_tx: EventLoopControllerTx,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        observe(pid, event_tx, event_filter, controller_tx);
    })
}

#[derive(Debug)]
pub struct EventLoopController(AtomicPtr<CFRunLoop>);

impl EventLoopController {
    pub fn new(run_loop: CFRetained<CFRunLoop>) -> Self {
        let ptr = CFRetained::into_raw(run_loop);
        Self(AtomicPtr::new(ptr.as_ptr()))
    }

    pub fn stop(self) {
        let run_loop = self.0.load(std::sync::atomic::Ordering::SeqCst);

        CFRunLoop::stop(unsafe { &*(run_loop as *const CFRunLoop) });
    }
}

impl Drop for EventLoopController {
    fn drop(&mut self) {
        let ptr = NonNull::new(self.0.load(std::sync::atomic::Ordering::SeqCst)).unwrap();
        // SAFETY: We are the owner of the `CFRunLoop` and we are dropping it to call the destructor.
        let _ = unsafe { CFRetained::from_raw(ptr) };
    }
}

pub type EventLoopControllerTx = tokio::sync::oneshot::Sender<EventLoopController>;
