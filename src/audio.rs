#[cfg(feature = "audio")]
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
#[cfg(feature = "audio")]
use std::io::Cursor;

// Embed sound files at compile time (only when audio feature is enabled)
#[cfg(feature = "audio")]
const WORK_TO_BREAK_SOUND: &[u8] = include_bytes!("../assets/sounds/work-to-break.wav");
#[cfg(feature = "audio")]
const BREAK_TO_WORK_SOUND: &[u8] = include_bytes!("../assets/sounds/break-to-work.wav");
#[cfg(feature = "audio")]
const WORK_TO_LONG_BREAK_SOUND: &[u8] = include_bytes!("../assets/sounds/work-to-long-break.wav");

#[derive(Debug, Clone, Copy)]
pub enum SoundType {
    WorkToBreak,
    BreakToWork,
    WorkToLongBreak,
}

#[cfg(feature = "audio")]
pub fn play_embedded_sound(
    sound_type: SoundType,
    volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let sound_data = match sound_type {
        SoundType::WorkToBreak => WORK_TO_BREAK_SOUND,
        SoundType::BreakToWork => BREAK_TO_WORK_SOUND,
        SoundType::WorkToLongBreak => WORK_TO_LONG_BREAK_SOUND,
    };

    // Check if the sound data is just a placeholder (empty/minimal WAV)
    if sound_data.len() <= 44 {
        // Fallback to system beep for placeholder files
        play_system_beep();
        return Ok(());
    }

    // Use tokio::spawn_blocking for audio playback to avoid blocking the async runtime
    // and prevent holding the audio device open
    let sound_data = sound_data.to_vec();
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::spawn_blocking(move || {
            if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                let sink = Sink::connect_new(_stream.mixer());

                let cursor = Cursor::new(sound_data);
                if let Ok(source) = Decoder::new(cursor) {
                    let source = source.amplify(volume);
                    sink.append(source);
                    // Wait for playback to complete before dropping the stream
                    sink.sleep_until_end();
                }
                // Stream is dropped here, releasing the audio device
            }
        });
    } else {
        // Fallback to std::thread if not in tokio runtime (e.g., tests without runtime)
        std::thread::spawn(move || {
            if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                let sink = Sink::connect_new(_stream.mixer());

                let cursor = Cursor::new(sound_data);
                if let Ok(source) = Decoder::new(cursor) {
                    let source = source.amplify(volume);
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        });
    }

    Ok(())
}

#[cfg(feature = "audio")]
pub fn play_system_beep() {
    // Use tokio::spawn_blocking for beep playback
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::spawn_blocking(|| {
            if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                let sink = Sink::connect_new(_stream.mixer());

                // Generate a simple beep tone
                let source = rodio::source::SineWave::new(800.0)
                    .take_duration(std::time::Duration::from_millis(300))
                    .amplify(0.3);

                sink.append(source);
                sink.sleep_until_end();
                // Stream is dropped here, releasing the audio device
            }
        });
    } else {
        // Fallback to std::thread if not in tokio runtime
        std::thread::spawn(|| {
            if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                let sink = Sink::connect_new(_stream.mixer());

                let source = rodio::source::SineWave::new(800.0)
                    .take_duration(std::time::Duration::from_millis(300))
                    .amplify(0.3);

                sink.append(source);
                sink.sleep_until_end();
            }
        });
    }
}

#[cfg(feature = "audio")]
pub fn play_custom_file<P: AsRef<std::path::Path>>(
    path: P,
    volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load file data before spawning task
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut buffer)?;

    // Use tokio::spawn_blocking for audio playback
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::spawn_blocking(move || {
            if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                let sink = Sink::connect_new(_stream.mixer());

                let cursor = Cursor::new(buffer);
                if let Ok(source) = Decoder::new(cursor) {
                    let source = source.amplify(volume);
                    sink.append(source);
                    // Wait for playback to complete before dropping the stream
                    sink.sleep_until_end();
                }
                // Stream is dropped here, releasing the audio device
            }
        });
    } else {
        // Fallback to std::thread if not in tokio runtime
        std::thread::spawn(move || {
            if let Ok(_stream) = OutputStreamBuilder::open_default_stream() {
                let sink = Sink::connect_new(_stream.mixer());

                let cursor = Cursor::new(buffer);
                if let Ok(source) = Decoder::new(cursor) {
                    let source = source.amplify(volume);
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        });
    }

    Ok(())
}

#[cfg(not(feature = "audio"))]
pub fn play_embedded_sound(
    _sound_type: SoundType,
    _volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Audio feature not enabled, do nothing
    Ok(())
}

#[cfg(not(feature = "audio"))]
pub fn play_system_beep() {
    // Audio feature not enabled, do nothing
}

#[cfg(not(feature = "audio"))]
pub fn play_custom_file<P: AsRef<std::path::Path>>(
    _path: P,
    _volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Audio feature not enabled, do nothing
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_types() {
        // Test that SoundType enum variants exist
        let _work_to_break = SoundType::WorkToBreak;
        let _break_to_work = SoundType::BreakToWork;
        let _work_to_long_break = SoundType::WorkToLongBreak;
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_embedded_sounds_exist() {
        // Only test when audio feature is enabled
        #[cfg(feature = "audio")]
        {
            // Test that embedded sound data exists (even if placeholder)
            assert!(!WORK_TO_BREAK_SOUND.is_empty());
            assert!(!BREAK_TO_WORK_SOUND.is_empty());
            assert!(!WORK_TO_LONG_BREAK_SOUND.is_empty());
        }
    }
}
