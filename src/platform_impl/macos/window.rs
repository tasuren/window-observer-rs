use accessibility::{AXAttribute, AXUIElement};
use objc2_core_foundation::{CGPoint, CGSize};

use super::{
    ax_function::{ax_ui_element_copy_attribute_value, ax_value_get_value},
    OSError,
};
use crate::window::{Position, Size};

/// Represents a macOS window and provides methods to interact with it.
pub struct MacOSWindow(AXUIElement);
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
    /// Creates a new `MacOSWindow` instance from an `AXUIElement`.
    pub fn new(element: AXUIElement) -> Self {
        Self(element)
    }

    /// Retrieves the underlying `AXUIElement`.
    pub fn ax_ui_element(&self) -> &AXUIElement {
        &self.0
    }

    /// Retrieves a specific attribute of the window via `AXUIElement`.
    fn get<T>(
        &self,
        attribute: &str,
        r#type: accessibility_sys::AXValueType,
    ) -> Result<T, OSError> {
        let ax_value =
            ax_ui_element_copy_attribute_value(&self.0, attribute).map_err(OSError::Ax)?;
        Ok(unsafe { ax_value_get_value::<T>(ax_value as _, r#type).unwrap() })
    }

    pub fn get_title(&self) -> Result<String, OSError> {
        Ok(self.0.attribute(&AXAttribute::title())?.to_string())
    }

    /// Retrieves the size of the window.
    pub fn get_size(&self) -> Result<Size, OSError> {
        self.get::<CGSize>(
            accessibility_sys::kAXSizeAttribute,
            accessibility_sys::kAXValueTypeCGSize,
        )
        .map(|v| v.into())
    }

    /// Retrieves the position of the window.
    pub fn get_position(&self) -> Result<Position, OSError> {
        self.get::<CGPoint>(
            accessibility_sys::kAXPositionAttribute,
            accessibility_sys::kAXValueTypeCGPoint,
        )
        .map(|v| v.into())
    }

    /// Checks if the window is currently active.
    pub fn is_active(&self) -> Result<bool, OSError> {
        Ok(self.0.attribute(&AXAttribute::main())?.into())
    }
}
