//! Windows media sessions are read on a dedicated MTA worker. Event callbacks
//! only mark the state dirty and wake the worker, even during bursts.

use muelle_shell::{icons::IconImage, imagery};
use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicIsize, AtomicU8, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::Duration,
};
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    },
    Storage::Streams::{DataReader, IRandomAccessStreamReference},
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        UI::WindowsAndMessaging::PostMessageW,
    },
    core::RuntimeType,
};
use windows_future::IAsyncOperation;

const SESSION: u8 = 1;
const TRACK: u8 = 2;
const PLAYBACK: u8 = 4;
const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy)]
pub enum Control {
    Previous,
    PlayPause,
    Next,
}

#[derive(Clone, Default, PartialEq)]
pub struct Track {
    generation: u64,
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub previous: bool,
    pub toggle: bool,
    pub next: bool,
    pub artwork: Option<Arc<IconImage>>,
}

#[cfg(test)]
impl Track {
    pub fn test_fixture() -> Self {
        Self {
            title: "A deliberately long track title for clipping".into(),
            artist: "Artist".into(),
            playing: true,
            previous: true,
            toggle: true,
            next: true,
            ..Default::default()
        }
    }
}

/// The supplied fallback is bundled, so it works offline and after installation.
pub fn artwork(track: &Track) -> Option<&IconImage> {
    static FALLBACK: OnceLock<Option<IconImage>> = OnceLock::new();
    track.artwork.as_deref().or_else(|| {
        FALLBACK
            .get_or_init(|| {
                imagery::decode(
                    include_bytes!("../../../assets/media-fallback.png"),
                    128,
                    1_000_000,
                )
                .map(imagery::circular)
            })
            .as_ref()
    })
}

