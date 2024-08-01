use crate::Error;

#[derive(Default, Debug)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Default, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Window(pub(crate) crate::platform_impl::Window);

impl Window {
    pub fn get_size(&self) -> Result<Size, Error> {
        self.0.get_size()
    }

    pub fn get_position(&self) -> Result<Position, Error> {
        self.0.get_position()
    }

    pub fn is_active(&self) -> Result<bool, Error> {
        self.0.is_active()
    }
}
