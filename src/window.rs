use crate::Error;

#[derive(Default, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Default, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub struct Window(pub(crate) crate::platform_impl::Window);

impl Window {
    pub fn get_size(&self) -> Result<Size, Error> {
        self.0.get_size()
    }

    pub fn get_position(&self) -> Result<Position, Error> {
        self.0.get_position()
    }

    pub fn is_main(&self) -> Result<bool, Error> {
        self.0.is_main()
    }
}
