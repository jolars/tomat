use crate::config::SoundConfig;

#[derive(Debug, Clone, Copy)]
pub enum SoundType {
    WorkToBreak,
    BreakToWork,
    WorkToLongBreak,
}

#[derive(Default)]
pub struct AudioController {
    #[cfg(feature = "audio")]
    sender: Option<std::sync::mpsc::Sender<playback::Command>>,
    #[cfg(feature = "audio")]
    worker: Option<std::thread::JoinHandle<()>>,
}

impl AudioController {
    pub fn new(config: &SoundConfig) -> Self {
        #[cfg(feature = "audio")]
        if std::env::var_os("TOMAT_TESTING").is_none() {
            let (sender, receiver) = std::sync::mpsc::channel();
            let config = config.clone();
            match std::thread::Builder::new()
                .name("tomat-audio".into())
                .spawn(move || playback::run(receiver, config, playback::RodioBackend::default()))
            {
                Ok(worker) => {
                    return Self {
                        sender: Some(sender),
                        worker: Some(worker),
                    };
                }
                Err(error) => eprintln!("Could not start audio worker: {error}"),
            }
        }
        #[cfg(not(feature = "audio"))]
        let _ = config;
        Self::default()
    }

    #[cfg(test)]
    pub fn silent() -> Self {
        Self::default()
    }

    pub fn set_ticking(&self, active: bool) {
        #[cfg(feature = "audio")]
        if let Some(sender) = &self.sender {
            let _ = sender.send(playback::Command::Ticking(active));
        }
        #[cfg(not(feature = "audio"))]
        let _ = active;
    }

    pub fn play_transition(&self, sound: SoundType) {
        #[cfg(feature = "audio")]
        if let Some(sender) = &self.sender {
            let _ = sender.send(playback::Command::Transition(sound));
        }
        #[cfg(not(feature = "audio"))]
        let _ = sound;
    }
}

impl Drop for AudioController {
    fn drop(&mut self) {
        #[cfg(feature = "audio")]
        {
            if let Some(sender) = self.sender.take() {
                let _ = sender.send(playback::Command::Shutdown);
            }
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }
}

#[cfg(feature = "audio")]
mod playback {
    use super::SoundType;
    use crate::config::{SoundConfig, SoundMode, TickingConfig};
    use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
    use std::io::Cursor;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{Receiver, RecvTimeoutError};
    use std::time::Duration;

    const WORK_TO_BREAK: &[u8] = include_bytes!("../assets/sounds/work-to-break.wav");
    const BREAK_TO_WORK: &[u8] = include_bytes!("../assets/sounds/break-to-work.wav");
    const WORK_TO_LONG_BREAK: &[u8] = include_bytes!("../assets/sounds/work-to-long-break.wav");
    const TICKING: &[u8] = include_bytes!("../assets/sounds/ticking.wav");
    type AudioResult<T> = Result<T, Box<dyn std::error::Error>>;
    type AudioSource = Box<dyn Source<Item = f32> + Send>;

    pub(super) enum Command {
        Ticking(bool),
        Transition(SoundType),
        Shutdown,
    }

    pub(super) trait Backend {
        fn start_ticking(&mut self, config: &TickingConfig) -> AudioResult<()>;
        fn stop_ticking(&mut self);
        fn play_transition(&mut self, config: &SoundConfig, sound: SoundType) -> AudioResult<()>;
        /// Release completed playback and report whether the device is still active.
        fn maintain(&mut self) -> bool;
        fn close(&mut self);
    }

