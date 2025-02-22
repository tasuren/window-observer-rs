use std::{cell::RefCell, rc::Rc};

mod sys {
    use window_observer::{Event, Window, WindowObserver};

    #[derive(Default)]
    pub struct System {
        observer: Option<WindowObserver>,
    }

    impl System {
        pub fn start(&mut self, pid: i32, callback: impl Fn(String) + 'static) -> Result<(), ()> {
            if self.observer.is_some() {
                return Err(());
            }

            let callback = move |event, window: Window| {
                let event = match event {
                    Event::Activated => "Activated".to_string(),
                    Event::Moved => format!("{:?}", window.get_position().unwrap()),
                    Event::Resized => format!("{:?}", window.get_size().unwrap()),
                };

                callback(event);
            };

            let mut observer = WindowObserver::new(pid, Box::new(callback)).unwrap();
            observer.add_target_event(Event::Activated).unwrap();
            observer.add_target_event(Event::Moved).unwrap();
            observer.add_target_event(Event::Resized).unwrap();
            observer.start().unwrap();

            self.observer = Some(observer);

            Ok(())
        }

        pub fn stop(&mut self) -> Result<(), ()> {
            match self.observer.take() {
                Some(mut observer) => observer.stop().unwrap(),
                None => return Err(()),
            }

            Ok(())
        }
    }
}

type SharedSystem = Rc<RefCell<sys::System>>;

mod presenter {
    use std::{cell::RefCell, rc::Rc};

    use libui::{prelude::Window, prelude::*, UI};

    use crate::SharedSystem;

    fn setup_layout(system: SharedSystem, _ui: &UI, window: &mut Window) {
        libui::layout! { _ui,
            let layout = VerticalBox(padded: true) {
                Compact: let _config = HorizontalBox(padded: true) {
                    Stretchy: let pid_box = Spinbox(i32::MIN, i32::MAX)
                    Compact: let start_button = Button("Start")
                    Compact: let stop_button = Button("stop")
                }
                Compact: let result_entry = MultilineEntry()
            }
        };

        // Prepare ui initial state.
        stop_button.disable();
        result_entry.set_readonly(true);

        // Set button handler.
        start_button.on_clicked({
            let mut stop_button = stop_button.clone();
            let system = Rc::clone(&system);

            move |this| {
                this.disable();

                let callback = {
                    let result_entry = RefCell::new(result_entry.clone());

                    move |event: String| {
                        result_entry.borrow_mut().append(&format!("{event}\n"));
                    }
                };
                system
                    .borrow_mut()
                    .start(pid_box.value(), callback)
                    .unwrap();

                stop_button.enable();
            }
        });

        stop_button.on_clicked({
            let mut start_button = start_button.clone();

            move |this| {
                this.disable();

                system.borrow_mut().stop().unwrap();

                start_button.enable();
            }
        });

        window.set_child(layout);
    }

    fn setup_window(app: &App) -> Window {
        let ui = &app.ui;

        Window::new(ui, "Window Observer", 500, 400, WindowType::NoMenubar)
    }

    pub struct App {
        ui: UI,
        window: Option<Window>,
    }

    impl App {
        pub fn new(system: SharedSystem) -> Self {
            let mut app = Self {
                ui: UI::init().unwrap(),
                window: None,
            };

            let mut window = setup_window(&app);
            setup_layout(system, &app.ui, &mut window);
            window.show();

            app.window = Some(window);

            app
        }

        pub fn run(&mut self) {
            self.ui.main();
        }
    }
}

fn main() {
    let system = Rc::new(RefCell::new(sys::System::default()));
    let mut app = presenter::App::new(system);
    app.run();
}
