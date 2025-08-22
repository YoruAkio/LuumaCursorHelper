use luuma_cursor_helper::{CursorDetector, CursorState, CursorEvent};

fn main() {
    println!("=== Luuma Cursor Helper Library Example ===\n");

    // @note create a cursor detector
    let mut detector = CursorDetector::new();
    
    println!("1. Basic Library Usage:");
    println!("   - Created CursorDetector instance");
    println!("   - Ready to monitor cursor activities\n");

    // @note demonstrate cursor state creation
    println!("2. CursorState Creation:");
    let state = CursorState {
        position: (100.0, 200.0),
        cursor_type: "arrow".to_string(),
        left_click: false,
        right_click: false,
        timestamp: CursorDetector::get_timestamp(),
    };
    println!("   Created state: {:?}", state);
    println!("   JSON: {}", state.to_json());
    println!();

    // @note demonstrate event handler usage
    println!("3. Event Handler Setup:");
    detector.set_event_handler(|event: CursorEvent| {
        match event {
            CursorEvent::Move { position, cursor_type, timestamp } => {
                println!("   [EVENT] Cursor moved to {:?} with type '{}' at {}", 
                         position, cursor_type, timestamp);
            }
            CursorEvent::Click { button, position, timestamp } => {
                println!("   [EVENT] {} click at {:?} at {}", 
                         button, position, timestamp);
            }
            CursorEvent::Release { button, timestamp } => {
                println!("   [EVENT] {} button released at {}", 
                         button, timestamp);
            }
            CursorEvent::TypeChange { new_type, position, timestamp } => {
                println!("   [EVENT] Cursor type changed to '{}' at {:?} at {}", 
                         new_type, position, timestamp);
            }
        }
    });
    println!("   Event handler configured to log all cursor events\n");

    // @note demonstrate callback usage
    println!("4. Callback Setup:");
    detector.set_callback(|state: &CursorState, event: &str| {
        println!("   [CALLBACK] {} - Position: {:?}, Type: {}, Left: {}, Right: {}", 
                 event, state.position, state.cursor_type, state.left_click, state.right_click);
    });
    println!("   Callback configured to log state changes\n");

    // @note demonstrate utility functions
    println!("5. Utility Functions:");
    let timestamp = CursorDetector::get_timestamp();
    println!("   Current timestamp: {}", timestamp);
    
    let cursor_type = CursorDetector::get_cursor_type();
    println!("   Current cursor type: {}", cursor_type);
    println!();

    // @note demonstrate JSON serialization
    println!("6. JSON Serialization:");
    let sample_event = CursorEvent::Move {
        position: (500.0, 600.0),
        cursor_type: "hand".to_string(),
        timestamp: CursorDetector::get_timestamp(),
    };
    println!("   Event JSON: {}", sample_event.to_json());
    println!();

    println!("7. Starting Monitoring:");
    println!("   The detector will now start monitoring cursor activities.");
    println!("   Move your mouse and click to see events in action.");
    println!("   Press Ctrl+C to stop.\n");

    // @note start monitoring (this will block)
    if let Err(error) = detector.start_monitoring() {
        eprintln!("Error starting monitoring: {}", error);
    }
}
