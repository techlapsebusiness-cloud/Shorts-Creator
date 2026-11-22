//! Linux screen capture through xdg-desktop-portal ScreenCast.
//!
//! The portal hands back a PipeWire remote, not a device ffmpeg can open.
//! GStreamer `pipewiresrc` writes `screen.mov` from that fd. The `pipewire`
//! crate's DMA-BUF negotiation is the path the plan allows us to skip.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalCursor {
    Metadata,
    Embedded,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalSource {
    Monitor,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropEdges {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub struct PipelineInput<'a> {
    pub node_id: u32,
    pub fd: i32,
    pub location: &'a str,
    pub encoder: &'a str,
    pub crop: Option<CropEdges>,
}

pub fn choose_cursor_mode(metadata: bool, embedded: bool) -> PortalCursor {
    if metadata {
        PortalCursor::Metadata
    } else if embedded {
        PortalCursor::Embedded
    } else {
        PortalCursor::Hidden
    }
}

/// A window id asks for a window. Anything else asks for a monitor, which a
/// region can then crop. The portal dialog is what the user actually picks.
pub fn choose_source(
    want_window: bool,
    monitor_available: bool,
    window_available: bool,
) -> Result<PortalSource, String> {
    if want_window {
        if window_available {
            return Ok(PortalSource::Window);
        }
        return Err(
            "This portal cannot capture a window. Capture a monitor, or a region that does not overlap the prompter."
                .to_string(),
        );
    }
    if monitor_available {
        return Ok(PortalSource::Monitor);
    }
    if window_available {
        return Ok(PortalSource::Window);
    }
    Err("The portal did not advertise a monitor or window source.".to_string())
}

/// `region` is (x, y, width, height) in the same space as `frame` (width, height).
pub fn crop_edges(frame: (i32, i32), region: (i32, i32, i32, i32)) -> Option<CropEdges> {
    let (frame_w, frame_h) = frame;
    let (x, y, width, height) = region;
    if frame_w <= 0 || frame_h <= 0 || width <= 0 || height <= 0 {
        return None;
    }
    let left = x.max(0);
    let top = y.max(0);
    let right = frame_w.saturating_sub(x.saturating_add(width));
    let bottom = frame_h.saturating_sub(y.saturating_add(height));
    if left == 0 && top == 0 && right == 0 && bottom == 0 {
        return None;
    }
    if left.saturating_add(right) >= frame_w || top.saturating_add(bottom) >= frame_h {
        return None;
    }
    Some(CropEdges {
        left,
        top,
        right,
        bottom,
    })
}

pub fn gstreamer_pipeline(input: &PipelineInput<'_>) -> String {
    let crop = input.crop.map_or(String::new(), |crop| {
        format!(
            "videocrop left={} top={} right={} bottom={} ! ",
            crop.left, crop.top, crop.right, crop.bottom
        )
    });
    let location = quote_location(input.location);
    format!(
        "pipewiresrc fd={} path={} do-timestamp=true always-copy=true client-name=shorts-studio ! videoconvert ! {crop}video/x-raw,format=I420 ! {} ! mp4mux ! filesink location={location}",
        input.fd, input.node_id, input.encoder
    )
}

fn quote_location(path: &str) -> String {
    let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn bitrate_kbps(screen_bitrate: &str) -> u32 {
    let trimmed = screen_bitrate.trim();
    let (digits, suffix) = trimmed.split_at(trimmed.len().saturating_sub(1));
    let number: u32 = if suffix.eq_ignore_ascii_case("m") {
        digits.parse().unwrap_or(20)
    } else if suffix.eq_ignore_ascii_case("k") {
        return digits.parse().unwrap_or(20_000);
    } else {
        return trimmed.parse().unwrap_or(20_000);
    };
    number.saturating_mul(1000)
}

#[cfg(target_os = "linux")]
mod session {
    use super::{
        PipelineInput, PortalCursor, PortalSource, choose_cursor_mode, choose_source, crop_edges,
        gstreamer_pipeline,
    };
    use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
    use std::os::fd::AsRawFd;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    enum Boot {
        Ready(u32),
        Failed(String),
    }

    pub struct PortalRecording {
        pid: u32,
        stop: Arc<AtomicBool>,
        thread: Option<JoinHandle<()>>,
    }

    impl PortalRecording {
        pub fn process_id(&self) -> u32 {
            self.pid
        }

        pub fn start(
            path: &Path,
            want_window: bool,
            region: Option<(f64, f64, f64, f64)>,
            duration: Option<f64>,
            halt: Arc<AtomicBool>,
        ) -> Result<Self, String> {
            let gst = Command::new("gst-launch-1.0")
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            if !gst {
                return Err(
                    "gst-launch-1.0 is missing. Install gstreamer and gst-plugin-pipewire."
                        .to_string(),
                );
            }
            let path = path.to_path_buf();
            let encoder = h264_element().to_string();
            let (tx, rx) = mpsc::channel();
            let stop = Arc::new(AtomicBool::new(false));
            let stop_flag = Arc::clone(&stop);
            let thread = thread::spawn(move || {
                if let Err(message) = run_cast(
                    &path,
                    want_window,
                    region,
                    duration,
                    &encoder,
                    &stop_flag,
                    &halt,
                    tx.clone(),
                ) {
                    let _ = tx.send(Boot::Failed(message));
                }
            });
            match rx.recv() {
                Ok(Boot::Ready(pid)) => Ok(Self {
                    pid,
                    stop,
                    thread: Some(thread),
                }),
                Ok(Boot::Failed(message)) => {
                    let _ = thread.join();
                    Err(message)
                }
                Err(_) => {
                    let _ = thread.join();
                    Err("portal thread ended before capture started".to_string())
                }
            }
        }

        pub fn pause(&self) -> Result<(), String> {
            signal(self.pid, "-STOP")
        }

        pub fn resume(&self) -> Result<(), String> {
            signal(self.pid, "-CONT")
        }

        pub fn stop(mut self) -> Result<(), String> {
            self.finish()
        }

        fn finish(&mut self) -> Result<(), String> {
            self.stop.store(true, Ordering::SeqCst);
            if let Some(thread) = self.thread.take() {
                thread
                    .join()
                    .map_err(|_| "portal thread panicked".to_string())?;
            }
            Ok(())
        }
    }

    impl Drop for PortalRecording {
        fn drop(&mut self) {
            let _ = self.finish();
        }
    }

    fn run_cast(
        path: &Path,
        want_window: bool,
        region: Option<(f64, f64, f64, f64)>,
        duration: Option<f64>,
        encoder: &str,
        stop: &AtomicBool,
        halt: &AtomicBool,
        tx: mpsc::Sender<Boot>,
    ) -> Result<(), String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|error| format!("tokio: {error}"))?;
        runtime.block_on(async {
            record_session(path, want_window, region, duration, encoder, stop, halt, tx).await
        })
    }

    async fn record_session(
        path: &Path,
        want_window: bool,
        region: Option<(f64, f64, f64, f64)>,
        duration: Option<f64>,
        encoder: &str,
        stop: &AtomicBool,
        halt: &AtomicBool,
        tx: mpsc::Sender<Boot>,
    ) -> Result<(), String> {
        use ashpd::desktop::PersistMode;

        let proxy = Screencast::new()
            .await
            .map_err(|error| format!("open screencast portal: {error}"))?;
        let modes = proxy
            .available_cursor_modes()
            .await
            .unwrap_or_else(|_| CursorMode::Hidden.into());
        let cursor = choose_cursor_mode(
            modes.contains(CursorMode::Metadata),
            modes.contains(CursorMode::Embedded),
        );
        let sources = proxy
            .available_source_types()
            .await
            .unwrap_or_else(|_| SourceType::Monitor | SourceType::Window);
        let source = choose_source(
            want_window,
            sources.contains(SourceType::Monitor),
            sources.contains(SourceType::Window),
        )?;
        let source_flag = match source {
            PortalSource::Monitor => SourceType::Monitor.into(),
            PortalSource::Window => SourceType::Window.into(),
        };
        let cursor_flag = match cursor {
            PortalCursor::Metadata => CursorMode::Metadata,
            PortalCursor::Embedded => CursorMode::Embedded,
            PortalCursor::Hidden => CursorMode::Hidden,
        };
        let session = proxy
            .create_session()
            .await
            .map_err(|error| format!("create screencast session: {error}"))?;
        proxy
            .select_sources(
                &session,
                cursor_flag,
                source_flag,
                false,
                None,
                PersistMode::DoNot,
            )
            .await
            .map_err(|error| format!("select screencast source: {error}"))?;
        let started = proxy
            .start(&session, None)
            .await
            .map_err(|error| format!("start screencast: {error}"))?
            .response()
            .map_err(|error| format!("screencast dialog: {error}"))?;
        let stream = started
            .streams()
            .first()
            .ok_or_else(|| "portal returned no streams".to_string())?;
        let crop = match (region, stream.size()) {
            (Some((x, y, width, height)), Some((frame_w, frame_h))) => {
                let (origin_x, origin_y) = stream.position().unwrap_or((0, 0));
                crop_edges(
                    (frame_w, frame_h),
                    (
                        x as i32 - origin_x,
                        y as i32 - origin_y,
                        width as i32,
                        height as i32,
                    ),
                )
            }
            _ => None,
        };
        let node_id = stream.pipe_wire_node_id();
        let fd = proxy
            .open_pipe_wire_remote(&session)
            .await
            .map_err(|error| format!("open pipewire remote: {error}"))?;
        inherit_fd(fd.as_raw_fd())?;
        let location = path.display().to_string();
        let pipeline = gstreamer_pipeline(&PipelineInput {
            node_id,
            fd: fd.as_raw_fd(),
            location: &location,
            encoder,
            crop,
        });
        let mut child = Command::new("gst-launch-1.0")
            .args(["-e", &pipeline])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("spawn gst-launch-1.0: {error}"))?;
        let _ = tx.send(Boot::Ready(child.id()));
        let started_at = Instant::now();
        while !stop.load(Ordering::SeqCst) && !halt.load(Ordering::Relaxed) {
            if duration.is_some_and(|secs| started_at.elapsed().as_secs_f64() >= secs) {
                break;
            }
            if matches!(child.try_wait(), Ok(Some(_))) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(40)).await;
        }
        finish_gst(&mut child);
        drop(fd);
        let _ = session.close().await;
        Ok(())
    }

    fn finish_gst(child: &mut Child) {
        let _ = Command::new("kill")
            .args(["-CONT", &child.id().to_string()])
            .status();
        let _ = Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status();
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
    }

    fn inherit_fd(fd: std::os::fd::RawFd) -> Result<(), String> {
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags < 0 {
                return Err("read pipewire fd flags".to_string());
            }
            if libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                return Err("keep pipewire fd open for gstreamer".to_string());
            }
        }
        Ok(())
    }

    fn signal(pid: u32, flag: &str) -> Result<(), String> {
        let status = Command::new("kill")
            .args([flag, &pid.to_string()])
            .status()
            .map_err(|error| format!("kill {flag}: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("kill {flag} exited {status}"))
        }
    }

    fn h264_element() -> &'static str {
        for name in ["vah264enc", "vaapih264enc"] {
            let ok = Command::new("gst-inspect-1.0")
                .arg(name)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            if ok {
                return match name {
                    "vah264enc" => "vah264enc bitrate=20000",
                    _ => "vaapih264enc bitrate=20000",
                };
            }
        }
        "x264enc bitrate=20000 speed-preset=veryfast key-int-max=60"
    }
}

