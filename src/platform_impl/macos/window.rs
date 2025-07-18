use accessibility::{AXUIElement, AXUIElementAttributes};
use accessibility_sys::kAXWindowRole;
use objc2_core_foundation::{CGPoint, CGSize};

use super::{
    ax_function::{ax_ui_element_copy_attribute_value, ax_value_get_value},
    PlatformError,
};
use crate::{
    platform_impl::macos::error::UnexpectedUIElementError,
    window::{Position, Size},
};

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

/// Represents a macOS window and provides methods to interact with it.
/// This is the wrapper of [`AXUIElement`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformWindow(AXUIElement);

unsafe impl Send for PlatformWindow {}
unsafe impl Sync for PlatformWindow {}

impl PlatformWindow {
    /// Creates a new [`PlatformWindow`] instance from an [`AXUIElement`].
    pub fn new(element: AXUIElement) -> Result<Self, UnexpectedUIElementError> {
        let role = match element.role() {
            Ok(role) => role.to_string(),
            Err(error) => {
                return Err(UnexpectedUIElementError {
                    element,
                    expected: kAXWindowRole,
                    received: Err(error),
                })
            }
        };

        match role.as_str() {
            accessibility_sys::kAXWindowRole => Ok(Self(element)),
            _ => Err(UnexpectedUIElementError {
                element,
                expected: kAXWindowRole,
                received: Ok(role),
            }),
        }
    }

    /// Creates a new [`PlatformWindow`] instance without checking the role.
    ///
    /// # Safety
    /// You need to ensure that the provided `AXUIElement` is indeed a window.
    /// This means that the role of the element must be [`kAXWindowRole`].
    pub unsafe fn new_unchecked(element: AXUIElement) -> Self {
        Self(element)
    }

    /// Retrieves the underlying [`AXUIElement`].
    pub fn ax_ui_element(&self) -> &AXUIElement {
        &self.0
    }

    /// Retrieves a specific attribute of the window via [`AXUIElement`].
    fn get<T>(
        &self,
        attribute: &str,
        r#type: accessibility_sys::AXValueType,
    ) -> Result<T, PlatformError> {
        let ax_value =
            ax_ui_element_copy_attribute_value(&self.0, attribute).map_err(PlatformError::Ax)?;
        Ok(unsafe { ax_value_get_value::<T>(ax_value as _, r#type).unwrap() })
    }

    /// Retrieves the title of the window.
    pub fn title(&self) -> Result<String, PlatformError> {
        Ok(self.0.title()?.to_string())
    }

    /// Retrieves the size of the window.
    pub fn size(&self) -> Result<Size, PlatformError> {
        self.get::<CGSize>(
            accessibility_sys::kAXSizeAttribute,
            accessibility_sys::kAXValueTypeCGSize,
        )
        .map(|v| v.into())
    }

    /// Retrieves the position of the window.
    pub fn position(&self) -> Result<Position, PlatformError> {
        self.get::<CGPoint>(
            accessibility_sys::kAXPositionAttribute,
            accessibility_sys::kAXValueTypeCGPoint,
        )
        .map(|v| v.into())
    }

    /// Checks if the window is currently active.
    pub fn is_focused(&self) -> Result<bool, PlatformError> {
        Ok(self.0.focused()?.into())
    }

    /// Retrieves the id of the window. The value is [`CGWindowID`][window_id] on macOS.
    ///
    /// # Warning
    /// This function will call private API `_AXUIElementGetWindow` of macOS.
    /// This is because the [`AXUIElement`][element] does not provide a public method to get the window id.
    ///
    /// [window_id]: https://developer.apple.com/documentation/coregraphics/cgwindowid?language=objc
    /// [element]: https://developer.apple.com/documentation/applicationservices/axuielement_h?language=objc
    #[cfg(feature = "macos-private-api")]
    pub fn id(&self) -> Result<u32, PlatformError> {
        super::ax_function::ax_ui_element_get_window_id(&self.0)
    }
}
