#[cfg(feature = "audio")]
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
#[cfg(feature = "audio")]
use std::io::Cursor;

#[cfg(feature = "audio")]
pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
}

#[cfg(not(feature = "audio"))]
pub struct AudioPlayer;

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
impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stream = OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(stream.mixer());

        Ok(AudioPlayer {
            _stream: stream,
            sink,
        })
    }

    pub fn play_embedded_sound(
        &self,
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
            self.play_system_beep();
            return Ok(());
        }

        let cursor = Cursor::new(sound_data);
        match Decoder::new(cursor) {
            Ok(source) => {
                let source = source.amplify(volume);
                self.sink.append(source);
            }
            Err(_) => {
                // If decoding fails, fall back to system beep
                self.play_system_beep();
            }
        }

        Ok(())
    }

    pub fn play_system_beep(&self) {
        // Generate a simple beep tone
        let source = rodio::source::SineWave::new(800.0)
            .take_duration(std::time::Duration::from_millis(300))
            .amplify(0.3);

        self.sink.append(source);
    }

    pub fn play_custom_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        volume: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let source = Decoder::new(std::io::BufReader::new(file))?.amplify(volume);

        self.sink.append(source);
        Ok(())
    }
}

#[cfg(not(feature = "audio"))]
impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AudioPlayer)
    }

    pub fn play_embedded_sound(
        &self,
        _sound_type: SoundType,
        _volume: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Audio feature not enabled, do nothing
        Ok(())
    }

    pub fn play_system_beep(&self) {
        // Audio feature not enabled, do nothing
    }

    pub fn play_custom_file<P: AsRef<std::path::Path>>(
        &self,
        _path: P,
        _volume: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Audio feature not enabled, do nothing
        Ok(())
    }
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
