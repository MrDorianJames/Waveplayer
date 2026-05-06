use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

fn socket_path() -> String {
    format!("/tmp/waveplayer-{}.sock", std::process::id())
}

fn find_existing_socket() -> Option<String> {
    if let Ok(entries) = std::fs::read_dir("/tmp") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("waveplayer-") && name.ends_with(".sock") {
                let path = entry.path().to_string_lossy().to_string();
                if UnixStream::connect(&path).is_ok() {
                    return Some(path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
    None
}

pub fn acquire_or_send(path: Option<&PathBuf>) -> Option<IpcReceiver> {
    if let Some(socket) = find_existing_socket() {
        if let Ok(mut stream) = UnixStream::connect(&socket) {
            let msg = path
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();
            let _ = stream.write_all(msg.as_bytes());
            return None;
        }
    }

    let my_socket = socket_path();
    let _ = std::fs::remove_file(&my_socket);
    let listener = UnixListener::bind(&my_socket)
    .expect("Could not bind IPC socket");
    listener.set_nonblocking(true)
    .expect("Could not set non-blocking");

    let socket_clone = my_socket.clone();
    std::panic::set_hook(Box::new(move |_| {
        let _ = std::fs::remove_file(&socket_clone);
    }));

    Some(IpcReceiver {
        listener,
         cli_path: path.cloned(),
    })
}

pub struct IpcReceiver {
    listener: UnixListener,
    cli_path: Option<PathBuf>,
}

impl Drop for IpcReceiver {
    fn drop(&mut self) {
        let path = socket_path();
        let _ = std::fs::remove_file(&path);
    }
}

impl IpcReceiver {
    pub fn try_recv(&self) -> Option<PathBuf> {
        match self.listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_read_timeout(
                    Some(std::time::Duration::from_millis(100))
                );
                let mut buf = String::new();
                let _ = stream.read_to_string(&mut buf);
                let trimmed = buf.trim().to_string();
                if trimmed.is_empty() {
                    return None;
                }
                let path = PathBuf::from(&trimmed);
                if let Some(ref cli) = self.cli_path {
                    if &path == cli {
                        return None;
                    }
                }
                Some(path)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }
}
