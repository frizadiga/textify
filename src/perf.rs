use std::time::Instant;

pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    pub fn new(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    // write it to a file <git-root>./perf.log
    pub fn log_to_file(&self, file_path: &str) -> std::io::Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        writeln!(file, "{}: {}ms", self.label, self.elapsed_ms())?;
        Ok(())
    }

    pub fn print_elapsed(&self) {
        println!("{}: {}ms", self.label, self.elapsed_ms());
        self.log_to_file("perf.log").unwrap_or_else(|err| {
            eprintln!("Failed to log performance data: {}", err);
        });
    }
}