fn read_artwork(thumbnail: IRandomAccessStreamReference) -> Option<Arc<IconImage>> {
    let stream =
        complete_with_timeout(thumbnail.OpenReadAsync().ok()?, Duration::from_millis(400))?;
    let size = stream.Size().ok()?;
    // Reject oversized streams before allocating or asking Windows to decode.
    if size == 0 || size > 2 * 1024 * 1024 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    if complete_with_timeout(
        reader.LoadAsync(size as u32).ok()?,
        Duration::from_millis(400),
    )? != size as u32
    {
        return None;
    }
    let mut bytes = vec![0; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    imagery::decode(&bytes, 128, 16_000_000)
        .map(imagery::circular)
        .map(Arc::new)
}

#[derive(Clone)]
struct Events {
    wake: SyncSender<()>,
    dirty: Arc<AtomicU8>,
}

impl Events {
    fn mark(&self, flags: u8) {
        self.dirty.fetch_or(flags, Ordering::Release);
        let _ = self.wake.try_send(());
    }
}

pub struct Watch {
    window: Arc<AtomicIsize>,
    state: Arc<Mutex<Option<Track>>>,
    events: Events,
    commands: SyncSender<(u64, Control)>,
}

impl Watch {
    pub fn start(hwnd: HWND, message: u32) -> Option<Self> {
        let window = Arc::new(AtomicIsize::new(hwnd.0 as isize));
        let state = Arc::new(Mutex::new(None));
        let (wake, inbox) = mpsc::sync_channel(1);
        let (commands, requests) = mpsc::sync_channel(4);
        let events = Events {
            wake,
            dirty: Arc::new(AtomicU8::new(0)),
        };
        {
            let (window, state, events) = (window.clone(), state.clone(), events.clone());
            thread::Builder::new()
                .name("muelle-media".into())
                .spawn(move || {
                    // SAFETY: This is a new thread. The apartment is kept alive
                    // until all of the worker's WinRT objects have been dropped.
                    if unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.is_ok() {
                        run(&window, message, &state, &events, &inbox, &requests);
                        unsafe { RoUninitialize() };
                    }
                })
                .ok()?;
        }
        Some(Self {
            window,
            state,
            events,
            commands,
        })
    }

    pub fn snapshot(&self) -> Option<Track> {
        self.state.lock().ok().and_then(|state| state.clone())
    }

    pub fn send(&self, track: &Track, control: Control) {
        if self.commands.try_send((track.generation, control)).is_ok() {
            self.events.mark(0);
        }
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        // The worker can be inside a media app's asynchronous operation.
        self.window.store(0, Ordering::Release);
        self.events.mark(0);
    }
}

fn complete<T: RuntimeType>(operation: IAsyncOperation<T>) -> Option<T> {
    complete_with_timeout(operation, TIMEOUT)
}

fn complete_with_timeout<T: RuntimeType>(
    operation: IAsyncOperation<T>,
    timeout: Duration,
) -> Option<T> {
    let (send, receive) = mpsc::channel();
    operation
        .SetCompleted(&windows_future::AsyncOperationCompletedHandler::<T>::new(
            move |_, _| {
                // Only a wake-up crosses threads. Non-agile streams and all other
                // WinRT results are retrieved on the worker's original apartment.
                let _ = send.send(());
                Ok(())
            },
        ))
        .ok()?;
    match receive.recv_timeout(timeout) {
        Ok(()) => operation.GetResults().ok(),
        Err(_) => {
            let _ = operation.Cancel();
            None
        }
    }
}

struct Current {
    session: Session,
    track_event: Option<i64>,
    playback_event: Option<i64>,
    title: String,
    artist: String,
    artwork: Option<Arc<IconImage>>,
}

impl Current {
    fn new(session: Session, events: &Events) -> Self {
        let track = events.clone();
        let playback = events.clone();
        let track_event = session
            .MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
                track.mark(TRACK);
                Ok(())
            }))
            .ok();
        let playback_event = session
            .PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
                playback.mark(PLAYBACK);
                Ok(())
            }))
            .ok();
        Self {
            session,
            track_event,
            playback_event,
            title: String::new(),
            artist: String::new(),
            artwork: None,
        }
    }

    fn update_track(&mut self) {
        // A failed read belongs to this metadata change, not to the previous
        // song. Never pair yesterday's title/artist with the current session.
        self.title.clear();
        self.artist.clear();
        self.artwork = None;
        if let Some(properties) = self
            .session
            .TryGetMediaPropertiesAsync()
            .ok()
            .and_then(complete)
        {
            self.title = properties
                .Title()
                .map(|value| value.to_string_lossy())
                .unwrap_or_default();
            self.artist = properties
                .Artist()
                .map(|value| value.to_string_lossy())
                .unwrap_or_default();
            self.artwork = properties.Thumbnail().ok().and_then(read_artwork);
        }
    }

    fn snapshot(&self, generation: u64) -> Option<Track> {
        let info = self.session.GetPlaybackInfo().ok()?;
        let status = info.PlaybackStatus().ok()?;
        if !matches!(status, Status::Playing | Status::Paused | Status::Changing) {
            return None;
        }
        let playing = matches!(status, Status::Playing | Status::Changing);
        let controls = info.Controls().ok()?;
        Some(Track {
            generation,
            title: self.title.clone(),
            artist: self.artist.clone(),
            artwork: self.artwork.clone(),
            playing,
            previous: controls.IsPreviousEnabled().unwrap_or(false),
            next: controls.IsNextEnabled().unwrap_or(false),
            toggle: controls.IsPlayPauseToggleEnabled().unwrap_or(false)
                || if playing {
                    controls.IsPauseEnabled()
                } else {
                    controls.IsPlayEnabled()
                }
                .unwrap_or(false),
        })
    }

    fn command(&self, control: Control) {
        let operation = match control {
            Control::Previous => self.session.TrySkipPreviousAsync(),
            Control::Next => self.session.TrySkipNextAsync(),
            Control::PlayPause => {
                let Ok(info) = self.session.GetPlaybackInfo() else {
                    return;
                };
                if info
                    .Controls()
                    .and_then(|c| c.IsPlayPauseToggleEnabled())
                    .unwrap_or(false)
                {
                    self.session.TryTogglePlayPauseAsync()
                } else if info
                    .PlaybackStatus()
                    .is_ok_and(|status| matches!(status, Status::Playing | Status::Changing))
                {
                    self.session.TryPauseAsync()
                } else {
                    self.session.TryPlayAsync()
                }
            }
        };
        if let Ok(operation) = operation {
            // Keep commands ordered even if the player acknowledges one slowly.
            let _ = complete(operation);
        }
    }
}

impl Drop for Current {
    fn drop(&mut self) {
        if let Some(token) = self.track_event {
            let _ = self.session.RemoveMediaPropertiesChanged(token);
        }
        if let Some(token) = self.playback_event {
            let _ = self.session.RemovePlaybackInfoChanged(token);
        }
    }
}

