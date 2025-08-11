use std::time::Duration;

use rodio::{source::SeekError, OutputStream, Sink, Source};

use porter_audio::Audio;

pub struct AudioPlayer {
    pub stream: OutputStream,
    pub sink: Sink,
    pub total_duration: Option<Duration>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        // Note: The stream must live as long as the sink
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Otherwise your ears will explode lol
        sink.set_volume(0.1);

        Self {
            stream,
            sink,
            total_duration: None,
        }
    }

    pub fn set_preview(&mut self, _name: String, audio: Audio) {
        self.play_new(audio);
    }

    pub fn play_new(&mut self, audio: Audio) {
        // Clear the old ones
        self.sink.clear();

        // Load the audio
        let samples = unsafe { audio.samples.align_to::<i16>().1.to_vec() };

        let source = rodio::buffer::SamplesBuffer::new(audio.channel_count, audio.frame_rate, samples);

        self.total_duration = source.total_duration();

        // Play the new one
        self.sink.append(source);
        self.play();
    }

    pub fn pos(&self) -> Option<f64> {
        if let Some(total_duration) = self.total_duration {
            let pos = Self::duration_div(self.sink.get_pos(), total_duration) * 100.0;
            Some(pos)
        } else {
            None
        }
    }

    pub fn current_time(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    pub fn seek(&self, percent: f32) -> Result<(), SeekError> {
        if let Some(total_duration) = self.total_duration {
            let pos = Self::duration_mul_f32(total_duration, percent);
            self.sink.try_seek(pos)
        } else {
            Err(SeekError::NotSupported { underlying_source: "Failed to get audio duration to seek" })
        }
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub fn has_audio(&self) -> bool {
        self.sink.len() > 0
    }

    pub fn duration_mul_f32(duration: Duration, value: f32) -> Duration {
        let seconds = duration.as_secs_f64() * value as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn duration_div(dur1: Duration, dur2: Duration) -> f64 {
        dur1.as_secs_f64() / dur2.as_secs_f64()
    }

    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }
}
