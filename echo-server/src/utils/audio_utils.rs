use tokio::process::{Command};
use tokio::io::{AsyncWriteExt};
use std::error::Error;

/// Target normalization loudness in dB
const TARGET_DB: f32 = -14.0;

pub async fn normalize_song_async(input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // Run ffmpeg volumedetect using stdin
    let mut vol_cmd = Command::new("ffmpeg")
        .args(&["-i", "pipe:0", "-af", "volumedetect", "-f", "null", "/dev/null"])
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Write input to ffmpeg stdin
    if let Some(mut stdin) = vol_cmd.stdin.take() {
        stdin.write_all(input).await?;
        stdin.shutdown().await?;
    } else {
    }

    let output = vol_cmd.wait_with_output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg volumedetect failed: {}", stderr).into());
    }

    // Parse mean_volume from stderr
    let stderr_str: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
    let mut mean_db: f32 = 0.0_f32;
    for line in stderr_str.lines() {
        if line.contains("mean_volume:") {
            if let Some(value_str) = line.split(':').nth(1) {
                mean_db = value_str.trim().replace(" dB", "").parse::<f32>()?;
                break;
            }
        }
    }

    let gain_db = TARGET_DB - mean_db;
    let gain_factor = 10f32.powf(gain_db / 20.0);


    // Apply normalization and capture output to memory
    let mut normalize_cmd = Command::new("ffmpeg")
        .args(&[
            "-i", "pipe:0",
            "-af", &format!("volume={}", gain_factor),
            "-b:a", "320k",
            "-f", "mp3",
            "pipe:1",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = normalize_cmd.stdin.take().ok_or("Failed to take stdin")?;
    let input_data = input.to_vec();

    // New async write task
    let write_task = tokio::spawn(async move {
        stdin.write_all(&input_data).await?;
        stdin.shutdown().await?;
        Ok::<(), std::io::Error>(())
    });

    // Concurrently, the main task waits for the process to finish and collects the output. This will read from stdout/stderr, unblocking ffmpeg.
    let norm_output = normalize_cmd.wait_with_output().await?;
    
    // Wait for the task to end to make sure no error were encountered during the process
    write_task.await??;

    if !norm_output.status.success() {
        let stderr = String::from_utf8_lossy(&norm_output.stderr);
        return Err(format!("ffmpeg normalization failed: {}", stderr).into());
    }

    let normalized_bytes = norm_output.stdout;
    Ok(normalized_bytes)
}
