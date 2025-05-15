//! Bindings for macOS

use std::rc::Rc;

use accessibility::{AXAttribute, AXUIElement};

use event::EventMacOSExt;
use objc2_core_foundation::{CFRetained, CFRunLoop};
use window::MacOSWindow;

use crate::{Error, Event, EventTx};

pub mod ax_error;
pub mod ax_observer;
pub mod error_helper;
pub mod event;
pub mod function;
pub mod window;

use self::{ax_observer::AXObserver, function::ax_is_process_trusted};

pub struct MacOSWindowObserver {
    _pid: i32,
    running: bool,
    runloop: CFRetained<CFRunLoop>,
    element: Rc<AXUIElement>,
    observer: AXObserver,
}

impl MacOSWindowObserver {
    pub fn new(pid: i32, event_tx: EventTx) -> Result<Self, crate::Error> {
        if !ax_is_process_trusted() {
            return Err(crate::Error::PermissinoDenied);
        };

        let element = Rc::new(AXUIElement::application(pid));
        let callback = {
            let element = Rc::clone(&element);

            move |notification: String| {
                let Some(event) = Event::from_ax_notification(&notification) else {
                    return;
                };

                let window_element = element.attribute(&AXAttribute::focused_window()).unwrap();
                let window = MacOSWindow(window_element);

                event_tx.send((crate::Window(window), event)).unwrap();
            }
        };

        Ok(Self {
            _pid: pid,
            running: false,
            runloop: CFRunLoop::current().unwrap(),
            element,
            observer: AXObserver::create(pid, Box::new(callback)),
        })
    }

    pub fn add_target_event(&self, event: Event) -> Result<(), Error> {
        self.observer
            .add_notification(&self.element, event.ax_notification())?;

        Ok(())
    }

    pub fn remove_target_event(&self, event: Event) -> Result<(), Error> {
        self.observer
            .remove_notification(&self.element, event.ax_notification())?;

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::AlreadyStarted);
        }
        self.running = true;

        let source = self.observer.get_run_loop_source();
        let source = Some(source.as_ref());
        let mode = unsafe { objc2_core_foundation::kCFRunLoopDefaultMode };
        self.runloop.add_source(source, mode);

        Ok(())
    }

    pub fn join(&self) {
        CFRunLoop::run();
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if !self.running {
            return Err(Error::AlreadyStopped);
        }

        let source = self.observer.get_run_loop_source();
        let source = Some(source.as_ref());
        let mode = unsafe { objc2_core_foundation::kCFRunLoopDefaultMode };
        self.runloop.remove_source(source, mode);

        self.running = false;
        Ok(())
    }
}
