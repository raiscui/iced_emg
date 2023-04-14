/*
 * @Author: Rais
 * @Date: 2023-04-13 15:52:29
 * @LastEditTime: 2023-04-14 17:38:01
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2023-04-13 14:31:42
 * @LastEditTime: 2023-04-13 15:49:39
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2023-04-12 18:01:05
 * @LastEditTime: 2023-04-12 23:53:05
 * @LastEditors: Rais
 * @Description:
 */
use emg_state::{topo, CloneState, StateVar};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use emg_common::{IdStr, NotNan};
use emg_renderer::{Blob, Format, Image};
use emg_state::{state_lit::StateVarLit, use_state, StateAnchor};
use num_traits::ToPrimitive;
use tracing::debug;

use thiserror::Error;
// use byte_slice_cast::*;
use derive_more::Display;
use gst::{element_error, glib, prelude::*, Element};
use gstreamer as gst;
use gstreamer_app as gst_app;

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {src}: {error} (debug: {debug:?})")]
struct ErrorMessage {
    src: glib::GString,
    error: glib::Error,
    debug: Option<glib::GString>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Glib(#[from] glib::Error),
    #[error("{0}")]
    Bool(#[from] glib::BoolError),
    #[error("failed to get the gstreamer bus")]
    Bus,
    #[error("{0}")]
    StateChange(#[from] gst::StateChangeError),
    #[error("failed to cast gstreamer element")]
    Cast,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("invalid URI")]
    Uri,
    #[error("failed to get media capabilities")]
    Caps,
    #[error("failed to query media duration or position")]
    Duration,
    #[error("failed to sync with playback")]
    Sync,
}

#[derive(Display)]
#[display(fmt = "VideoPlayer{{uri:{uri}}}")]
pub struct VideoPlayer {
    uri: IdStr,
    bus: gst::Bus,
    source: gst::Bin,

    width: i32,
    height: i32,
    framerate: f64,
    duration: std::time::Duration,

    // frame: Arc<Mutex<Option<Image>>>,
    frame: Arc<StateVar<Option<Image>>>,
    wait: mpsc::Receiver<()>,
    paused: bool,
    muted: bool,
    looping: bool,
    is_eos: bool,
    restart_stream: bool,
}

impl std::fmt::Debug for VideoPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoPlayer")
            .field("uri", &self.uri)
            // .field("bus", &self.bus)
            // .field("source", &self.source)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("framerate", &self.framerate)
            .field("duration", &self.duration)
            // .field("frame", &self.frame)
            // .field("wait", &self.wait)
            .field("paused", &self.paused)
            .field("muted", &self.muted)
            .field("looping", &self.looping)
            .field("is_eos", &self.is_eos)
            .field("restart_stream", &self.restart_stream)
            .finish()
    }
}

// impl VideoPlayerT for VideoPlayer {
impl VideoPlayer {
    // type Error = Error;
    // type ImageOut = Image;
    #[topo::nested]
    pub fn new(uri: &str, live: bool) -> Result<Self, Error> {
        gst::init()?;

        let source = gst::parse_launch(&format!("playbin uri=\"{uri}\" video-sink=\"videoconvert ! videoscale ! appsink name=app_sink caps=video/x-raw,format=RGBA,pixel-aspect-ratio=1/1\""))?;
        let source = source.downcast::<gst::Bin>().unwrap();

        let video_sink = source.property::<Option<Element>>("video-sink").unwrap();
        let pad = video_sink.pads().get(0).cloned().unwrap();
        let pad = pad.dynamic_cast::<gst::GhostPad>().unwrap();
        let bin = pad
            .parent_element()
            .unwrap()
            .downcast::<gst::Bin>()
            .unwrap();

        let app_sink = bin.by_name("app_sink").unwrap();
        let app_sink = app_sink.downcast::<gst_app::AppSink>().unwrap();

        // let frame = Arc::new(Mutex::new(None));
        // let frame_weak = Arc::downgrade(&frame);
        let f_data = use_state(|| None);
        let frame = Arc::new(f_data);
        let frame_weak = Arc::downgrade(&frame);

        let (notify, wait) = mpsc::channel();

        app_sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let frame = match frame_weak.upgrade() {
                        Some(frame) => frame,
                        None => return Ok(gst::FlowSuccess::Ok),
                    };

                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                    let pad = sink.static_pad("sink").ok_or(gst::FlowError::Error)?;

                    let caps = pad.current_caps().ok_or(gst::FlowError::Error)?;
                    let s = caps.structure(0).ok_or(gst::FlowError::Error)?;
                    let width = s.get::<i32>("width").map_err(|_| gst::FlowError::Error)?;
                    let height = s.get::<i32>("height").map_err(|_| gst::FlowError::Error)?;

                    // *frame.lock().map_err(|_| gst::FlowError::Error)? = Some(from_pixels(
                    //     width as _,
                    //     height as _,
                    //     map.as_slice().to_owned(),
                    // ));
                    frame.set(Some(from_pixels(
                        width as _,
                        height as _,
                        map.as_slice().to_owned(),
                    )));

                    // notify.send(()).map_err(|_| gst::FlowError::Error)?;

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        source.set_state(gst::State::Playing)?;
        source.state(gst::ClockTime::from_seconds(15)).0?;

        debug!(target = "video-player", "0");

        // extract resolution and framerate
        // TODO(jazzfool): maybe we want to extract some other information too?
        let caps = pad.current_caps().ok_or(Error::Caps)?;
        debug!(target = "video-player", "1");

        let s = caps.structure(0).ok_or(Error::Caps)?;
        debug!(target = "video-player", "2");

        let width = s.get::<i32>("width").map_err(|_| Error::Caps)?;
        debug!(target = "video-player", "3");

        let height = s.get::<i32>("height").map_err(|_| Error::Caps)?;
        debug!(target = "video-player", "4");

        let framerate = s
            .get::<gst::Fraction>("framerate")
            .map_err(|_| Error::Caps)?;
        debug!(target = "video-player", "5");

        let duration = if !live {
            std::time::Duration::from_nanos(
                source
                    .query_duration::<gst::ClockTime>()
                    .ok_or(Error::Duration)?
                    .nseconds(),
            )
        } else {
            std::time::Duration::from_secs(0)
        };

        Ok(VideoPlayer {
            uri: uri.into(),
            bus: source.bus().unwrap(),
            source,

            width,
            height,
            framerate: framerate.0 .to_f64().unwrap(/* if the video framerate is bad then it would've been implicitly caught far earlier */),
            duration,

            frame,
            wait,
            paused: false,
            muted: false,
            looping: false,
            is_eos: false,
            restart_stream: false,
        })
    }

    pub fn watch_frame(&self) -> StateAnchor<Image> {
        self.frame.clone().watch().map(|x| {
            x.clone()
                .unwrap_or_else(|| from_pixels(1, 1, vec![0, 0, 0, 1]))
        })
    }

    pub fn frame_image(&self) -> Image {
        // self.frame
        //     .lock()
        //     .expect("failed to lock frame")
        //     .clone()
        //     .unwrap_or_else(|| from_pixels(1, 1, vec![0, 0, 0, 1]))
        todo!()
    }
}

fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    let data = Arc::new(pixels);
    let blob = Blob::new(data);
    Image::new(blob, Format::Rgba8, width, height)
}
