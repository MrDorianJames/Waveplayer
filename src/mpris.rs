use mpris_server::{
    Metadata, PlaybackStatus, Player, Time, TrackId,
};
use std::sync::mpsc as std_mpsc;

/// Commands sent FROM MPRIS controls TO the iced app
#[derive(Debug, Clone)]
pub enum MprisCommand {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
    Seek(f64),        // seconds offset
    SetPosition(f64), // absolute seconds
}

/// Updates sent FROM iced app TO the MPRIS server
#[derive(Debug)]
pub enum MprisUpdate {
    Metadata {
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        duration_secs: f64,
        is_playing: bool,
        artwork: Option<Vec<u8>>,
        track_number: u32,
    },
    PlaybackStatus(bool),
    Position(f64),
}

pub type MprisUpdateSender = std_mpsc::SyncSender<MprisUpdate>;
pub type MprisCommandReceiver = std_mpsc::Receiver<MprisCommand>;

/// Write artwork to a unique temp file per track and return the file:// URI.
/// Also cleans up the previous track's artwork file.
fn save_artwork_to_tmp(data: &[u8], track_number: u32) -> Option<String> {
    // Clean up old artwork files
    for i in 0..track_number {
        let old = std::env::temp_dir().join(format!("waveplayer-artwork-{}.jpg", i));
        let _ = std::fs::remove_file(old);
    }
    let path = std::env::temp_dir().join(format!("waveplayer-artwork-{}.jpg", track_number));
    std::fs::write(&path, data).ok()?;
    Some(format!("file://{}", path.to_string_lossy()))
}

pub fn spawn() -> (MprisUpdateSender, MprisCommandReceiver) {
    let (update_tx, update_rx) = std_mpsc::sync_channel::<MprisUpdate>(32);
    let (cmd_tx, cmd_rx) = std_mpsc::sync_channel::<MprisCommand>(32);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("MPRIS tokio runtime");

        let local = tokio::task::LocalSet::new();

        local.block_on(&rt, async move {
            let player = match Player::builder("waveplayer")
                .identity("WavePlayer")
                .can_play(true)
                .can_pause(true)
                .can_go_next(true)
                .can_go_previous(true)
                .can_seek(true)
                .can_control(true)
                .build()
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("MPRIS server failed to start: {e}");
                    return;
                }
            };

            let tx = cmd_tx.clone();
            player.connect_play(move |_| { let _ = tx.send(MprisCommand::Play); });

            let tx = cmd_tx.clone();
            player.connect_pause(move |_| { let _ = tx.send(MprisCommand::Pause); });

            let tx = cmd_tx.clone();
            player.connect_play_pause(move |_| { let _ = tx.send(MprisCommand::PlayPause); });

            let tx = cmd_tx.clone();
            player.connect_stop(move |_| { let _ = tx.send(MprisCommand::Stop); });

            let tx = cmd_tx.clone();
            player.connect_next(move |_| { let _ = tx.send(MprisCommand::Next); });

            let tx = cmd_tx.clone();
            player.connect_previous(move |_| { let _ = tx.send(MprisCommand::Previous); });

            let tx = cmd_tx.clone();
            player.connect_seek(move |_, offset| {
                let secs = offset.as_micros() as f64 / 1_000_000.0;
                let _ = tx.send(MprisCommand::Seek(secs));
            });

            let tx = cmd_tx.clone();
            player.connect_set_position(move |_, _track_id, pos| {
                let secs = pos.as_micros() as f64 / 1_000_000.0;
                let _ = tx.send(MprisCommand::SetPosition(secs));
            });

            tokio::task::spawn_local(player.run());

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                while let Ok(update) = update_rx.try_recv() {
                    match update {
                        MprisUpdate::Metadata {
                            title, artist, album,
                            duration_secs, is_playing,
                            artwork, track_number,
                        } => {
                            let track_id = TrackId::try_from(
                                format!("/org/waveplayer/track/{}", track_number).as_str()
                            ).unwrap_or(TrackId::NO_TRACK);

                            let mut meta = Metadata::builder()
                                .trackid(track_id);

                            if let Some(t) = &title { meta = meta.title(t.as_str()); }
                            if let Some(a) = &artist { meta = meta.artist([a.as_str()]); }
                            if let Some(al) = &album { meta = meta.album(al.as_str()); }
                            if duration_secs > 0.0 {
                                meta = meta.length(Time::from_micros(
                                    (duration_secs * 1_000_000.0) as i64
                                ));
                            }
                            // Use track_number in filename so KDE sees a new URI
                            if let Some(ref art_data) = artwork {
                                if let Some(uri) = save_artwork_to_tmp(art_data, track_number) {
                                    meta = meta.art_url(uri.as_str());
                                }
                            }

                            let _ = player.set_metadata(meta.build()).await;

                            let status = if is_playing {
                                PlaybackStatus::Playing
                            } else {
                                PlaybackStatus::Paused
                            };
                            let _ = player.set_playback_status(status).await;
                        }
                        MprisUpdate::PlaybackStatus(is_playing) => {
                            let status = if is_playing {
                                PlaybackStatus::Playing
                            } else {
                                PlaybackStatus::Paused
                            };
                            let _ = player.set_playback_status(status).await;
                        }
                        MprisUpdate::Position(secs) => {
                            player.set_position(
                                Time::from_micros((secs * 1_000_000.0) as i64)
                            );
                        }
                    }
                }
            }
        });
    });

    (update_tx, cmd_rx)
}
