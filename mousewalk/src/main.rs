use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use rdev::{display_size, listen, simulate, Event, EventType};

// Convert f64 to i32 with rounding (for coordinates)
fn as_i32(x: f64) -> i32 {
    x.round() as i32
}

fn main() {
    // Must get screen size
    let (w, h) = display_size().expect("Failed to get display size");
    let max_x = (w - 1) as i32;
    let max_y = (h - 1) as i32;
    println!("Screen resolution: {} x {}", w, h);

    // Shared flag to signal exit
    let going_exit = Arc::new(AtomicBool::new(false));

    // Shared previous mouse position
    // We store as f64 internally (rdev gives f64)
    let prev = Arc::new(std::sync::Mutex::new((0.0_f64, 0.0_f64)));

    // Spawn listener thread
    {
        let going_exit = Arc::clone(&going_exit);
        let prev = Arc::clone(&prev);

        thread::spawn(move || {
            // Callback for mouse events
            let callback = move |event: Event| {
                if let EventType::MouseMove { x, y } = event.event_type {
                    let mut pp = prev.lock().unwrap();
                    let (x_prev, y_prev) = *pp;

                    // Only compare if prev is nonzero (i.e. after first sample)
                    if x_prev != 0.0 || y_prev != 0.0 {
                        let dx = (x - x_prev).abs();
                        let dy = (y - y_prev).abs();
                        if (dx - dy).abs() > std::f64::EPSILON {
                            // Not exactly diagonal => exit
                            going_exit.store(true, Ordering::SeqCst);
                        }
                    }
                    *pp = (x, y);
                }
            };

            if let Err(err) = listen(callback) {
                eprintln!("Error listening to global events: {:?}", err);
            }
        });
    }

    // Movement direction
    let mut dir_x: i32 = 1;
    let mut dir_y: i32 = 1;

    // We will track a local cursor position (approx) based on events
    // Alternatively we can rely purely on prev (from listener) for boundary checking
    loop {
        if going_exit.load(Ordering::SeqCst) {
            println!("Detected manual mouse move → exiting.");
            break;
        }

        // Simulate moving the mouse by (dir_x, dir_y)
        // rdev simulate requires f64 coords, so we convert to relative move
        let ev = EventType::MouseMove {
            x: dir_x as f64,
            y: dir_y as f64,
        };
        if let Err(err) = simulate(&ev) {
            eprintln!("Failed to simulate mouse move: {:?}", err);
        }

        // Sleep for small interval
        thread::sleep(Duration::from_millis(1));

        // Get last known cursor position from prev
        let (cx, cy) = {
            let pp = prev.lock().unwrap();
            *pp
        };

        let cx_i = as_i32(cx);
        let cy_i = as_i32(cy);

        // Bounce off edges
        if cy_i >= max_y {
            dir_y = -1;
        }
        if cy_i <= 1 {
            dir_y = 1;
        }
        if cx_i >= max_x {
            dir_x = -1;
        }
        if cx_i <= 1 {
            dir_x = 1;
        }
    }
}
