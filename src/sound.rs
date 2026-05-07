use rodio::{OutputStream, Sink, Source, source::SineWave};
use std::time::Duration;

pub struct SoundEngine {
    _stream: OutputStream,
    sink: Sink,
}

impl SoundEngine {
    pub fn new() -> Option<Self> {
        let stream_handle = OutputStream::try_default().ok()?;
        let sink = Sink::try_new(&stream_handle.1).ok()?;
        Some(Self {
            _stream: stream_handle.0,
            sink,
        })
    }

    pub fn play_step(&self, block_type: crate::block::BlockType) {
        let freq = match block_type {
            crate::block::BlockType::Wood => 200.0,
            crate::block::BlockType::Stone => 150.0,
            crate::block::BlockType::Sand => 300.0,
            crate::block::BlockType::Grass | crate::block::BlockType::Dirt => 250.0,
            _ => 220.0,
        };
        let source = SineWave::new(freq)
            .take_duration(Duration::from_millis(60))
            .amplify(0.1);
        self.sink.append(source);
    }

    pub fn play_place(&self) {
        let source = SineWave::new(440.0)
            .take_duration(Duration::from_millis(100))
            .amplify(0.15);
        self.sink.append(source);
    }

    pub fn play_break(&self) {
        let source = SineWave::new(180.0)
            .take_duration(Duration::from_millis(120))
            .amplify(0.15);
        self.sink.append(source);
    }
}
