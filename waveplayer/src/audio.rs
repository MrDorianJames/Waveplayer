use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    formats::{FormatOptions, SeekMode, SeekTo},
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
        units::Time,
};
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};

struct Inner {
    position_secs: f64,
    is_playing: bool,
    seek_target: Option<f64>,
    ended: bool,
    end_notified: bool,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            position_secs: 0.0,
            is_playing: false,
            seek_target: None,
            ended: false,
            end_notified: false,
        }
    }
}

pub struct AudioEngine {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    inner: Arc<Mutex<Inner>>,
    sink: Arc<Mutex<Option<Sink>>>,
    current_path: Arc<Mutex<Option<PathBuf>>>,
    volume: Arc<Mutex<f32>>,
    generation: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default()
        .expect("Could not open audio output stream");
        Self {
            _stream: stream,
            _handle: handle,
            inner: Arc::new(Mutex::new(Inner::default())),
            sink: Arc::new(Mutex::new(None)),
            current_path: Arc::new(Mutex::new(None)),
            volume: Arc::new(Mutex::new(0.8)),
            generation: Arc::new(AtomicU64::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn load(&self, path: &Path) {
        self.stop_flag.store(true, Ordering::SeqCst);
        {
            let mut sl = self.sink.lock().unwrap();
            if let Some(ref s) = *sl { s.stop(); }
            *sl = None;
        }
        *self.current_path.lock().unwrap() = Some(path.to_path_buf());
        {
            let mut inn = self.inner.lock().unwrap();
            inn.position_secs = 0.0;
            inn.is_playing = false;
            inn.seek_target = None;
            inn.ended = false;
            inn.end_notified = false;
        }
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.stop_flag.store(false, Ordering::SeqCst);
        self.start_thread(path, 0.0, gen);
    }

    pub fn play(&self) {
        let mut inn = self.inner.lock().unwrap();
        inn.is_playing = true;
        drop(inn);
        let mut attempts = 0;
        loop {
            let sl = self.sink.lock().unwrap();
            if sl.is_some() || attempts > 50 {
                if let Some(ref s) = *sl { s.play(); }
                break;
            }
            drop(sl);
            attempts += 1;
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        {
            let mut sl = self.sink.lock().unwrap();
            if let Some(ref s) = *sl { s.stop(); }
            *sl = None;
        }
        let mut inn = self.inner.lock().unwrap();
        inn.is_playing = false;
        inn.position_secs = 0.0;
        inn.ended = false;
        inn.end_notified = false;
    }

    pub fn rewind(&self) {
        let mut inn = self.inner.lock().unwrap();
        inn.ended = false;
        inn.end_notified = false;
        inn.is_playing = false;
        inn.position_secs = 0.0;
    }

    fn restart(&self, start_secs: f64, play: bool) {
        self.stop_flag.store(true, Ordering::SeqCst);
        {
            let mut sl = self.sink.lock().unwrap();
            if let Some(ref s) = *sl { s.stop(); }
            *sl = None;
        }
        {
            let mut inn = self.inner.lock().unwrap();
            inn.ended = false;
            inn.end_notified = false;
            inn.seek_target = None;
            inn.position_secs = start_secs;
            inn.is_playing = play;
        }
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.stop_flag.store(false, Ordering::SeqCst);
        let path = self.current_path.lock().unwrap().clone();
        if let Some(p) = path {
            self.start_thread(&p, start_secs, gen);
        }
    }

    fn start_thread(&self, path: &Path, start_secs: f64, gen: u64) {
        let path = path.to_path_buf();
        let inner = Arc::clone(&self.inner);
        let sink_arc = Arc::clone(&self.sink);
        let volume = *self.volume.lock().unwrap();
        let handle = self._handle.clone();
        let generation = Arc::clone(&self.generation);
        let stop_flag = Arc::clone(&self.stop_flag);

        thread::spawn(move || {
            let is_current = || {
                generation.load(Ordering::SeqCst) == gen
                && !stop_flag.load(Ordering::SeqCst)
            };

            let file = match std::fs::File::open(&path) {
                Ok(f) => f,
                      Err(e) => { eprintln!("open error: {e}"); return; }
            };
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let mut hint = Hint::new();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                hint.with_extension(ext);
            }
            let probed = match symphonia::default::get_probe().format(
                &hint, mss,
                &FormatOptions { enable_gapless: true, ..Default::default() },
                                                                      &MetadataOptions::default(),
            ) {
                Ok(p) => p,
                      Err(e) => { eprintln!("probe error: {e}"); return; }
            };

            if !is_current() { return; }

            let mut format = probed.format;
            let track = match format.tracks().iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            {
                Some(t) => t.clone(),
                      None => { eprintln!("no track"); return; }
            };
            let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
            let channels = track.codec_params.channels
            .map(|c| c.count()).unwrap_or(2) as u16;
            let track_id = track.id;
            let mut decoder = match symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            {
                Ok(d) => d,
                      Err(e) => { eprintln!("codec error: {e}"); return; }
            };

            let mut position_secs = start_secs;
            if start_secs > 0.0 {
                if let Ok(seeked) = format.seek(
                    SeekMode::Accurate,
                    SeekTo::Time {
                        time: Time::from(start_secs),
                                                track_id: Some(track_id),
                    },
                ) {
                    position_secs = seeked.actual_ts as f64 / sample_rate as f64;
                }
                decoder.reset();
            }

            if !is_current() { return; }

            let sink = match Sink::try_new(&handle) {
                Ok(s) => s,
                      Err(e) => { eprintln!("sink error: {e}"); return; }
            };
            sink.set_volume(volume);
            sink.pause();
            *sink_arc.lock().unwrap() = Some(sink);

            loop {
                if !is_current() { break; }

                let seek_target = inner.lock().unwrap().seek_target.take();
                if let Some(target) = seek_target {
                    {
                        let mut sl = sink_arc.lock().unwrap();
                        if let Some(ref s) = *sl { s.stop(); }
                        *sl = None;
                    }
                    if !is_current() { break; }

                    let new_sink = match Sink::try_new(&handle) {
                        Ok(s) => s,
                      Err(_) => break,
                    };
                    new_sink.set_volume(volume);
                    new_sink.pause();
                    *sink_arc.lock().unwrap() = Some(new_sink);

                    match format.seek(
                        SeekMode::Accurate,
                        SeekTo::Time {
                            time: Time::from(target),
                                      track_id: Some(track_id),
                        },
                    ) {
                        Ok(seeked) => {
                            position_secs = seeked.actual_ts as f64 / sample_rate as f64;
                        }
                        Err(_) => { position_secs = target; }
                    }
                    decoder.reset();
                    inner.lock().unwrap().position_secs = position_secs;
                    continue;
                }

                let is_playing = inner.lock().unwrap().is_playing;
                {
                    let sl = sink_arc.lock().unwrap();
                    if let Some(ref s) = *sl {
                        if is_playing && s.is_paused() { s.play(); }
                        else if !is_playing && !s.is_paused() { s.pause(); }
                    }
                }

                if !is_playing {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }

                loop {
                    if !is_current() { break; }
                    if inner.lock().unwrap().seek_target.is_some() { break; }
                    let len = sink_arc.lock().unwrap()
                    .as_ref().map(|s| s.len()).unwrap_or(0);
                    if len < 8 { break; }
                    thread::sleep(Duration::from_millis(1));
                }

                if !is_current() { break; }
                if inner.lock().unwrap().seek_target.is_some() { continue; }

                let packet = match format.next_packet() {
                    Ok(p) => p,
                      Err(_) => {
                          // Wait for sink to finish playing before marking ended
                          loop {
                              if !is_current() { break; }
                              let len = sink_arc.lock().unwrap()
                              .as_ref().map(|s| s.len()).unwrap_or(0);
                              if len == 0 { break; }
                              thread::sleep(Duration::from_millis(5));
                          }
                          if generation.load(Ordering::SeqCst) == gen
                              && !stop_flag.load(Ordering::SeqCst)
                              {
                                  {
                                      let mut sl = sink_arc.lock().unwrap();
                                      if let Some(ref s) = *sl { s.stop(); }
                                      *sl = None;
                                  }
                                  if generation.load(Ordering::SeqCst) == gen
                                      && !stop_flag.load(Ordering::SeqCst)
                                      {
                                          let mut inn = inner.lock().unwrap();
                                          inn.is_playing = false;
                                          inn.ended = true;
                                          inn.end_notified = false;
                                          inn.seek_target = Some(0.0);
                                          inn.position_secs = 0.0;
                                      }
                              }
                              continue;
                      }
                };

                if packet.track_id() != track_id { continue; }

                let decoded = match decoder.decode(&packet) {
                    Ok(d) => d,
                      Err(_) => continue,
                };

                let mut samples: Vec<f32> = Vec::new();
                let frames = match &decoded {
                    AudioBufferRef::F32(buf) => {
                        let f = buf.frames();
                        for i in 0..f {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[i]);
                            }
                        }
                        f
                    }
                    AudioBufferRef::S16(buf) => {
                        let f = buf.frames();
                        for i in 0..f {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[i] as f32 / i16::MAX as f32);
                            }
                        }
                        f
                    }
                    AudioBufferRef::S32(buf) => {
                        let f = buf.frames();
                        for i in 0..f {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[i] as f32 / i32::MAX as f32);
                            }
                        }
                        f
                    }
                    AudioBufferRef::U8(buf) => {
                        let f = buf.frames();
                        for i in 0..f {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push((buf.chan(ch)[i] as f32 - 128.0) / 128.0);
                            }
                        }
                        f
                    }
                    AudioBufferRef::S24(buf) => {
                        let f = buf.frames();
                        for i in 0..f {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[i].inner() as f32 / 8388607.0);
                            }
                        }
                        f
                    }
                    _ => continue,
                };

                if samples.is_empty() { continue; }

                position_secs += frames as f64 / sample_rate as f64;
                inner.lock().unwrap().position_secs = position_secs;

                let buf = SamplesBuffer::new(channels, sample_rate, samples);
                let sl = sink_arc.lock().unwrap();
                if let Some(ref s) = *sl { s.append(buf); }
            }

            let mut sl = sink_arc.lock().unwrap();
            if let Some(ref s) = *sl { s.stop(); }
            *sl = None;
        });
    }

    pub fn toggle_play_pause(&self) {
        let inn = self.inner.lock().unwrap();
        let ended = inn.ended;
        let is_playing = inn.is_playing;
        drop(inn);

        if ended {
            let mut inn = self.inner.lock().unwrap();
            inn.ended = false;
            inn.end_notified = false;
            inn.is_playing = true;
            drop(inn);
            let mut attempts = 0;
            loop {
                let sl = self.sink.lock().unwrap();
                if sl.is_some() || attempts > 50 {
                    if let Some(ref s) = *sl { s.play(); }
                    break;
                }
                drop(sl);
                attempts += 1;
                thread::sleep(Duration::from_millis(10));
            }
            return;
        }

        let mut inn = self.inner.lock().unwrap();
        inn.is_playing = !is_playing;
        let playing = inn.is_playing;
        drop(inn);

        let mut attempts = 0;
        loop {
            let sl = self.sink.lock().unwrap();
            if sl.is_some() || attempts > 50 {
                if let Some(ref s) = *sl {
                    if playing { s.play(); } else { s.pause(); }
                }
                break;
            }
            drop(sl);
            attempts += 1;
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn seek(&self, secs: f64) {
        let ended = self.inner.lock().unwrap().ended;
        if ended {
            self.restart(secs, false);
        } else {
            let mut inn = self.inner.lock().unwrap();
            inn.seek_target = Some(secs);
            inn.position_secs = secs;
        }
    }

    pub fn set_volume(&self, v: f32) {
        *self.volume.lock().unwrap() = v;
        let sl = self.sink.lock().unwrap();
        if let Some(ref s) = *sl { s.set_volume(v); }
    }

    pub fn is_playing(&self) -> bool {
        self.inner.lock().unwrap().is_playing
    }

    pub fn position_secs(&self) -> f64 {
        self.inner.lock().unwrap().position_secs
    }

    pub fn take_ended(&self) -> bool {
        let mut inn = self.inner.lock().unwrap();
        if inn.ended && !inn.end_notified {
            inn.end_notified = true;
            inn.is_playing = false;
            return true;
        }
        false
    }
}