    pub(super) fn run(receiver: Receiver<Command>, config: SoundConfig, mut backend: impl Backend) {
        let mut ticking_requested = false;
        loop {
            // Poll only while playing so finished alerts and failed devices are released.
            let command = if backend.maintain() {
                match receiver.recv_timeout(Duration::from_millis(25)) {
                    Ok(command) => command,
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            } else {
                match receiver.recv() {
                    Ok(command) => command,
                    Err(_) => break,
                }
            };
            match command {
                Command::Ticking(active) => {
                    let active = active && config.ticking.enabled;
                    if active != ticking_requested {
                        ticking_requested = active;
                        if active {
                            if let Err(error) = backend.start_ticking(&config.ticking) {
                                eprintln!("Could not start ticking audio: {error}");
                            }
                        } else {
                            backend.stop_ticking();
                        }
                    }
                }
                Command::Transition(sound) => {
                    ticking_requested = false;
                    backend.stop_ticking();
                    if config.effective_mode() != SoundMode::None
                        && let Err(error) = backend.play_transition(&config, sound)
                    {
                        eprintln!("Could not play transition audio: {error}");
                    }
                }
                Command::Shutdown => break,
            }
        }
        backend.close();
    }

    #[derive(Default)]
    pub(super) struct RodioBackend {
        output: Option<Output>,
    }

    struct Output {
        ticking: Option<Player>,
        alerts: Vec<Player>,
        failed: Arc<AtomicBool>,
        sink: MixerDeviceSink,
    }

    impl RodioBackend {
        fn output(&mut self) -> AudioResult<&mut Output> {
            if self.output.is_none() {
                let failed = Arc::new(AtomicBool::new(false));
                let on_error = failed.clone();
                let mut sink = DeviceSinkBuilder::from_default_device()?
                    .with_error_callback(move |_| {
                        on_error.store(true, Ordering::Relaxed);
                    })
                    .open_sink_or_fallback()?;
                sink.log_on_drop(false);
                self.output = Some(Output {
                    ticking: None,
                    alerts: Vec::new(),
                    failed,
                    sink,
                });
            }
            Ok(self.output.as_mut().unwrap())
        }
    }

    impl Backend for RodioBackend {
        fn start_ticking(&mut self, config: &TickingConfig) -> AudioResult<()> {
            let source = load_source(config.file.as_deref(), TICKING)?;
            let output = self.output()?;
            let player = Player::connect_new(output.sink.mixer());
            player.set_volume(volume(config.volume, 0.2));
            player.append(source.repeat_infinite());
            output.ticking = Some(player);
            Ok(())
        }

        fn stop_ticking(&mut self) {
            if let Some(output) = &mut self.output
                && let Some(player) = output.ticking.take()
            {
                player.stop();
            }
        }

        fn play_transition(&mut self, config: &SoundConfig, sound: SoundType) -> AudioResult<()> {
            let source: AudioSource = match config.effective_mode() {
                SoundMode::None => return Ok(()),
                SoundMode::SystemBeep => beep(),
                SoundMode::Embedded => {
                    let (file, embedded) = match sound {
                        SoundType::WorkToBreak => (config.work_to_break.as_deref(), WORK_TO_BREAK),
                        SoundType::BreakToWork => (config.break_to_work.as_deref(), BREAK_TO_WORK),
                        SoundType::WorkToLongBreak => {
                            (config.work_to_long_break.as_deref(), WORK_TO_LONG_BREAK)
                        }
                    };
                    match load_source(file, embedded) {
                        Ok(source) => Box::new(source.amplify(volume(config.volume, 0.5))),
                        Err(error) => {
                            eprintln!("Could not decode transition audio, using beep: {error}");
                            beep()
                        }
                    }
                }
            };
            let output = self.output()?;
            let player = Player::connect_new(output.sink.mixer());
            player.append(source);
            output.alerts.push(player);
            Ok(())
        }

        fn maintain(&mut self) -> bool {
            if let Some(output) = &mut self.output {
                if output.failed.load(Ordering::Relaxed) {
                    eprintln!("Audio output failed; playback will retry on the next activation");
                    self.close();
                    return false;
                }
                output.alerts.retain(|player| !player.empty());
                if output.ticking.is_some() || !output.alerts.is_empty() {
                    return true;
                }
            }
            self.close();
            false
        }

        fn close(&mut self) {
            self.output = None;
        }
    }

    fn volume(value: f32, default: f32) -> f32 {
        if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            default
        }
    }

    fn beep() -> AudioSource {
        Box::new(
            rodio::source::SineWave::new(800.0)
                .take_duration(Duration::from_millis(300))
                .amplify(0.3),
        )
    }

    fn nonempty(source: impl Source<Item = f32> + Send + 'static) -> AudioResult<AudioSource> {
        let source = source.buffered();
        if source.clone().next().is_none() {
            return Err("Audio file contains no samples".into());
        }
        Ok(Box::new(source))
    }

