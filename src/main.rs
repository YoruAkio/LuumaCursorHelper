use luuma_cursor_helper::CursorDetector;

fn main() {
    println!("Cursor Detection Program Started");
    println!("Monitoring cursor position, type, and mouse clicks...");
    println!("Press Ctrl+C to exit\n");

    let mut detector = CursorDetector::new();
    
    if let Err(error) = detector.start_monitoring() {
        eprintln!("Error: {:?}", error);
    }
}
