//! Thin wrappers around `ffmpeg`/`ffprobe` CLI invocations for the audio
//! production pipeline (docs/12_AUDIO_PRODUCTION.md §5).
//!
//! All I/O uses filenames relative to a per-asset working directory
//! (`Command::current_dir`) rather than absolute paths: ffmpeg's filter-graph
//! option parser treats `:` as a value separator, which collides with a
//! Windows drive letter (`D:\...`) unless painstakingly escaped. Relative
//! paths sidestep the problem entirely.

use serde::Deserialize;
use std::path::Path;
use std::process::Command;

fn run(workdir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .current_dir(workdir)
        .arg("-y")
        .args(args)
        .output()
        .map_err(|e| format!("failed to spawn ffmpeg: {e}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(format!("ffmpeg failed ({:?}): {stderr}", output.status));
    }
    Ok(stderr)
}

/// Write `text` to a temp file in `workdir` and synthesize it with ffmpeg's
/// `flite` lavfi source (48kHz mono WAV).
pub fn synthesize_segment(workdir: &Path, text: &str, voice: &str, seg_txt: &str, out_wav: &str) -> Result<(), String> {
    std::fs::write(workdir.join(seg_txt), text).map_err(|e| e.to_string())?;
    let filter = format!("flite=textfile={seg_txt}:voice={voice}");
    run(workdir, &["-f", "lavfi", "-i", &filter, "-ar", "48000", "-ac", "1", out_wav])?;
    Ok(())
}

pub fn make_silence(workdir: &Path, duration_sec: f64, out_wav: &str) -> Result<(), String> {
    let dur = format!("{duration_sec}");
    run(
        workdir,
        &["-f", "lavfi", "-i", "anullsrc=r=48000:cl=mono", "-t", &dur, out_wav],
    )?;
    Ok(())
}

/// Concatenate WAV parts in order (ffmpeg's concat demuxer). A single part
/// is just copied — no need to invoke ffmpeg for a no-op concat.
pub fn concat_wavs(workdir: &Path, parts: &[String], out_wav: &str) -> Result<(), String> {
    if parts.len() == 1 {
        std::fs::copy(workdir.join(&parts[0]), workdir.join(out_wav)).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let list = parts.iter().map(|p| format!("file '{p}'\n")).collect::<String>();
    std::fs::write(workdir.join("concat.txt"), list).map_err(|e| e.to_string())?;
    run(
        workdir,
        &["-f", "concat", "-safe", "0", "-i", "concat.txt", "-c:a", "pcm_s16le", "-ar", "48000", "-ac", "1", out_wav],
    )?;
    Ok(())
}

/// Last-resort speech-rate correction (spec §3: "only as a last resort
/// time-stretch with `ffmpeg -af atempo` within 0.93-1.07, never beyond").
/// `factor` > 1 speeds up (shortens), < 1 slows down (lengthens).
pub fn apply_atempo(workdir: &Path, input: &str, output: &str, factor: f64) -> Result<(), String> {
    let clamped = factor.clamp(0.93, 1.07);
    let filter = format!("atempo={clamped}");
    run(workdir, &["-i", input, "-af", &filter, output])?;
    Ok(())
}

/// Trim silence below -50dBFS from both ends (reverse-trim-reverse, since
/// `silenceremove` only trims the head).
pub fn trim_silence(workdir: &Path, input: &str, output: &str) -> Result<(), String> {
    let filter = "silenceremove=start_periods=1:start_threshold=-50dB:start_silence=0.05,areverse,\
                  silenceremove=start_periods=1:start_threshold=-50dB:start_silence=0.05,areverse";
    run(workdir, &["-i", input, "-af", filter, output])?;
    Ok(())
}

/// Pad 0.5s lead-in / 1.0s tail (digital silence), per spec §5.2.
pub fn pad_lead_tail(workdir: &Path, input: &str, output: &str) -> Result<(), String> {
    run(workdir, &["-i", input, "-af", "adelay=500|500,apad=pad_dur=1.0", output])?;
    Ok(())
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoudnormMeasurement {
    pub input_i: String,
    pub input_tp: String,
    pub input_lra: String,
    pub input_thresh: String,
    pub target_offset: String,
}

fn extract_json(stderr: &str) -> Result<serde_json::Value, String> {
    let start = stderr.rfind('{').ok_or("no JSON block in ffmpeg output")?;
    let end = stderr.rfind('}').ok_or("no JSON block in ffmpeg output")?;
    serde_json::from_str(&stderr[start..=end]).map_err(|e| format!("loudnorm JSON parse failed: {e}"))
}

/// Pass 1: measure loudness/peak stats without modifying audio.
pub fn loudnorm_measure(workdir: &Path, input: &str) -> Result<LoudnormMeasurement, String> {
    let filter = "loudnorm=I=-16:LRA=11:TP=-1.5:print_format=json";
    let stderr = run(workdir, &["-i", input, "-af", filter, "-f", "null", "-"])?;
    let v = extract_json(&stderr)?;
    serde_json::from_value(v).map_err(|e| format!("loudnorm measurement decode failed: {e}"))
}

#[derive(Debug, Deserialize)]
pub struct LoudnormApplied {
    pub output_i: String,
    pub output_tp: String,
}

/// Pass 2: apply linear loudness normalisation using the pass-1 measurement.
/// Returns the filter's own report of what it actually produced
/// (`output_i`/`output_tp`) — the authoritative, real post-normalisation
/// loudness/peak, not a re-derived estimate.
pub fn loudnorm_apply(workdir: &Path, input: &str, output: &str, m: &LoudnormMeasurement) -> Result<LoudnormApplied, String> {
    let filter = format!(
        "loudnorm=I=-16:LRA=11:TP=-1.5:measured_I={}:measured_LRA={}:measured_TP={}:measured_thresh={}:offset={}:linear=true:print_format=json",
        m.input_i, m.input_lra, m.input_tp, m.input_thresh, m.target_offset
    );
    let stderr = run(workdir, &["-i", input, "-af", &filter, "-ar", "48000", "-c:a", "pcm_s24le", output])?;
    let v = extract_json(&stderr)?;
    serde_json::from_value(v).map_err(|e| format!("loudnorm apply-result decode failed: {e}"))
}

pub fn encode_opus(workdir: &Path, input: &str, output: &str) -> Result<(), String> {
    run(workdir, &["-i", input, "-c:a", "libopus", "-b:a", "48k", "-ac", "1", output])?;
    Ok(())
}

pub fn encode_mp3(workdir: &Path, input: &str, output: &str) -> Result<(), String> {
    run(workdir, &["-i", input, "-c:a", "libmp3lame", "-b:a", "128k", "-ac", "1", output])?;
    Ok(())
}

/// Real measured duration in seconds (ffprobe).
pub fn probe_duration(workdir: &Path, input: &str) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .current_dir(workdir)
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", input])
        .output()
        .map_err(|e| format!("failed to spawn ffprobe: {e}"))?;
    if !output.status.success() {
        return Err(format!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("ffprobe duration parse failed: {e}"))
}

/// Overall peak level in dBFS (`astats`). Used as a real (not fabricated)
/// clipping proxy: the two-pass `loudnorm` above already enforces a
/// -1.5dBTP ceiling mathematically, so a peak at/above ~0dBFS here would
/// indicate the pipeline actually failed to limit the signal.
pub fn measure_peak_db(workdir: &Path, input: &str) -> Result<f64, String> {
    let output = Command::new("ffmpeg")
        .current_dir(workdir)
        .args(["-i", input, "-af", "astats=metadata=0:reset=0", "-f", "null", "-"])
        .output()
        .map_err(|e| format!("failed to spawn ffmpeg (astats): {e}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines().rev() {
        if let Some(rest) = line.trim().strip_prefix("Peak level dB:") {
            return rest.trim().parse::<f64>().map_err(|e| format!("astats peak parse failed: {e}"));
        }
    }
    Err("astats did not report a peak level".to_string())
}
