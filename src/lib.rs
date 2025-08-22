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
//! - High-performance optimizations with caching and debouncing
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
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, Duration};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;


/// Mouse button types for better performance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl std::fmt::Display for MouseButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseButton::Left => write!(f, "left"),
            MouseButton::Right => write!(f, "right"),
            MouseButton::Middle => write!(f, "middle"),
        }
    }
}

/// Simple cursor type cache with static strings
fn get_cursor_type_static(name: &str) -> &'static str {
    match name {
        "arrow" => "arrow",
        "ibeam" => "ibeam", 
        "wait" => "wait",
        "cross" => "cross",
        "up_arrow" => "up_arrow",
        "size" => "size",
        "size_nw_se" => "size_nw_se",
        "size_ne_sw" => "size_ne_sw",
        "size_we" => "size_we",
        "size_ns" => "size_ns",
        "size_all" => "size_all",
        "no" => "no",
        "hand" => "hand",
        "app_starting" => "app_starting",
        "help" => "help",
        "pin" => "pin",
        "person" => "person",
        _ => "custom",
    }
}

/// Cached cursor information for performance
#[derive(Debug, Clone)]
struct CachedCursor {
    handle: usize, // Store as usize for thread safety
    name: &'static str,
}

/// Global cursor cache for performance optimization
static CURSOR_CACHE: OnceLock<Arc<Vec<CachedCursor>>> = OnceLock::new();

/// Initialize cursor cache once at startup
fn init_cursor_cache() -> Arc<Vec<CachedCursor>> {
    let mut cursors = Vec::new();
    
    unsafe {
        let cursor_pairs = [
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

        for (cursor_id, name) in cursor_pairs {
            if let Ok(handle) = LoadCursorW(None, cursor_id) {
                let static_name = get_cursor_type_static(name);
                cursors.push(CachedCursor { handle: handle.0 as usize, name: static_name });
            }
        }
    }
    
    Arc::new(cursors)
}

/// Get cached cursor type name efficiently
fn get_cached_cursor_type(cursor_handle: HCURSOR) -> &'static str {
    let cache = CURSOR_CACHE.get_or_init(init_cursor_cache);
    
    for cached_cursor in cache.iter() {
        if cursor_handle.0 as usize == cached_cursor.handle {
            return cached_cursor.name;
        }
    }
    
    "custom"
}

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

/// Different types of cursor events with interned strings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CursorEvent {
    /// Cursor moved to a new position
    Move { position: (f64, f64), cursor_type: String, timestamp: String },
    /// Mouse button was clicked
    Click { button: MouseButton, position: (f64, f64), timestamp: String },
    /// Mouse button was released
    Release { button: MouseButton, timestamp: String },
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

/// Smart event batcher with single channel
#[derive(Debug)]
struct SmartEventBatcher {
    events: Vec<CursorEvent>,
    last_flush: Instant,
    flush_interval: Duration,
    max_buffer_size: usize,
    sender: Sender<Vec<CursorEvent>>,
}

impl SmartEventBatcher {
    fn new(flush_interval_ms: u64, max_size: usize, sender: Sender<Vec<CursorEvent>>) -> Self {
        Self {
            events: Vec::with_capacity(max_size),
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(flush_interval_ms),
            max_buffer_size: max_size,
            sender,
        }
    }

    fn add_event(&mut self, event: CursorEvent) -> bool {
        self.events.push(event);
        
        // Smart batching: flush when buffer is full or interval has passed
        if self.events.len() >= self.max_buffer_size || 
           self.last_flush.elapsed() >= self.flush_interval {
            self.flush();
            true
        } else {
            false
        }
    }

    fn flush(&mut self) {
        if !self.events.is_empty() {
            let events = std::mem::take(&mut self.events);
            let _ = self.sender.send(events); // Non-blocking send
            self.last_flush = Instant::now();
        }
    }

    fn force_flush(&mut self) {
        self.flush();
    }
}

