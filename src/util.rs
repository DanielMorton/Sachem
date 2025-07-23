use std::time::Instant;

pub fn print_hms(start: &Instant) {
    let millis = start.elapsed().as_millis();
    let seconds = millis / 1000;
    let (hour, minute, second) = (seconds / 3600, (seconds % 3600) / 60, seconds % 60);
    println!(
        "Elapsed time: {:02}:{:02}:{:02}.{:03}",
        hour,
        minute,
        second,
        millis % 1000
    );
}
