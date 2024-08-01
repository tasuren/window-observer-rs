use accessibility_sys::AXUIElementRef;
use objc2_foundation::{CGPoint, CGSize};

use crate::{
    window::{Position, Size},
    Error,
};

use super::helper::{ax_ui_element_copy_attribute_value, ax_value_get_value};

pub struct Window(pub AXUIElementRef);

impl Into<Size> for CGSize {
    fn into(self) -> Size {
        Size {
            width: self.width as _,
            height: self.height as _,
        }
    }
}

impl Into<Position> for CGPoint {
    fn into(self) -> Position {
        Position {
            x: self.x as _,
            y: self.y as _,
        }
    }
}

impl Window {
    unsafe fn get<T>(
        &self,
        attribute: &str,
        r#type: accessibility_sys::AXValueType,
    ) -> Result<T, Error> {
        unsafe {
            let size = ax_ui_element_copy_attribute_value(self.0 as _, attribute)?;
            Ok(ax_value_get_value::<T>(size as _, r#type).unwrap())
        }
    }

    pub fn get_size(&self) -> Result<Size, Error> {
        unsafe {
            self.get::<CGSize>(
                accessibility_sys::kAXSizeAttribute,
                accessibility_sys::kAXValueTypeCGSize,
            )
            .map(|v| v.into())
        }
    }

    pub fn get_position(&self) -> Result<Position, Error> {
        unsafe {
            self.get::<CGPoint>(
                accessibility_sys::kAXPositionAttribute,
                accessibility_sys::kAXValueTypeCGPoint,
            )
            .map(|v| v.into())
        }
    }

    pub fn is_main(&self) -> Result<bool, Error> {
        unsafe {
            Ok(
                ax_ui_element_copy_attribute_value(
                    self.0 as _,
                    accessibility_sys::kAXMainAttribute,
                )? as u8
                    != 0,
            )
        }
    }
}
