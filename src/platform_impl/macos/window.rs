use accessibility::{AXAttribute, AXUIElement};
use objc2_foundation::{CGPoint, CGSize};

use crate::{
    window::{Position, Size},
    Error,
};

use super::function::{ax_ui_element_copy_attribute_value, ax_value_get_value};

pub struct MacOSWindow(pub AXUIElement);
unsafe impl Send for MacOSWindow {}
unsafe impl Sync for MacOSWindow {}

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

impl MacOSWindow {
    fn get<T>(&self, attribute: &str, r#type: accessibility_sys::AXValueType) -> Result<T, Error> {
        let ax_value = ax_ui_element_copy_attribute_value(&self.0, attribute)?;
        Ok(unsafe { ax_value_get_value::<T>(ax_value as _, r#type).unwrap() })
    }

    pub fn get_size(&self) -> Result<Size, Error> {
        self.get::<CGSize>(
            accessibility_sys::kAXSizeAttribute,
            accessibility_sys::kAXValueTypeCGSize,
        )
        .map(|v| v.into())
    }

    pub fn get_position(&self) -> Result<Position, Error> {
        self.get::<CGPoint>(
            accessibility_sys::kAXPositionAttribute,
            accessibility_sys::kAXValueTypeCGPoint,
        )
        .map(|v| v.into())
    }

    pub fn is_active(&self) -> Result<bool, Error> {
        Ok(self.0.attribute(&AXAttribute::focused())?.into())
    }
}
