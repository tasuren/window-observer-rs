//! Bindings for macOS

use std::rc::Rc;

use accessibility::{AXAttribute, AXUIElement};

use event_helper::EventMacOSExt;
use objc2_core_foundation::{CFRetained, CFRunLoop};
use window::MacOSWindow;

use crate::{Error, Event, EventTx};

pub mod ax_observer;
pub mod error_helper;
pub mod event_helper;
pub mod function;
pub mod window;

use self::{ax_observer::AXObserver, function::ax_is_process_trusted};

pub use accessibility_sys::AXError;

/// Observes macOS window events and provides an interface to manage them.
/// This is wrapper of `AXObserver`.
pub struct MacOSWindowObserver {
    _pid: i32,
    running: bool,
    runloop: CFRetained<CFRunLoop>,
    ax_ui_element: Rc<AXUIElement>,
    ax_observer: AXObserver,
}

impl MacOSWindowObserver {
    /// Creates a new `MacOSWindowObserver` for a given process ID and event channel.
    pub fn new(pid: i32, event_tx: EventTx) -> Result<Self, crate::Error> {
        if !ax_is_process_trusted() {
            return Err(crate::Error::PermissinoDenied);
        };

        let ax_ui_element = Rc::new(AXUIElement::application(pid));
        let callback = {
            let ax_ui_element = Rc::clone(&ax_ui_element);

            move |notification: String| {
                let Some(event) = Event::from_ax_notification(&notification) else {
                    return;
                };

                let window_element = ax_ui_element
                    .attribute(&AXAttribute::focused_window())
                    .unwrap();
                let window = MacOSWindow::new(window_element);

                event_tx.send((crate::Window(window), event)).unwrap();
            }
        };

        Ok(Self {
            _pid: pid,
            running: false,
            runloop: CFRunLoop::current().unwrap(),
            ax_ui_element,
            ax_observer: AXObserver::create(pid, Box::new(callback)),
        })
    }

    /// Retrieves the PID of the process being observed.
    pub fn pid(&self) -> i32 {
        self._pid
    }

    /// Retrieves the `AXUIElement` of the application being observed.
    pub fn ax_ui_element(&self) -> &AXUIElement {
        &self.ax_ui_element
    }

    /// Retrieves the `AXObserver` being used by this.
    pub fn ax_observer(&self) -> &AXObserver {
        &self.ax_observer
    }

    /// Adds a event to be observed.
    pub fn add_target_event(&self, event: Event) -> Result<(), Error> {
        self.ax_observer
            .add_notification(&self.ax_ui_element, event.ax_notification())?;

        Ok(())
    }

    /// Removes a event from being observed.
    pub fn remove_target_event(&self, event: Event) -> Result<(), Error> {
        self.ax_observer
            .remove_notification(&self.ax_ui_element, event.ax_notification())?;

        Ok(())
    }

    /// Starts the event observer.
    pub fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::AlreadyStarted);
        }
        self.running = true;

        let source = self.ax_observer.get_run_loop_source();
        let source = Some(source.as_ref());
        let mode = unsafe { objc2_core_foundation::kCFRunLoopDefaultMode };
        self.runloop.add_source(source, mode);

        Ok(())
    }

    /// Runs the event loop (`CFRunLoop`) for the observer. It will block the current thread.
    ///
    /// If a `CFRunLoop` is already running, this call is not necessary.
    /// For example, when shared with a GUI application.
    pub fn join(&self) {
        CFRunLoop::run();
    }

    /// Stops the observer.
    pub fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::AlreadyStopped);
        }

        let source = self.ax_observer.get_run_loop_source();
        let source = Some(source.as_ref());
        let mode = unsafe { objc2_core_foundation::kCFRunLoopDefaultMode };
        self.runloop.remove_source(source, mode);

        self.running = false;
        Ok(())
    }
}
