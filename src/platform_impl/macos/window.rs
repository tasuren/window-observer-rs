use accessibility_sys::AXUIElementRef;
use objc2_foundation::{CGPoint, CGSize};

use crate::{
    window::{Position, Size},
    Error,
};

use super::helper::{ax_ui_element_copy_attribute_value, ax_value_get_value};

pub struct Window(pub AXUIElementRef);

impl From<CGSize> for Size {
    fn from(size: CGSize) -> Self {
        Size {
            width: size.width as _,
            height: size.height as _,
        }
    }
}

impl From<CGPoint> for Position {
    fn from(point: CGPoint) -> Self {
        Position {
            x: point.x as _,
            y: point.y as _,
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

    pub fn is_active(&self) -> Result<bool, Error> {
        unsafe {
            Ok(
                ax_ui_element_copy_attribute_value(
                    self.0 as _,
                    accessibility_sys::kAXFocusedAttribute,
                )? as u8
                    != 0,
            )
        }
    }
}