    fn load_source(custom: Option<&str>, embedded: &'static [u8]) -> AudioResult<AudioSource> {
        if let Some(path) = custom {
            let source = (|| {
                let file = std::fs::File::open(path)?;
                nonempty(Decoder::try_from(file)?)
            })();
            match source {
                Ok(source) => return Ok(source),
                Err(error) => {
                    eprintln!(
                        "Could not load custom sound '{path}', using embedded sound: {error}"
                    );
                }
            }
        }
        nonempty(Decoder::new(Cursor::new(embedded))?)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::{Mutex, mpsc};

        #[derive(Default)]
        struct FakeBackend {
            events: Arc<Mutex<Vec<&'static str>>>,
            fail_start: bool,
        }

        impl Backend for FakeBackend {
            fn start_ticking(&mut self, _: &TickingConfig) -> AudioResult<()> {
                self.events.lock().unwrap().push("start");
                if self.fail_start {
                    return Err("no output device".into());
                }
                Ok(())
            }
            fn stop_ticking(&mut self) {
                self.events.lock().unwrap().push("stop");
            }
            fn play_transition(&mut self, _: &SoundConfig, _: SoundType) -> AudioResult<()> {
                self.events.lock().unwrap().push("alert");
                Ok(())
            }
            fn maintain(&mut self) -> bool {
                false
            }
            fn close(&mut self) {
                self.events.lock().unwrap().push("close");
            }
        }

        fn run_commands(
            config: SoundConfig,
            commands: Vec<Command>,
            fail_start: bool,
        ) -> Vec<&'static str> {
            let backend = FakeBackend {
                fail_start,
                ..FakeBackend::default()
            };
            let events = backend.events.clone();
            let (sender, receiver) = mpsc::channel();
            for command in commands {
                sender.send(command).unwrap();
            }
            drop(sender);
            run(receiver, config, backend);
            events.lock().unwrap().clone()
        }

        fn ticking_config() -> SoundConfig {
            SoundConfig {
                ticking: TickingConfig {
                    enabled: true,
                    ..TickingConfig::default()
                },
                ..SoundConfig::default()
            }
        }

        #[test]
        fn worker_stops_loop_before_alert_and_deduplicates_requests() {
            let events = run_commands(
                ticking_config(),
                vec![
                    Command::Ticking(true),
                    Command::Ticking(true),
                    Command::Transition(SoundType::WorkToBreak),
                    Command::Ticking(false),
                    Command::Transition(SoundType::BreakToWork),
                    Command::Ticking(true),
                    Command::Ticking(false),
                    Command::Ticking(true),
                    Command::Shutdown,
                    Command::Ticking(true),
                ],
                false,
            );
            assert_eq!(
                events,
                [
                    "start", "stop", "alert", "stop", "alert", "start", "stop", "start", "close"
                ]
            );
        }

        #[test]
        fn disabled_ticking_never_starts_and_alert_mode_is_independent() {
            assert_eq!(
                run_commands(SoundConfig::default(), vec![Command::Ticking(true)], false),
                ["close"]
            );
            for mode in [SoundMode::None, SoundMode::SystemBeep] {
                let mut config = ticking_config();
                config.mode = Some(mode.clone());
                let events = run_commands(
                    config,
                    vec![
                        Command::Ticking(true),
                        Command::Transition(SoundType::WorkToBreak),
                    ],
                    false,
                );
                if mode == SoundMode::None {
                    assert_eq!(events, ["start", "stop", "close"]);
                } else {
                    assert_eq!(events, ["start", "stop", "alert", "close"]);
                }
            }
        }

        #[test]
        fn output_failure_retries_only_after_a_new_activation() {
            assert_eq!(
                run_commands(
                    ticking_config(),
                    vec![
                        Command::Ticking(true),
                        Command::Ticking(true),
                        Command::Ticking(false),
                        Command::Ticking(true),
                    ],
                    true
                ),
                ["start", "stop", "start", "close"]
            );
        }

