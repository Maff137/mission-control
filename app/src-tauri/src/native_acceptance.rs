//! Opt-in debug-only packaged acceptance synchronization. No IPC or network surface.
use std::{io::Write, path::Path, time::Duration};

pub fn verify(identifier: &str, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !identifier.starts_with("com.missioncontrol.acceptance.")
        || directory.file_name().and_then(|name| name.to_str()) != Some(identifier)
        || directory.is_symlink()
    {
        return Err(
            "Acceptance builds require a dedicated, non-symlink native storage identifier".into(),
        );
    }
    println!("ACCEPTANCE_STORAGE={}", directory.display());
    std::io::stdout().flush()?;
    Ok(())
}

pub fn checkpoint(boundary: &str) {
    if std::env::var("MC_ACCEPTANCE_BOUNDARY").as_deref() == Ok(boundary) {
        println!("ACCEPTANCE_BOUNDARY={boundary}");
        std::io::stdout().flush().expect("acceptance stdout");
        // Bounded wait. The controller kills only the Child it spawned after this signal.
        for _ in 0..600 {
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("Acceptance controller did not terminate the checkpoint process");
    }
}
