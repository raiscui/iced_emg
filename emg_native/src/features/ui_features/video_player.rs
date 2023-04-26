/*
 * @Author: Rais
 * @Date: 2023-04-13 15:52:29
 * @LastEditTime: 2023-04-26 15:13:43
 * @LastEditors: Rais
 * @Description:
 */

use emg_global::{global_anima_running_add, global_anima_running_remove, global_elapsed};

use emg_state::{
    general_traits::CloneStateOut, state_store_with, topo, use_state_voa, CloneState,
    StateMultiAnchor, StateVOA, StateVar,
};
use std::sync::{Arc, Mutex};
use std::{rc::Rc, sync::RwLock};

use emg_common::{IdStr, NotNan, RenderLoopCommand};
use emg_renderer::{Blob, Format, Image};
use emg_state::{state_lit::StateVarLit, use_state, StateAnchor};
use num_traits::ToPrimitive;
use tracing::{debug, debug_span, trace};

use thiserror::Error;
// use byte_slice_cast::*;
use derive_more::Display;
use gst::{element_error, glib, prelude::*, Element};
use gstreamer as gst;
use gstreamer_app as gst_app;

use crate::global_loop_controller;

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
    pub source: Rc<gst::Bin>,

    width: i32,
    height: i32,
    framerate: f64,
    duration: std::time::Duration,

    frame: Arc<Mutex<Option<Image>>>,
    frame_image_sa: StateAnchor<Image>,
    paused: StateVOA<bool>,
    muted: bool,
    looping: bool,
    is_eos: bool,
    restart_stream: bool,
    paused_callback: StateAnchor<bool>, //Save for  global_anima_running_remove
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        global_anima_running_remove(&self.paused_callback)
    }
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

impl VideoPlayer {
    #[topo::nested]
    pub fn new(uri: &str, live: bool) -> Result<Self, Error>
// where
        // F: Fn() + 'static + Send,
    {
        let _span = debug_span!("VideoPlayer").entered();
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

        // ─────────────────────────────────────────────────────────────

        let frame = Arc::new(Mutex::new(None));
        let frame_weak = Arc::downgrade(&frame);
        let frame_weak2 = std::sync::Weak::clone(&frame_weak);

        let paused = use_state_voa(|| true);
        debug!(target:"video-player"," paused:{:?}", paused);

        //NOTE don't use insert_before_fn_in_topo make video Paused. 如果stateVOA设置内部为 anchor 或者 做了 bi, 那么这种 before_fn 不管用
        // let source2 = source.clone();
        // let af = paused
        //     .insert_before_fn_in_topo(
        //         move |_skip, _current, v| {
        //             let is_paused = v.get_out_val();

        //             source2.set_state(if is_paused {
        //                 gst::State::Paused
        //             } else {
        //                 gst::State::Playing
        //             })
        //             .unwrap(/* state was changed in ctor; state errors caught there */);
        //         },
        //         false,
        //         &[],
        //     )
        //     .unwrap();
        // paused.link_callback_drop(af);
        // ─────────────────────────────────────────────────────────────

        let source = Rc::new(source);
        let source_wk = Rc::downgrade(&source);

        let paused_callback = paused.watch().debounce().map(move |&is_paused| {
            trace!(target:"video-player-global-check","------------- paused change ==========={}", is_paused);

            source_wk.upgrade().expect("source is can't up to Rc now").set_state(if is_paused {
                    gst::State::Paused
                } else {
                    gst::State::Playing
                })
                .unwrap(/* state was changed in ctor; state errors caught there */);


            // !is_paused
            //不再激活 global_anima_running_add, 由 global_loop_controller 激活loop
            // 好处是不用去管哪些什么时候片子播完了,播放错误等非paused变化导致需要停止loop的, 必须要 手动暂停
            //只要 没有 publish(RenderLoopCommand::Schedule)  , loop就停了
            // 被动检查变主动
            false
        });
        let use_am_watch = false;
        // if use_am_watch {
        global_anima_running_add(paused_callback.clone());
        // }

        let frame_image_sa = global_elapsed().watch().map_mut(
            from_pixels(1, 1, vec![0, 0, 0, 1]),
            move |out, _x| {
                // let frame_image_sa = global_elapsed().watch().map_mut(
                // from_pixels(1, 1, vec![0, 0, 0, 1]),
                // move |out, _x| {
                // ─────────────────────────────────────-

                // if is_paused {
                //     println!("============= paused change ==========={}", is_paused);
                // }

                let opt_frame_image = frame_weak2
                    .upgrade()
                    .and_then(|f| f.lock().expect("in main thread,failed lock frame").take());
                if let Some(image) = opt_frame_image {
                    *out = image;
                    true
                } else {
                    false
                }
            },
        );

        // ─────────────────────────────────────────────────────────────

        // let (notify, wait) = mpsc::channel();
        let loop_controller = global_loop_controller();
        // ─────────────────────────────────────────────────────────────
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

                    *frame.lock().map_err(|_| gst::FlowError::Error)? = Some(from_pixels(
                        width as _,
                        height as _,
                        map.as_slice().to_owned(),
                    ));

                    // render_signal();
                    debug!(target:"RenderLoopCommand","will send schedule_render message...");
                    // loop_controller
                    // .send(RenderLoopCommand::Schedule)
                    // .expect("video send new frame got");
                    if !use_am_watch {
                        loop_controller.publish(RenderLoopCommand::Schedule);
                    }
                    debug!(target:"RenderLoopCommand","schedule_render message sended .");

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        trace!(target: "video-player", "set_state");
        // source.set_state(gst::State::Playing)?;
        source.set_state(gst::State::Paused)?;

        trace!(target: "video-player", "state set wait");
        source.state(gst::ClockTime::from_seconds(15)).0?;

        trace!(target: "video-player", "0");

        // extract resolution and framerate
        // TODO(jazzfool): maybe we want to extract some other information too?
        let caps = pad.current_caps().ok_or(Error::Caps)?;
        trace!(target: "video-player", "1");

        let s = caps.structure(0).ok_or(Error::Caps)?;
        trace!(target: "video-player", "2");

        let width = s.get::<i32>("width").map_err(|_| Error::Caps)?;
        trace!(target: "video-player", "3");

        let height = s.get::<i32>("height").map_err(|_| Error::Caps)?;
        trace!(target: "video-player", "4");

        let framerate = s
            .get::<gst::Fraction>("framerate")
            .map_err(|_| Error::Caps)?;
        trace!(target: "video-player", "5");

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
            frame_image_sa,
            // wait,
            paused,
            muted: false,
            looping: false,
            is_eos: false,
            restart_stream: false,
            paused_callback,
        })
    }

    pub fn set_paused(&self, is_pause: bool) -> Result<(), Error> {
        self.source.set_state(if is_pause {
            gst::State::Paused
        } else {
            gst::State::Playing
        })?;
        Ok(())
    }

    // pub fn watch_frame(&self) -> StateAnchor<Image> {
    //     self.frame.clone().watch().map(|x| {
    //         x.clone()
    //             .unwrap_or_else(|| from_pixels(1, 1, vec![0, 0, 0, 1]))
    //     })
    // }

    pub fn frame_image(&self) -> Image {
        // self.frame
        //     .lock()
        //     .expect("failed to lock frame")
        //     .clone()
        //     .unwrap_or_else(|| from_pixels(1, 1, vec![0, 0, 0, 1]))
        todo!()
    }
    pub fn frame_image_sa(&self) -> &StateAnchor<Image> {
        &self.frame_image_sa
    }

    #[inline]
    pub fn paused(&self) -> StateVOA<bool> {
        self.paused
    }
}

