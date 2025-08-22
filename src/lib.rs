//! # Luuma Cursor Helper
//! 
//! A Rust library for detecting cursor position, type, and mouse clicks with real-time logging.
//! 
//! ## Features
//! 
//! - Real-time cursor position tracking
//! - Cursor type detection (arrow, hand, I-beam, etc.)
//! - Mouse click detection (left and right clicks)
//! - Timestamped logging of all cursor activities
//! - Windows API integration for accurate cursor type detection
//! 
//! ## Example
//! 
//! ```rust
//! use luuma_cursor_helper::{CursorDetector, CursorState};
//! 
//! fn main() {
//!     let mut detector = CursorDetector::new();
//!     detector.start_monitoring();
//! }
//! ```

use rdev::{listen, EventType, Button};
use device_query::{DeviceQuery, DeviceState};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorInfo, CURSORINFO, CURSOR_SHOWING, HCURSOR, LoadCursorW, IDC_ARROW, IDC_IBEAM, IDC_WAIT, IDC_CROSS, IDC_UPARROW, IDC_SIZE, IDC_SIZENWSE, IDC_SIZENESW, IDC_SIZEWE, IDC_SIZENS, IDC_SIZEALL, IDC_NO, IDC_HAND, IDC_APPSTARTING, IDC_HELP, IDC_PIN, IDC_PERSON};
use windows::Win32::Foundation::POINT;

/// Represents the current state of the cursor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorState {
    /// Current cursor position (x, y)
    pub position: (f64, f64),
    /// Current cursor type (arrow, hand, ibeam, etc.)
    pub cursor_type: String,
    /// Whether left mouse button is pressed
    pub left_click: bool,
    /// Whether right mouse button is pressed
    pub right_click: bool,
    /// Timestamp when this state was captured
    pub timestamp: String,
}

impl CursorState {
    /// Create a new cursor state with default values
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            cursor_type: "default".to_string(),
            left_click: false,
            right_click: false,
            timestamp: CursorDetector::get_timestamp(),
        }
    }

    /// Convert cursor state to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    
    /// Create cursor state from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert cursor state to pretty-formatted JSON string
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Different types of cursor events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CursorEvent {
    /// Cursor moved to a new position
    Move { position: (f64, f64), cursor_type: String, timestamp: String },
    /// Mouse button was clicked
    Click { button: String, position: (f64, f64), timestamp: String },
    /// Mouse button was released
    Release { button: String, timestamp: String },
    /// Cursor type changed
    TypeChange { new_type: String, position: (f64, f64), timestamp: String },
}

impl CursorEvent {
    /// Convert cursor event to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    
    /// Create cursor event from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert cursor event to pretty-formatted JSON string
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Callback function type for cursor events
pub type CursorCallback = Box<dyn Fn(&CursorState, &str) + Send>;

/// Event handler function type for cursor events
pub type CursorEventHandler = Box<dyn Fn(CursorEvent) + Send>;

/// Main cursor detector that monitors cursor activities
pub struct CursorDetector {
    state: CursorState,
    callback: Option<CursorCallback>,
    event_handler: Option<CursorEventHandler>,
}

impl CursorDetector {
    /// Create a new cursor detector
    pub fn new() -> Self {
        Self {
            state: CursorState::new(),
            callback: None,
            event_handler: None,
        }
    }

    /// Set a callback function to be called when cursor events occur
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(&CursorState, &str) + Send + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    /// Set an event handler function to be called when cursor events occur
    pub fn set_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(CursorEvent) + Send + 'static,
    {
        self.event_handler = Some(Box::new(handler));
    }

    /// Get current timestamp in formatted string
    pub fn get_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }

    /// Log a message with timestamp
    pub fn log_message(message: &str) {
        let timestamp = Self::get_timestamp();
        println!("[{}] {}", timestamp, message);
    }

    /// Log cursor position and type
    pub fn log_cursor_state(position: (f64, f64), cursor_type: &str) {
        let timestamp = Self::get_timestamp();
        println!("[{}] Cursor Pos: ({:.0}, {:.0}) | Type: {}", timestamp, position.0, position.1, cursor_type);
    }

    /// Get cursor type name by comparing with standard cursors
    fn get_cursor_type_name(cursor_handle: HCURSOR) -> String {
        unsafe {
            let standard_cursors = [
                (IDC_ARROW, "arrow"),
                (IDC_IBEAM, "ibeam"),
                (IDC_WAIT, "wait"),
                (IDC_CROSS, "cross"),
                (IDC_UPARROW, "up_arrow"),
                (IDC_SIZE, "size"),
                (IDC_SIZENWSE, "size_nw_se"),
                (IDC_SIZENESW, "size_ne_sw"),
                (IDC_SIZEWE, "size_we"),
                (IDC_SIZENS, "size_ns"),
                (IDC_SIZEALL, "size_all"),
                (IDC_NO, "no"),
                (IDC_HAND, "hand"),
                (IDC_APPSTARTING, "app_starting"),
                (IDC_HELP, "help"),
                (IDC_PIN, "pin"),
                (IDC_PERSON, "person"),
            ];

            for (standard_cursor, name) in standard_cursors.iter() {
                if let Ok(standard_handle) = LoadCursorW(None, *standard_cursor) {
                    if cursor_handle.0 == standard_handle.0 {
                        return name.to_string();
                    }
                }
            }

            format!("custom_{:p}", cursor_handle.0)
        }
    }

