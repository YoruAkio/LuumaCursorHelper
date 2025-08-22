# Luuma Cursor Helper

A high-performance Rust library for detecting cursor position, type, and mouse clicks with real-time logging. This library provides a simple and efficient way to monitor cursor activities in Windows applications using lock-free atomic operations and optimized event processing.

## Features

- **Real-time cursor position tracking** - Monitor cursor movement with precise coordinates
- **Cursor type detection** - Detect cursor types like arrow, hand, I-beam, wait, cross, etc.
- **Mouse click detection** - Track left and right mouse button presses and releases
- **Timestamped logging** - All cursor activities are logged with precise timestamps
- **JSON serialization** - Export cursor data as JSON for easy integration with other projects
- **Event-driven architecture** - Handle cursor events with custom callbacks
- **High-performance optimizations** - Cached cursor handles, debounced checks, async processing
- **Windows API integration** - Uses Windows API for accurate cursor type detection
- **Easy to use** - Simple API with minimal setup required

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
luuma_cursor_helper = { git = "https://github.com/YoruAkio/LuumaCursorHelper" }
```

## Documentation

📚 **Live Documentation**: [https://yoruakio.github.io/LuumaCursorHelper](https://yoruakio.github.io/LuumaCursorHelper)

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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Issues

If you encounter any issues or have questions, please open an issue on GitHub.