#[cfg(target_os = "linux")]
pub use session::PortalRecording;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_mode_prefers_metadata() {
        assert_eq!(choose_cursor_mode(true, true), PortalCursor::Metadata);
        assert_eq!(choose_cursor_mode(false, true), PortalCursor::Embedded);
        assert_eq!(choose_cursor_mode(false, false), PortalCursor::Hidden);
    }

    #[test]
    fn source_follows_the_request_and_refuses_a_missing_window() {
        assert_eq!(
            choose_source(false, true, true).unwrap(),
            PortalSource::Monitor
        );
        assert_eq!(
            choose_source(true, true, true).unwrap(),
            PortalSource::Window
        );
        assert!(choose_source(true, true, false).is_err());
        assert_eq!(
            choose_source(false, false, true).unwrap(),
            PortalSource::Window
        );
    }

    #[test]
    fn region_crop_and_pipeline_name_the_pipewire_fd() {
        let crop = crop_edges((1920, 1080), (100, 80, 640, 360)).unwrap();
        assert_eq!(
            crop,
            CropEdges {
                left: 100,
                top: 80,
                right: 1180,
                bottom: 640,
            }
        );
        assert!(crop_edges((100, 100), (0, 0, 100, 100)).is_none());
        let pipeline = gstreamer_pipeline(&PipelineInput {
            node_id: 42,
            fd: 7,
            location: "/tmp/take/screen.mov",
            encoder: "x264enc bitrate=20000 speed-preset=veryfast key-int-max=60",
            crop: Some(crop),
        });
        assert!(pipeline.contains("fd=7"));
        assert!(pipeline.contains("path=42"));
        assert!(pipeline.contains("always-copy=true"));
        assert!(pipeline.contains("videocrop left=100 top=80 right=1180 bottom=640"));
        assert!(pipeline.contains("x264enc bitrate=20000"));
        assert!(pipeline.contains("location=\"/tmp/take/screen.mov\""));
        assert_eq!(bitrate_kbps("20M"), 20_000);
    }
}