fn run(
    window: &AtomicIsize,
    message: u32,
    state: &Mutex<Option<Track>>,
    events: &Events,
    inbox: &Receiver<()>,
    requests: &Receiver<(u64, Control)>,
) {
    let Some(manager) = Manager::RequestAsync().ok().and_then(complete) else {
        return;
    };
    let sessions = events.clone();
    let current_session = events.clone();
    let sessions_event = manager
        .SessionsChanged(&TypedEventHandler::new(move |_, _| {
            sessions.mark(SESSION);
            Ok(())
        }))
        .ok();
    let current_event = manager
        .CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
            current_session.mark(SESSION);
            Ok(())
        }))
        .ok();
    let mut current: Option<Current> = None;
    let mut generation: u64 = 0;
    events.mark(SESSION);
    while inbox.recv().is_ok() && window.load(Ordering::Acquire) != 0 {
        let mut dirty = events.dirty.swap(0, Ordering::AcqRel);
        if dirty & SESSION != 0 {
            let session = manager.GetCurrentSession().ok();
            if session.as_ref() != current.as_ref().map(|active| &active.session) {
                generation = generation.wrapping_add(1);
                current = session.map(|session| Current::new(session, events));
                dirty |= TRACK;
            }
        }
        if dirty != 0 {
            if dirty & TRACK != 0
                && let Some(current) = &mut current
            {
                current.update_track();
            }
            let latest = current
                .as_ref()
                .and_then(|current| current.snapshot(generation));
            if let Ok(mut snapshot) = state.lock()
                && *snapshot != latest
            {
                *snapshot = latest;
                drop(snapshot);
                let hwnd = window.load(Ordering::Acquire);
                if hwnd != 0 {
                    // SAFETY: Posts a notification by value to the bar. The
                    // message holds no reference to the worker or its data.
                    unsafe {
                        let _ =
                            PostMessageW(Some(HWND(hwnd as *mut _)), message, WPARAM(0), LPARAM(0));
                    }
                }
            }
        }
        for (target, control) in requests.try_iter().take(4) {
            if window.load(Ordering::Acquire) == 0 {
                break;
            }
            if target == generation
                && let Some(active) = &current
                && manager.GetCurrentSession().ok().as_ref() == Some(&active.session)
            {
                active.command(control);
            }
        }
    }
    if let Some(token) = sessions_event {
        let _ = manager.RemoveSessionsChanged(token);
    }
    if let Some(token) = current_event {
        let _ = manager.RemoveCurrentSessionChanged(token);
    }
}

#[cfg(test)]
mod native_probe {
    use super::*;

    #[test]
    #[ignore = "Read-only artwork probe of the current Windows media session"]
    fn native_artwork_stream_stays_on_its_apartment_and_fallback_decodes()
    -> windows::core::Result<()> {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)?;
        }
        let result = (|| {
            use windows::{
                Storage::Streams::{
                    DataWriter, InMemoryRandomAccessStream, RandomAccessStreamReference,
                },
                core::Interface,
            };
            let check_stream = || -> Option<()> {
                let stream = InMemoryRandomAccessStream::new().ok()?;
                let writer =
                    DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0).ok()?).ok()?;
                writer
                    .WriteBytes(include_bytes!("../../../assets/media-fallback.png"))
                    .ok()?;
                complete(writer.StoreAsync().ok()?)?;
                let reference = RandomAccessStreamReference::CreateFromStream(&stream).ok()?;
                let image = read_artwork(reference.cast().ok()?)?;
                assert!(image.width > 0 && image.width <= 128 && image.width == image.height);
                Some(())
            };
            assert!(
                check_stream().is_some(),
                "Known local thumbnail stream did not decode"
            );
            let fallback = artwork(&Track::default())
                .is_some_and(|image| image.width > 0 && image.width <= 128);
            let Some(manager) = Manager::RequestAsync().ok().and_then(complete) else {
                return fallback;
            };
            if let Ok(session) = manager.GetCurrentSession() {
                let (wake, _inbox) = mpsc::sync_channel(1);
                let events = Events {
                    wake,
                    dirty: Arc::new(AtomicU8::new(0)),
                };
                let mut current = Current::new(session, &events);
                current.update_track();
                if let Some(image) = &current.artwork {
                    assert!(image.width <= 128 && image.height <= 128);
                    eprintln!("Windows media artwork: {}x{}", image.width, image.height);
                } else {
                    eprintln!(
                        "Current session provides no readable artwork; bundled fallback is available"
                    );
                }
            } else {
                eprintln!("No active media session; bundled fallback is available");
            }
            fallback
        })();
        unsafe {
            RoUninitialize();
        }
        assert!(result);
        Ok(())
    }

    #[test]
    #[ignore = "Requires the Windows media session service"]
    fn windows_media_manager_responds_in_mta() -> windows::core::Result<()> {
        // SAFETY: Initialize this test thread's apartment and release it
        // after the manager has been dropped.
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)?;
        }
        let manager = Manager::RequestAsync().ok().and_then(complete);
        let connected = manager
            .as_ref()
            .is_some_and(|manager| manager.GetSessions().is_ok());
        drop(manager);
        // SAFETY: Balances this thread's successful initialization.
        unsafe {
            RoUninitialize();
        }
        assert!(connected);
        Ok(())
    }
}