    /// Get actual cursor type using Windows API
    pub fn get_cursor_type() -> String {
        unsafe {
            let mut cursor_info = CURSORINFO {
                cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                flags: CURSOR_SHOWING,
                hCursor: HCURSOR::default(),
                ptScreenPos: POINT { x: 0, y: 0 },
            };
            
            if GetCursorInfo(&mut cursor_info).is_ok() {
                let cursor_handle = cursor_info.hCursor;
                Self::get_cursor_type_name(cursor_handle)
            } else {
                "error".to_string()
            }
        }
    }

    /// Get current cursor state
    pub fn get_state(&self) -> &CursorState {
        &self.state
    }

    /// Start monitoring cursor activities  
    pub fn start_monitoring(&mut self) -> Result<(), String> {
        let device_state = DeviceState::new();
        
        // Initialize with current mouse position
        let initial_mouse = device_state.get_mouse();
        self.state.position = (initial_mouse.coords.0 as f64, initial_mouse.coords.1 as f64);
        self.state.cursor_type = Self::get_cursor_type();
        self.state.timestamp = Self::get_timestamp();
        
        Self::log_cursor_state(self.state.position, &self.state.cursor_type);

        // Listen for mouse and keyboard events
        let mut current_state = self.state.clone();
        
        if let Err(error) = listen(move |event| {
            match event.event_type {
                EventType::MouseMove { x, y } => {
                    let new_position = (x, y);
                    if new_position != current_state.position {
                        current_state.position = new_position;
                        current_state.timestamp = Self::get_timestamp();
                        
                        let new_cursor_type = Self::get_cursor_type();
                        let cursor_type_changed = new_cursor_type != current_state.cursor_type;
                        
                        if cursor_type_changed {
                            current_state.cursor_type = new_cursor_type.clone();
                            
                            // @note emit cursor type change event
                            let _type_event = CursorEvent::TypeChange {
                                new_type: new_cursor_type.clone(),
                                position: current_state.position,
                                timestamp: current_state.timestamp.clone(),
                            };
                            
                            Self::log_message(&format!("Cursor type changed to: {}", new_cursor_type));
                        }
                        
                        // @note emit cursor move event
                        let _move_event = CursorEvent::Move {
                            position: current_state.position,
                            cursor_type: current_state.cursor_type.clone(),
                            timestamp: current_state.timestamp.clone(),
                        };
                        
                        Self::log_cursor_state(current_state.position, &current_state.cursor_type);
                    }
                }
                EventType::ButtonPress(Button::Left) => {
                    if !current_state.left_click {
                        current_state.left_click = true;
                        current_state.timestamp = Self::get_timestamp();
                        
                        // @note emit click event
                        let _click_event = CursorEvent::Click {
                            button: "left".to_string(),
                            position: current_state.position,
                            timestamp: current_state.timestamp.clone(),
                        };
                        
                        Self::log_message(&format!("Left click at position ({:.0}, {:.0})", 
                            current_state.position.0, current_state.position.1));
                    }
                }
                EventType::ButtonRelease(Button::Left) => {
                    if current_state.left_click {
                        current_state.left_click = false;
                        current_state.timestamp = Self::get_timestamp();
                        
                        // @note emit release event
                        let _release_event = CursorEvent::Release {
                            button: "left".to_string(),
                            timestamp: current_state.timestamp.clone(),
                        };
                        
                        Self::log_message("Left click released");
                    }
                }
                EventType::ButtonPress(Button::Right) => {
                    if !current_state.right_click {
                        current_state.right_click = true;
                        current_state.timestamp = Self::get_timestamp();
                        
                        // @note emit click event
                        let _click_event = CursorEvent::Click {
                            button: "right".to_string(),
                            position: current_state.position,
                            timestamp: current_state.timestamp.clone(),
                        };
                        
                        Self::log_message(&format!("Right click at position ({:.0}, {:.0})", 
                            current_state.position.0, current_state.position.1));
                    }
                }
                EventType::ButtonRelease(Button::Right) => {
                    if current_state.right_click {
                        current_state.right_click = false;
                        current_state.timestamp = Self::get_timestamp();
                        
                        // @note emit release event
                        let _release_event = CursorEvent::Release {
                            button: "right".to_string(),
                            timestamp: current_state.timestamp.clone(),
                        };
                        
                        Self::log_message("Right click released");
                    }
                }
                _ => {}
            }
        }) {
            return Err(format!("Failed to start listening: {:?}", error));
        }

        Ok(())
    }
}

impl Default for CursorDetector {
    fn default() -> Self {
        Self::new()
    }
}
