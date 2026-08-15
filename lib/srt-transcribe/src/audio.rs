use anyhow::{Context as _, Result, anyhow};
use std::path::Path;
use tokio_util::sync::CancellationToken;

fn ffmpeg_command(ffmpeg_path: &str) -> tokio::process::Command {
    #[allow(unused_mut)]
    let mut command = tokio::process::Command::new(ffmpeg_path);
    #[cfg(windows)]
    {
        command.creation_flags(0x0800_4000);
    }
    command
}

pub async fn convert_to_wav(
    ffmpeg_path: &str,
    input_path: &Path,
    output_path: &Path,
    cancel_token: Option<&CancellationToken>,
) -> Result<()> {
    let mut child = ffmpeg_command(ffmpeg_path)
        .arg("-y")
        .arg("-i")
        .arg(input_path)
        .args(["-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le"])
        .arg(output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn FFmpeg process for audio conversion")?;

    let mut stderr_reader = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stderr"))?;

    let stderr_handle = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = stderr_reader.read_to_end(&mut buf).await;
        buf
    });

    let res = if let Some(token) = cancel_token {
        tokio::select! {
            res = child.wait() => res,
            _ = token.cancelled() => {
                let _ = child.kill().await;
                anyhow::bail!("Audio conversion cancelled");
            }
        }
    } else {
        child.wait().await
    };

    let status = res.context("Failed to wait for FFmpeg process")?;
    let stderr_bytes = stderr_handle.await.unwrap_or_default();

    if !status.success() {
        let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();
        anyhow::bail!("FFmpeg audio conversion failed: {}", stderr);
    }

    Ok(())
}

pub async fn segment_to_wav_chunks(
    ffmpeg_path: &str,
    input_path: &Path,
    out_dir: &Path,
    segment_seconds: u32,
    cancel_token: Option<&CancellationToken>,
) -> Result<Vec<std::path::PathBuf>> {
    let pattern = out_dir.join("chunk_%05d.wav");
    let mut child = ffmpeg_command(ffmpeg_path)
        .arg("-y")
        .arg("-i")
        .arg(input_path)
        .args([
            "-ar",
            "16000",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            "-f",
            "segment",
            "-segment_time",
        ])
        .arg(segment_seconds.max(1).to_string())
        .arg(&pattern)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn FFmpeg process for audio segmentation")?;

    let mut stderr_reader = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stderr"))?;
    let stderr_handle = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = stderr_reader.read_to_end(&mut buf).await;
        buf
    });

    let res = if let Some(token) = cancel_token {
        tokio::select! {
            res = child.wait() => res,
            _ = token.cancelled() => {
                let _ = child.kill().await;
                anyhow::bail!("Audio segmentation cancelled");
            }
        }
    } else {
        child.wait().await
    };

    let status = res.context("Failed to wait for FFmpeg process")?;
    let stderr_bytes = stderr_handle.await.unwrap_or_default();
    if !status.success() {
        let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();
        anyhow::bail!("FFmpeg audio segmentation failed: {}", stderr);
    }

    let mut chunks: Vec<std::path::PathBuf> = std::fs::read_dir(out_dir)
        .context("Failed to read segment output dir")?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().map(|x| x == "wav").unwrap_or(false)
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("chunk_"))
                    .unwrap_or(false)
        })
        .collect();
    chunks.sort();

    if chunks.is_empty() {
        anyhow::bail!("FFmpeg produced no audio chunks");
    }
    Ok(chunks)
}

pub fn read_wav_to_f32(wav_path: &Path) -> Result<Vec<f32>> {
    let reader = hound::WavReader::open(wav_path).context("Failed to open WAV file")?;
    let spec = reader.spec();
    let total_samples = reader.len() as usize;
    let channels = spec.channels.max(1) as usize;
    let expected_mono_samples = total_samples / channels;

    let mut mono_samples = Vec::with_capacity(expected_mono_samples);

    match (spec.sample_format, spec.channels) {
        (hound::SampleFormat::Int, 1) => {
            let max_val = (1 << (spec.bits_per_sample.saturating_sub(1))) as f32;
            for s in reader.into_samples::<i32>().flatten() {
                mono_samples.push(s as f32 / max_val);
            }
        }
        (hound::SampleFormat::Int, 2) => {
            let max_val = (1 << (spec.bits_per_sample.saturating_sub(1))) as f32;
            let mut iter = reader.into_samples::<i32>().flatten();
            while let Some(left) = iter.next() {
                if let Some(right) = iter.next() {
                    mono_samples.push(((left as f32 + right as f32) / 2.0) / max_val);
                } else {
                    mono_samples.push(left as f32 / max_val);
                }
            }
        }
        (hound::SampleFormat::Int, ch) => {
            let max_val = (1 << (spec.bits_per_sample.saturating_sub(1))) as f32;
            let mut iter = reader.into_samples::<i32>().flatten();
            let mut chunk = Vec::with_capacity(ch as usize);
            loop {
                chunk.clear();
                for _ in 0..ch {
                    if let Some(s) = iter.next() {
                        chunk.push(s);
                    } else {
                        break;
                    }
                }
                if chunk.is_empty() {
                    break;
                }
                let sum: f32 = chunk.iter().map(|&s| s as f32).sum();
                mono_samples.push((sum / chunk.len() as f32) / max_val);
            }
        }
        (hound::SampleFormat::Float, 1) => {
            for s in reader.into_samples::<f32>().flatten() {
                mono_samples.push(s);
            }
        }
        (hound::SampleFormat::Float, 2) => {
            let mut iter = reader.into_samples::<f32>().flatten();
            while let Some(left) = iter.next() {
                if let Some(right) = iter.next() {
                    mono_samples.push((left + right) / 2.0);
                } else {
                    mono_samples.push(left);
                }
            }
        }
        (hound::SampleFormat::Float, ch) => {
            let mut iter = reader.into_samples::<f32>().flatten();
            let mut chunk = Vec::with_capacity(ch as usize);
            loop {
                chunk.clear();
                for _ in 0..ch {
                    if let Some(s) = iter.next() {
                        chunk.push(s);
                    } else {
                        break;
                    }
                }
                if chunk.is_empty() {
                    break;
                }
                let sum: f32 = chunk.iter().sum();
                mono_samples.push(sum / chunk.len() as f32);
            }
        }
    }

    Ok(mono_samples)
}