//TODO bump pool
fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    let data = Arc::new(pixels);
    let blob = Blob::new(data);
    Image::new(blob, Format::Rgba8, width, height)
}

#[cfg(test)]
mod test_video {
    use crate::Bus;

    use super::VideoPlayer;
    use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};

    use emg_common::RenderLoopCommand;
    use tracing::{debug, debug_span, info, instrument, warn};

    fn tracing_init() -> Result<()> {
        println!("tracing init");
        // use tracing_error::ErrorLayer;
        use tracing_subscriber::prelude::*;

        let filter_layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_indent_amount(4)
            .with_targets(true)
            .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
                |metadata, cx| {
                    let skip_target = ["emg_state", "underlay", "to_layout_override"];
                    for t in skip_target {
                        if metadata.target().contains(t) {
                            return false;
                        }
                    }

                    let skip_span = ["xxx"];
                    for t in skip_span {
                        if metadata.name().contains(t) {
                            return false;
                        }
                    }

                    let skip_fields = ["native_events"];
                    // let skip_fields = ["window_event"];

                    for x in metadata.fields() {
                        let f_str = format!("{}", x);
                        if skip_fields.contains(&f_str.as_str()) {
                            return false;
                        }
                    }

                    // let keep_target = ["emg_element"];
                    // if !keep_target.iter().any(|t| metadata.target().starts_with(t)) {
                    //     return false;
                    // }

                    // let keep_span = ["event_matching"];
                    // if metadata.is_span() && keep_span.contains(&metadata.name()) {
                    //     return true;
                    // }

                    true
                },
            ))
            .with_filter(tracing_subscriber::EnvFilter::new(
                // "shaping=warn,[DRAG]=debug,[CLICK]=debug,winit_event=debug,[event_matching]=debug,[LayoutOverride]=debug",
                // "shaping=warn,[DRAG]=debug,[event_matching_filter]=debug",
                // "[event_matching]=debug,[event_matching_filter]=debug",
                "video-player=debug,run-loop=debug,RenderLoopCommand=debug,",
            ))
            .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
                |metadata, cx| {
                    // let keep_target = ["emg_element"];
                    // if !keep_target.iter().any(|t| metadata.target().starts_with(t)) {
                    //     return false;
                    // }

                    let keep_span = [];
                    if metadata.is_span() && keep_span.contains(&metadata.name()) {
                        return true;
                    }

                    keep_span.is_empty()
                },
            ));

        // ─────────────────────────────────────────────────────────────────────────────

        tracing_subscriber::registry()
            // .with(layout_override_layer)
            // .with(event_matching_layer)
            // .with(touch_layer)
            // .with(tracing_subscriber::fmt::layer().with_filter(tracing::metadata::LevelFilter::ERROR))
            .with(filter_layer)
            // .with(out_layer)
            // .with(ErrorLayer::default())
            .init();

        // ─────────────────────────────────────────────────────────────────────────────

        color_eyre::install()
    }

    #[test]
    fn video_test() {
        tracing_init().unwrap();
        let bus = Bus::new(|x: RenderLoopCommand| {});
        illicit::Layer::new().offer(bus).enter(|| {
            debug!(target:"video-player","xxxx");
            let x = VideoPlayer::new(
                "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                false,
            );
            let x = VideoPlayer::new(
                "file:///Users/cuiluming/Downloads/sintel_trailer-1080p.mp4",
                false,
            );
        });
    }
}