/// Lock-free debouncer using atomics
#[derive(Debug)]
struct AtomicDebouncer {
    last_check_ms: AtomicU64,
    interval_ms: u64,
    last_cursor_handle: AtomicU64,
}

impl AtomicDebouncer {
    fn new(interval_ms: u64) -> Self {
        Self {
            last_check_ms: AtomicU64::new(0),
            interval_ms,
            last_cursor_handle: AtomicU64::new(0),
        }
    }

    fn should_check(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let last_check = self.last_check_ms.load(Ordering::Relaxed);
        
        if now_ms.saturating_sub(last_check) >= self.interval_ms {
            self.last_check_ms.store(now_ms, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    fn has_changed(&self, cursor_handle: HCURSOR) -> bool {
        let handle_value = cursor_handle.0 as u64;
        let last_handle = self.last_cursor_handle.swap(handle_value, Ordering::Relaxed);
        handle_value != last_handle
    }
}

/// Lock-free cursor state using atomics for performance
#[derive(Debug)]
struct AtomicCursorState {
    position_x: AtomicU64, // Store as bits of f64
    position_y: AtomicU64,
    left_click: AtomicBool,
    right_click: AtomicBool,
}

impl AtomicCursorState {
    fn new() -> Self {
        Self {
            position_x: AtomicU64::new(0),
            position_y: AtomicU64::new(0),
            left_click: AtomicBool::new(false),
            right_click: AtomicBool::new(false),
        }
    }

    fn update_position(&self, x: f64, y: f64) {
        self.position_x.store(x.to_bits(), Ordering::Relaxed);
        self.position_y.store(y.to_bits(), Ordering::Relaxed);
    }

    fn get_position(&self) -> (f64, f64) {
        let x = f64::from_bits(self.position_x.load(Ordering::Relaxed));
        let y = f64::from_bits(self.position_y.load(Ordering::Relaxed));
        (x, y)
    }

    fn set_left_click(&self, clicked: bool) {
        self.left_click.store(clicked, Ordering::Relaxed);
    }

    fn set_right_click(&self, clicked: bool) {
        self.right_click.store(clicked, Ordering::Relaxed);
    }

    fn get_left_click(&self) -> bool {
        self.left_click.load(Ordering::Relaxed)
    }

    fn get_right_click(&self) -> bool {
        self.right_click.load(Ordering::Relaxed)
    }
}

/// Main cursor detector that monitors cursor activities
pub struct CursorDetector {
    atomic_state: Arc<AtomicCursorState>,
    callback: Option<CursorCallback>,
    event_handler: Option<CursorEventHandler>,
    event_batcher: Option<SmartEventBatcher>,
    _cursor_debouncer: AtomicDebouncer,
    event_sender: Option<Sender<Vec<CursorEvent>>>,
    processing_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl CursorDetector {
    /// Create a new cursor detector
    pub fn new() -> Self {
        Self {
            atomic_state: Arc::new(AtomicCursorState::new()),
            callback: None,
            event_handler: None,
            event_batcher: None,
            _cursor_debouncer: AtomicDebouncer::new(16), // 60fps debouncing
            event_sender: None,
            processing_thread: None,
            running: Arc::new(AtomicBool::new(false)),
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

    /// Get actual cursor type using Windows API with caching
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
                get_cached_cursor_type(cursor_handle).to_string()
            } else {
                "error".to_string()
            }
        }
    }

    /// Get current cursor state (lock-free)
    pub fn get_state(&self) -> CursorState {
        let position = self.atomic_state.get_position();
        CursorState {
            position,
            cursor_type: Self::get_cursor_type(),
            left_click: self.atomic_state.get_left_click(),
            right_click: self.atomic_state.get_right_click(),
            timestamp: Self::get_timestamp(),
        }
    }

    /// Stop monitoring and clean up resources
    pub fn stop(&mut self) -> Result<(), String> {
        // Signal shutdown atomically
        self.running.store(false, Ordering::Relaxed);

        // Force flush event batcher
        if let Some(batcher) = &mut self.event_batcher {
            batcher.force_flush();
        }

        // Wait for processing thread to finish
        if let Some(thread) = self.processing_thread.take() {
            thread.join().map_err(|e| format!("Failed to join thread: {:?}", e))?;
        }

        Ok(())
    }

    /// Check if event handlers are present (conditional event creation)
    fn has_handlers(&self) -> bool {
        self.event_handler.is_some() || self.callback.is_some()
    }

    /// Start monitoring cursor activities  
    pub fn start_monitoring(&mut self) -> Result<(), String> {
        let device_state = DeviceState::new();
        
        // Initialize atomic state with current mouse position
        let initial_mouse = device_state.get_mouse();
        let initial_position = (initial_mouse.coords.0 as f64, initial_mouse.coords.1 as f64);
        self.atomic_state.update_position(initial_position.0, initial_position.1);
        
        Self::log_cursor_state(initial_position, &Self::get_cursor_type());

        // Single channel setup with smart batching
        let (tx, rx) = mpsc::channel();
        self.event_sender = Some(tx.clone());
        
        // Create smart event batcher
        self.event_batcher = Some(SmartEventBatcher::new(50, 100, tx)); // 50ms flush, max 100 events

        // Set running flag atomically
        self.running.store(true, Ordering::Relaxed);

        // Move event handler to processing thread
        let event_handler = self.event_handler.take();
        let running = Arc::clone(&self.running);
        let processing_thread = thread::spawn(move || {
            Self::process_events_with_timeout(rx, event_handler, running);
        });
        self.processing_thread = Some(processing_thread);

        // Listen for mouse and keyboard events
        let atomic_state = Arc::clone(&self.atomic_state);
        let event_sender = self.event_sender.clone();
        let cursor_debouncer = Arc::new(AtomicDebouncer::new(16));
        let running = Arc::clone(&self.running);
        let has_handlers = self.has_handlers();
        
        if let Err(error) = listen(move |event| {
            // Check if we should stop atomically
            if !running.load(Ordering::Relaxed) {
                return;
            }

            match event.event_type {
                EventType::MouseMove { x, y } => {
                    let new_position = (x, y);
                    let current_position = atomic_state.get_position();

                    if new_position != current_position {
                        // Update position atomically
                        atomic_state.update_position(new_position.0, new_position.1);
                        
                        // Only create events if handlers exist (conditional event creation)
                        if has_handlers {
                            let mut events = Vec::new();
                            
                            // Only check cursor type with debouncing
                            if cursor_debouncer.should_check() {
                                unsafe {
                                    let mut cursor_info = CURSORINFO {
                                        cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                                        flags: CURSOR_SHOWING,
                                        hCursor: HCURSOR::default(),
                                        ptScreenPos: POINT { x: 0, y: 0 },
                                    };
                                    
                                    if GetCursorInfo(&mut cursor_info).is_ok() {
                                        if cursor_debouncer.has_changed(cursor_info.hCursor) {
                                            let cursor_type = get_cached_cursor_type(cursor_info.hCursor);
                                            
                                            // Create type change event
                                            let type_event = CursorEvent::TypeChange {
                                                new_type: cursor_type.to_string(),
                                                position: new_position,
                                                timestamp: Self::get_timestamp(),
                                            };
                                            events.push(type_event);
                                            
                                            Self::log_message(&format!("Cursor type changed to: {}", cursor_type));
                                        }
                                    }
                                }
                            }
                            
                            // Create move event with static cursor type
                            let cursor_type = get_cached_cursor_type(unsafe {
                                let mut cursor_info = CURSORINFO {
                                    cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                                    flags: CURSOR_SHOWING,
                                    hCursor: HCURSOR::default(),
                                    ptScreenPos: POINT { x: 0, y: 0 },
                                };
                                if GetCursorInfo(&mut cursor_info).is_ok() {
                                    cursor_info.hCursor
                                } else {
                                    HCURSOR::default()
                                }
                            });
                            
                            let move_event = CursorEvent::Move {
                                position: new_position,
                                cursor_type: cursor_type.to_string(),
                                timestamp: Self::get_timestamp(),
                            };
                            events.push(move_event);
                            
                            // Send events in batch (non-blocking)
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(events);
                            }
                        }
                        
                        Self::log_cursor_state(new_position, get_cached_cursor_type(unsafe {
                            let mut cursor_info = CURSORINFO {
                                cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                                flags: CURSOR_SHOWING,
                                hCursor: HCURSOR::default(),
                                ptScreenPos: POINT { x: 0, y: 0 },
                            };
                            if GetCursorInfo(&mut cursor_info).is_ok() {
                                cursor_info.hCursor
                            } else {
                                HCURSOR::default()
                            }
                        }));
                    }
                }
                EventType::ButtonPress(Button::Left) => {
                    if !atomic_state.get_left_click() {
                        atomic_state.set_left_click(true);
                        
                        // Only create event if handlers exist (conditional event creation)
                        if has_handlers {
                            let position = atomic_state.get_position();
                            let click_event = CursorEvent::Click {
                                button: MouseButton::Left,
                                position,
                                timestamp: Self::get_timestamp(),
                            };
                            
                            // Send event asynchronously (non-blocking)
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(vec![click_event]);
                            }
                        }
                        
                        let position = atomic_state.get_position();
                        Self::log_message(&format!("Left click at position ({:.0}, {:.0})", 
                            position.0, position.1));
                    }
                }
                EventType::ButtonRelease(Button::Left) => {
                    if atomic_state.get_left_click() {
                        atomic_state.set_left_click(false);
                        
                        // Only create event if handlers exist (conditional event creation)
                        if has_handlers {
                            let release_event = CursorEvent::Release {
                                button: MouseButton::Left,
                                timestamp: Self::get_timestamp(),
                            };
                            
                            // Send event asynchronously (non-blocking)
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(vec![release_event]);
                            }
                        }
                        
                        Self::log_message("Left click released");
                    }
                }
                EventType::ButtonPress(Button::Right) => {
                    if !atomic_state.get_right_click() {
                        atomic_state.set_right_click(true);
                        
                        // Only create event if handlers exist (conditional event creation)
                        if has_handlers {
                            let position = atomic_state.get_position();
                            let click_event = CursorEvent::Click {
                                button: MouseButton::Right,
                                position,
                                timestamp: Self::get_timestamp(),
                            };
                            
                            // Send event asynchronously (non-blocking)
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(vec![click_event]);
                            }
                        }
                        
                        let position = atomic_state.get_position();
                        Self::log_message(&format!("Right click at position ({:.0}, {:.0})", 
                            position.0, position.1));
                    }
                }
                EventType::ButtonRelease(Button::Right) => {
                    if atomic_state.get_right_click() {
                        atomic_state.set_right_click(false);
                        
                        // Only create event if handlers exist (conditional event creation)
                        if has_handlers {
                            let release_event = CursorEvent::Release {
                                button: MouseButton::Right,
                                timestamp: Self::get_timestamp(),
                            };
                            
                            // Send event asynchronously (non-blocking)
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(vec![release_event]);
                            }
                        }
                        
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

    /// Process events with proper blocking and timeout (no busy waiting)
    fn process_events_with_timeout(
        receiver: Receiver<Vec<CursorEvent>>,
        event_handler: Option<CursorEventHandler>,
        running: Arc<AtomicBool>
    ) {
        let timeout = Duration::from_millis(100); // 100ms timeout
        
        while running.load(Ordering::Relaxed) {
            // Use blocking receive with timeout to avoid busy waiting
            match receiver.recv_timeout(timeout) {
                Ok(events) => {
                    // Batch process events efficiently
                    if let Some(handler) = &event_handler {
                        for event in events {
                            handler(event);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout is expected, continue loop
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel disconnected, exit gracefully
                    break;
                }
            }
        }
    }
}

impl Default for CursorDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CursorDetector {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
