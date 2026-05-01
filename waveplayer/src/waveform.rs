use std::path::{Path, PathBuf};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::DecoderOptions,
    errors::Error,
    formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
};

#[derive(Debug, Clone)]
pub struct WaveformData {
    pub path: PathBuf,
    pub peaks: Vec<f32>,
    pub rms: Vec<f32>,
    pub duration_secs: f32,
    pub sample_rate: u32,
}

impl WaveformData {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        const NUM_BUCKETS: usize = 2048;

        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("Probe failed: {e}"))?;
        let mut format = probed.format;
        let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Codec error: {e}"))?;
        let track_id = track.id;
        let mut all_samples: Vec<f32> = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(Error::IoError(_)) | Err(Error::ResetRequired) => break,
                Err(e) => { eprintln!("Packet error: {e}"); break; }
            };
            if packet.track_id() != track_id { continue; }
            let decoded = match decoder.decode(&packet) {
                Ok(d) => d,
                Err(e) => { eprintln!("Decode error: {e}"); continue; }
            };
            let spec = *decoded.spec();
            let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
            sample_buf.copy_interleaved_ref(decoded);
            let channels = spec.channels.count();
            for frame in sample_buf.samples().chunks(channels) {
                all_samples.push(frame.iter().sum::<f32>() / channels as f32);
            }
        }

        if all_samples.is_empty() {
            return Err("No audio samples decoded".to_string());
        }

        let total = all_samples.len();
        let duration_secs = total as f32 / sample_rate as f32;
        let bucket_size = (total / NUM_BUCKETS).max(1);
        let mut peaks = Vec::with_capacity(NUM_BUCKETS);
        let mut rms_vals = Vec::with_capacity(NUM_BUCKETS);

        for chunk in all_samples.chunks(bucket_size).take(NUM_BUCKETS) {
            peaks.push(chunk.iter().map(|s| s.abs()).fold(0.0f32, f32::max));
            rms_vals.push((chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt());
        }

        while peaks.len() < NUM_BUCKETS { peaks.push(0.0); rms_vals.push(0.0); }

        let max_peak = peaks.iter().cloned().fold(0.0f32, f32::max);
        if max_peak > 0.0 {
            for p in &mut peaks { *p /= max_peak; }
            for r in &mut rms_vals { *r /= max_peak; }
        }

        Ok(WaveformData {
            path: path.to_path_buf(),
           peaks,
           rms: rms_vals,
           duration_secs,
           sample_rate,
        })
    }
}
