use rodio::{OutputStream, Sink, Source, source::SineWave};
use std::time::Duration;

pub struct SoundEngine {
    _stream: OutputStream,
    sink_left: Sink,
    sink_right: Sink,
}

impl SoundEngine {
    pub fn new() -> Option<Self> {
        let stream_handle = OutputStream::try_default().ok()?;
        // For spatial audio we use two sinks (left/right channels)
        // In practice rodio mixes them, so we'll use volume panning
        let sink = Sink::try_new(&stream_handle.1).ok()?;
        // We'll just use one sink and control volume via the source
        // For true stereo we'd need custom sample output
        Some(Self {
            _stream: stream_handle.0,
            sink_left: sink,
            sink_right: Sink::try_new(&stream_handle.1).ok()?,
        })
    }

    /// Play a sound with spatial positioning relative to the listener
    /// angle: radians from listener's forward direction
    /// distance: blocks away (affects volume)
    pub fn play_spatial(&self, freq: f32, duration_ms: u64, angle: f64, distance: f64) {
        // Volume falloff by distance (inverse square law, clamped)
        let vol = (1.0 / (1.0 + distance * distance * 0.1)) as f32;
        let vol = vol.clamp(0.01, 0.3);

        // Left/right panning based on angle
        // angle=0: directly ahead (equal both)
        // angle=PI/2: to the right (more right speaker)
        // angle=-PI/2: to the left (more left speaker)
        let pan = angle.sin(); // -1 to 1
        let left_vol = vol * ((1.0 - pan) * 0.5).max(0.1) as f32;
        let right_vol = vol * ((1.0 + pan) * 0.5).max(0.1) as f32;

        let source_left = SineWave::new(freq)
            .take_duration(Duration::from_millis(duration_ms))
            .amplify(left_vol);
        let source_right = SineWave::new(freq * 1.01) // slight detune for richness
            .take_duration(Duration::from_millis(duration_ms))
            .amplify(right_vol);

        self.sink_left.append(source_left);
        self.sink_right.append(source_right);
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
        self.sink_left.append(source);
    }

    pub fn play_place(&self) {
        let source = SineWave::new(440.0)
            .take_duration(Duration::from_millis(100))
            .amplify(0.15);
        self.sink_left.append(source);
    }

    pub fn play_break(&self) {
        let source = SineWave::new(180.0)
            .take_duration(Duration::from_millis(120))
            .amplify(0.15);
        self.sink_left.append(source);
    }

    /// Play a mob sound with spatial positioning
    pub fn play_mob_sound(
        &self,
        mob_x: f64, mob_z: f64,
        player_x: f64, player_z: f64,
        player_yaw: f64,
    ) {
        let dx = mob_x - player_x;
        let dz = mob_z - player_z;
        let distance = (dx * dx + dz * dz).sqrt();

        // Angle relative to player's facing direction
        let world_angle = dz.atan2(dx);
        let relative_angle = world_angle - player_yaw;

        self.play_spatial(280.0, 150, relative_angle, distance);
    }
}