        #[test]
        fn controller_drop_stops_and_joins_worker() {
            let backend = FakeBackend::default();
            let events = backend.events.clone();
            let (sender, receiver) = mpsc::channel();
            let worker = std::thread::spawn(move || run(receiver, ticking_config(), backend));
            let controller = super::super::AudioController {
                sender: Some(sender),
                worker: Some(worker),
            };
            controller.set_ticking(true);
            drop(controller);
            assert_eq!(*events.lock().unwrap(), ["start", "close"]);
        }

        #[test]
        fn worker_releases_finished_playback_without_another_command() {
            struct CompletingBackend {
                completed: Arc<AtomicBool>,
                events: mpsc::Sender<&'static str>,
                active: bool,
            }
            impl Backend for CompletingBackend {
                fn start_ticking(&mut self, _: &TickingConfig) -> AudioResult<()> {
                    unreachable!()
                }
                fn stop_ticking(&mut self) {}
                fn play_transition(&mut self, _: &SoundConfig, _: SoundType) -> AudioResult<()> {
                    self.active = true;
                    self.events.send("playing").unwrap();
                    Ok(())
                }
                fn maintain(&mut self) -> bool {
                    if self.active && self.completed.load(Ordering::Relaxed) {
                        self.active = false;
                        self.events.send("released").unwrap();
                    }
                    self.active
                }
                fn close(&mut self) {}
            }
            let completed = Arc::new(AtomicBool::new(false));
            let (events, observed) = mpsc::channel();
            let backend = CompletingBackend {
                completed: completed.clone(),
                events,
                active: false,
            };
            let (sender, receiver) = mpsc::channel();
            let worker = std::thread::spawn(move || run(receiver, SoundConfig::default(), backend));
            sender
                .send(Command::Transition(SoundType::WorkToBreak))
                .unwrap();
            assert_eq!(
                observed.recv_timeout(Duration::from_secs(2)).unwrap(),
                "playing"
            );
            assert!(observed.try_recv().is_err());
            completed.store(true, Ordering::Relaxed);
            assert_eq!(
                observed.recv_timeout(Duration::from_secs(2)).unwrap(),
                "released"
            );
            drop(sender);
            worker.join().unwrap();
        }

        #[test]
        fn embedded_loop_repeats_exactly_without_an_output_device() {
            let source = load_source(None, TICKING).unwrap();
            assert_eq!(source.channels().get(), 1);
            assert_eq!(source.sample_rate().get(), 22_050);
            let samples: Vec<_> = source.collect();
            assert_eq!(samples.len(), 8 * 22_050);
            assert!(samples.iter().any(|sample| sample.abs() > 0.01));
            let repeated: Vec<_> = load_source(None, TICKING)
                .unwrap()
                .repeat_infinite()
                .take(samples.len() * 2)
                .collect();
            assert_eq!(&repeated[..samples.len()], samples);
            assert_eq!(&repeated[samples.len()..], samples);
        }

        #[test]
        fn invalid_custom_files_fall_back_and_valid_files_override() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("custom.wav");
            for content in [None, Some(b"invalid audio".as_slice()), Some(&[])] {
                if let Some(content) = content {
                    std::fs::write(&path, content).unwrap();
                }
                let fallback: Vec<_> = load_source(path.to_str(), TICKING).unwrap().collect();
                assert_eq!(
                    fallback,
                    load_source(None, TICKING).unwrap().collect::<Vec<_>>()
                );
            }
            std::fs::write(&path, WORK_TO_BREAK).unwrap();
            let custom: Vec<_> = load_source(path.to_str(), TICKING).unwrap().collect();
            assert_eq!(
                custom,
                load_source(None, WORK_TO_BREAK)
                    .unwrap()
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn alerts_longer_than_five_seconds_play_to_completion() {
            let (player, mut output) = Player::new();
            player
                .append(rodio::source::SineWave::new(800.0).take_duration(Duration::from_secs(6)));
            let mut samples = 0;
            while !player.empty() && samples < 48_000 * 7 {
                output.next().unwrap();
                samples += 1;
            }
            assert!(player.empty());
            assert!(samples >= 48_000 * 6);
        }

        #[test]
        fn embedded_alerts_decode() {
            for data in [WORK_TO_BREAK, BREAK_TO_WORK, WORK_TO_LONG_BREAK] {
                assert!(load_source(None, data).unwrap().next().is_some());
            }
        }
    }
}
