# Luuma Cursor Helper

A Rust library for detecting cursor position, type, and mouse clicks with real-time logging. This library provides a simple and efficient way to monitor cursor activities in Windows applications.

## Features

- **Real-time cursor position tracking** - Monitor cursor movement with precise coordinates
- **Cursor type detection** - Detect cursor types like arrow, hand, I-beam, wait, cross, etc.
- **Mouse click detection** - Track left and right mouse button presses and releases
- **Timestamped logging** - All cursor activities are logged with precise timestamps
- **JSON serialization** - Export cursor data as JSON for easy integration with other projects
- **Event-driven architecture** - Handle cursor events with custom callbacks
- **Windows API integration** - Uses Windows API for accurate cursor type detection
- **Easy to use** - Simple API with minimal setup required

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
luuma_cursor_helper = "0.1.0"
```

## Quick Start

```rust
use luuma_cursor_helper::CursorDetector;

fn main() {
    let mut detector = CursorDetector::new();
    
    // Start monitoring cursor activities
    if let Err(error) = detector.start_monitoring() {
        eprintln!("Error: {:?}", error);
    }
}
```

## API Reference

### CursorDetector

The main struct for monitoring cursor activities.

#### Methods

- `new()` - Create a new cursor detector
- `start_monitoring()` - Start monitoring cursor activities
- `get_state()` - Get current cursor state
- `set_callback()` - Set a callback function for cursor events
- `get_cursor_type()` - Get current cursor type
- `log_message()` - Log a message with timestamp
- `log_cursor_state()` - Log cursor position and type

### CursorState

Represents the current state of the cursor.

#### Fields

- `position: (f64, f64)` - Current cursor position (x, y)
- `cursor_type: String` - Current cursor type (arrow, hand, ibeam, etc.)
- `left_click: bool` - Whether left mouse button is pressed
- `right_click: bool` - Whether right mouse button is pressed
- `timestamp: String` - Timestamp when this state was captured

#### Methods

- `new()` - Create a new cursor state with default values
- `to_json()` - Convert cursor state to JSON string
- `from_json(json: &str)` - Create cursor state from JSON string
- `to_json_pretty()` - Convert cursor state to pretty-formatted JSON string

### CursorEvent

Represents different types of cursor events.

#### Variants

- `Move { position, cursor_type, timestamp }` - Cursor moved to a new position
- `Click { button, position, timestamp }` - Mouse button was clicked
- `Release { button, timestamp }` - Mouse button was released
- `TypeChange { new_type, position, timestamp }` - Cursor type changed

#### Methods

- `to_json()` - Convert cursor event to JSON string
- `from_json(json: &str)` - Create cursor event from JSON string
- `to_json_pretty()` - Convert cursor event to pretty-formatted JSON string

## Usage Examples

### Running Examples

You can run the included examples to see the library in action:

```bash
# Run the library usage example
cargo run --example lib_example

# Run the main cursor monitoring program
cargo run
```

### Basic Usage

```rust
use luuma_cursor_helper::CursorDetector;

fn main() {
    let mut detector = CursorDetector::new();
    detector.start_monitoring().unwrap();
}
```

### With Custom Event Handler

```rust
use luuma_cursor_helper::{CursorDetector, CursorEvent};

fn main() {
    let mut detector = CursorDetector::new();
    
    // Set a custom event handler for cursor events
    detector.set_event_handler(|event: CursorEvent| {
        match event {
            CursorEvent::Move { position, cursor_type, timestamp } => {
                println!("Cursor moved to {:?} with type {} at {}", position, cursor_type, timestamp);
            }
            CursorEvent::Click { button, position, timestamp } => {
                println!("{} click at {:?} at {}", button, position, timestamp);
            }
            CursorEvent::Release { button, timestamp } => {
                println!("{} button released at {}", button, timestamp);
            }
            CursorEvent::TypeChange { new_type, position, timestamp } => {
                println!("Cursor type changed to {} at {:?} at {}", new_type, position, timestamp);
            }
        }
    });
    
    detector.start_monitoring().unwrap();
}
```

### JSON Serialization

```rust
use luuma_cursor_helper::{CursorDetector, CursorState, CursorEvent};

fn main() {
    // Create a cursor state
    let state = CursorState {
        position: (100.0, 200.0),
        cursor_type: "arrow".to_string(),
        left_click: false,
        right_click: false,
        timestamp: CursorDetector::get_timestamp(),
    };

    // Convert to JSON
    let json = state.to_json();
    println!("JSON: {}", json);
    // Output: {"position":[100.0,200.0],"cursor_type":"arrow","left_click":false,"right_click":false,"timestamp":"2024-01-01 12:00:00.000"}

    // Convert from JSON
    let restored_state = CursorState::from_json(&json).unwrap();
    println!("Restored: {:?}", restored_state);

    // Pretty JSON formatting
    let pretty_json = state.to_json_pretty();
    println!("Pretty JSON:\n{}", pretty_json);
}
```

### Get Current Cursor State

```rust
use luuma_cursor_helper::CursorDetector;

fn main() {
    let mut detector = CursorDetector::new();
    
    // Get current cursor state
    let state = detector.get_state();
    println!("Current position: {:?}", state.position);
    println!("Current type: {}", state.cursor_type);
}
```

## Supported Cursor Types

The library can detect the following cursor types:

- `arrow` - Standard arrow cursor
- `ibeam` - Text selection cursor (I-beam)
- `hand` - Hand cursor for clickable elements
- `wait` - Hourglass/wait cursor
- `cross` - Crosshair cursor
- `up_arrow` - Up arrow cursor
- `size` - Size cursor
- `size_nw_se` - Northwest-southeast resize cursor
- `size_ne_sw` - Northeast-southwest resize cursor
- `size_we` - Horizontal resize cursor
- `size_ns` - Vertical resize cursor
- `size_all` - Move cursor
- `no` - Forbidden cursor
- `app_starting` - Application starting cursor
- `help` - Help cursor
- `pin` - Pin cursor
- `person` - Person cursor
- `custom_*` - Custom cursors (with hex address)

## Output Format

### Console Logging

The library logs cursor activities in the following format:

```
[2025-08-22 04:21:11.222] Cursor Pos: (1190, 902) | Type: arrow
[2025-08-22 04:21:11.223] Cursor type changed to: hand
[2025-08-22 04:21:11.224] Left click at position (1190, 902)
[2025-08-22 04:21:11.225] Left click released
```

### JSON Output

#### CursorState JSON:
```json
{
  "position": [1190.0, 902.0],
  "cursor_type": "arrow",
  "left_click": false,
  "right_click": false,
  "timestamp": "2025-08-22 04:21:11.222"
}
```

#### CursorEvent JSON Examples:

**Move Event:**
```json
{
  "Move": {
    "position": [1190.0, 902.0],
    "cursor_type": "arrow",
    "timestamp": "2025-08-22 04:21:11.222"
  }
}
```

**Click Event:**
```json
{
  "Click": {
    "button": "left",
    "position": [1190.0, 902.0],
    "timestamp": "2025-08-22 04:21:11.223"
  }
}
```

**Release Event:**
```json
{
  "Release": {
    "button": "left",
    "timestamp": "2025-08-22 04:21:11.224"
  }
}
```

**Type Change Event:**
```json
{
  "TypeChange": {
    "new_type": "hand",
    "position": [1190.0, 902.0],
    "timestamp": "2025-08-22 04:21:11.225"
  }
}
```

## Requirements

- Windows operating system
- Rust 1.70 or later
- Windows API access

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Yoru Akio**

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Issues

If you encounter any issues or have questions, please open an issue on GitHub.
