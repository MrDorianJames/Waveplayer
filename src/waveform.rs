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
use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Debug, Clone)]
pub struct WaveformData {
    pub path: PathBuf,
    pub peaks: Vec<f32>,
    pub rms: Vec<f32>,
    pub duration_secs: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: Option<u32>,
    pub bitrate_kbps: Option<u32>,
    pub codec: String,
    pub file_size: u64,
    pub low_energy: Vec<f32>,
    pub mid_energy: Vec<f32>,
    pub high_energy: Vec<f32>,
}

impl WaveformData {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        const NUM_BUCKETS: usize = 2048;
        const FFT_SIZE: usize = 1024;

        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
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
        let channels = track.codec_params.channels
        .map(|c| c.count() as u16)
        .unwrap_or(2);
        let bit_depth = track.codec_params.bits_per_sample;
        let codec_str = format!("{:?}", track.codec_params.codec)
        .replace("CODEC_TYPE_", "")
        .to_uppercase();

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
            let ch = spec.channels.count();
            for frame in sample_buf.samples().chunks(ch) {
                all_samples.push(frame.iter().sum::<f32>() / ch as f32);
            }
        }

        if all_samples.is_empty() {
            return Err("No audio samples decoded".to_string());
        }

        let total = all_samples.len();
        let duration_secs = total as f32 / sample_rate as f32;

        let bitrate_kbps = if duration_secs > 0.0 && file_size > 0 {
            Some(((file_size * 8) as f32 / duration_secs / 1000.0) as u32)
        } else {
            None
        };

        let bucket_size = (total / NUM_BUCKETS).max(1);
        let mut peaks = Vec::with_capacity(NUM_BUCKETS);
        let mut rms_vals = Vec::with_capacity(NUM_BUCKETS);
        let mut low_energy = Vec::with_capacity(NUM_BUCKETS);
        let mut mid_energy = Vec::with_capacity(NUM_BUCKETS);
        let mut high_energy = Vec::with_capacity(NUM_BUCKETS);

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let hz_per_bin = sample_rate as f32 / FFT_SIZE as f32;
        let low_end = (250.0 / hz_per_bin) as usize;
        let mid_end = (4000.0 / hz_per_bin) as usize;
        let high_end = (FFT_SIZE / 2).min((20000.0 / hz_per_bin) as usize);

        for chunk in all_samples.chunks(bucket_size).take(NUM_BUCKETS) {
            peaks.push(chunk.iter().map(|s| s.abs()).fold(0.0f32, f32::max));
            rms_vals.push((chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt());

            let mut fft_input: Vec<Complex<f32>> = (0..FFT_SIZE)
            .map(|i| {
                let sample = if i < chunk.len() { chunk[i] } else { 0.0 };
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32
                / (FFT_SIZE - 1) as f32).cos());
                Complex::new(sample * window, 0.0)
            })
            .collect();

            fft.process(&mut fft_input);

            let mag: Vec<f32> = fft_input[..FFT_SIZE / 2]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

            let low = mag[1..low_end.max(2)].iter().sum::<f32>() / (low_end as f32).max(1.0);
            let mid = mag[low_end..mid_end.min(mag.len())].iter().sum::<f32>()
            / ((mid_end - low_end) as f32).max(1.0);
            let high = mag[mid_end..high_end.min(mag.len())].iter().sum::<f32>()
            / ((high_end - mid_end) as f32).max(1.0);

            low_energy.push(low);
            mid_energy.push(mid);
            high_energy.push(high);
        }

        while peaks.len() < NUM_BUCKETS {
            peaks.push(0.0);
            rms_vals.push(0.0);
            low_energy.push(0.0);
            mid_energy.push(0.0);
            high_energy.push(0.0);
        }

        let max_peak = peaks.iter().cloned().fold(0.0f32, f32::max);
        if max_peak > 0.0 {
            for p in &mut peaks { *p /= max_peak; }
            for r in &mut rms_vals { *r /= max_peak; }
        }

        let norm = |v: &mut Vec<f32>| {
            let max = v.iter().cloned().fold(0.0f32, f32::max);
            if max > 0.0 { for x in v.iter_mut() { *x /= max; } }
        };
        norm(&mut low_energy);
        norm(&mut mid_energy);
        norm(&mut high_energy);

        Ok(WaveformData {
            path: path.to_path_buf(),
           peaks,
           rms: rms_vals,
           duration_secs,
           sample_rate,
           channels,
           bit_depth,
           bitrate_kbps,
           codec: codec_str,
           file_size,
           low_energy,
           mid_energy,
           high_energy,
        })
    }
}
