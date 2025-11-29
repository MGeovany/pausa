// Native macOS menu bar text support using NSStatusItem
// This is a workaround for Tauri's lack of native text support in menu bar
// See: https://github.com/tauri-apps/tao/issues/65

#[cfg(target_os = "macos")]
use objc::{msg_send, runtime::Object, runtime::Class};
#[cfg(target_os = "macos")]
use objc::*; // Import all macros including class!, sel!
#[cfg(target_os = "macos")]
use std::ffi::CString;

#[cfg(target_os = "macos")]
static mut STATUS_ITEM: Option<*mut Object> = None;

// Link to AppKit framework
#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

/// Initialize native macOS menu bar status item with text support
#[cfg(target_os = "macos")]
pub fn init_menu_bar_text() -> Result<(), String> {
    unsafe {
        // Wrap in a catch-all to prevent foreign exceptions from crashing Rust
        // Note: This is a workaround - ideally we'd use @try/@catch but Rust can't handle that
        // Instead, we'll be extra careful with null checks
        
        // Get NSStatusBar class
        let ns_status_bar_class: *const Class = class!(NSStatusBar);
        if ns_status_bar_class.is_null() {
            return Err("Failed to get NSStatusBar class".to_string());
        }
        
        // Get system status bar
        let status_bar: *mut Object = msg_send![ns_status_bar_class, systemStatusBar];
        
        if status_bar.is_null() {
            return Err("Failed to get system status bar".to_string());
        }
        
        // NSVariableStatusItemLength = -1.0
        let variable_length: f64 = -1.0;
        let status_item: *mut Object = msg_send![status_bar, statusItemWithLength: variable_length];
        
        if status_item.is_null() {
            return Err("Failed to create status item".to_string());
        }
        
        STATUS_ITEM = Some(status_item);
        
        Ok(())
    }
}

/// Update the menu bar text (macOS only)
#[cfg(target_os = "macos")]
pub fn update_menu_bar_text(text: &str) -> Result<(), String> {
    unsafe {
        if let Some(status_item) = STATUS_ITEM {
            // Create NSString from Rust string
            let ns_string_class: *const Class = class!(NSString);
            let c_string = CString::new(text).map_err(|e| format!("Failed to create CString: {}", e))?;
            let ns_string: *mut Object = msg_send![ns_string_class, stringWithUTF8String: c_string.as_ptr()];
            
            if ns_string.is_null() {
                return Err("Failed to create NSString".to_string());
            }
            
            // Set the title (text) on the status item
            let _: () = msg_send![status_item, setTitle: ns_string];
            
            Ok(())
        } else {
            Err("Status item not initialized. Call init_menu_bar_text() first.".to_string())
        }
    }
}

/// Cleanup native menu bar status item
#[cfg(target_os = "macos")]
pub fn cleanup_menu_bar_text() {
    unsafe {
        if let Some(status_item) = STATUS_ITEM {
            let ns_status_bar_class: *const Class = class!(NSStatusBar);
            let status_bar: *mut Object = msg_send![ns_status_bar_class, systemStatusBar];
            let _: () = msg_send![status_bar, removeStatusItem: status_item];
            STATUS_ITEM = None;
        }
    }
}

// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn init_menu_bar_text() -> Result<(), String> {
    Ok(()) // No-op on non-macOS
}

#[cfg(not(target_os = "macos"))]
pub fn update_menu_bar_text(_text: &str) -> Result<(), String> {
    Ok(()) // No-op on non-macOS
}

#[cfg(not(target_os = "macos"))]
pub fn cleanup_menu_bar_text() {
    // No-op on non-macOS
}

