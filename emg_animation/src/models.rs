pub mod color;
pub mod convert;
pub mod opacity;
use emg_core::{measures::Unit, smallvec, IdStr, SmallVec, TypeName};
use emg_core::{vector, Vector};
// use iter_fixed::IntoIteratorFixed;
use crate::{Debuggable, MOTION_SIZE, PROP_SIZE};
use derive_more::Display;
use ordered_float::NotNan;
use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Duration,
};
use tracing::{trace, warn};

// use emg_debuggable::{dbg4, Debuggable};

type EaseFnT = Rc<Debuggable<Box<dyn Fn(Precision) -> Precision>>>;

const DIM2: usize = 2;
const DIM3: usize = 3;
#[derive(Clone, Debug, Eq)]
pub struct Easing {
    pub progress: NotNan<Precision>,
    pub duration: Duration,
    pub start: NotNan<Precision>,
    pub ease: EaseFnT,
}

impl PartialEq for Easing {
    fn eq(&self, other: &Self) -> bool {
        self.progress == other.progress
            && self.duration == other.duration
            && self.start == other.start
            && self.ease.text == other.ease.text
    }
}

// impl Clone for Easing {
//     fn clone(&self) -> Self {
//         Self{
//             progress:self.progress,
//             duration: self.duration,
//             start: self.start,
//             ease: Rc::clone(&self.ease),
//         }
//     }
// }

/*
 * @Author: Rais
 * @Date: 2021-05-07 19:19:51
 * @LastEditTime: 2021-05-16 18:00:33
 * @LastEditors: Rais
 * @Description:
 */
// 1秒(s) ＝1000毫秒(ms)
// 1毫秒(ms)＝1000微秒(us)
// 1微秒 micro (us)＝1000纳秒(ns)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Spring {
        stiffness: NotNan<Precision>,
        damping: NotNan<Precision>,
    },
    Easing(Easing),
    AtSpeed {
        per_second: NotNan<Precision>,
    },
}

#[derive(Display, Clone, Debug, PartialEq, Eq)]
#[display(fmt = "Motion:{{pos:{}}}", position)]
pub struct Motion {
    pub(crate) position: NotNan<Precision>,
    pub(crate) velocity: NotNan<Precision>,
    pub(crate) target: NotNan<Precision>,
    pub(crate) interpolation: Interpolation,
    pub(crate) unit: Unit,
    pub(crate) interpolation_override: Option<Interpolation>,
}

impl Motion {
    /// Get a mutable reference to the motion's interpolation override.
    pub fn interpolation_override_mut(&mut self) -> &mut Option<Interpolation> {
        &mut self.interpolation_override
    }

    /// Get a reference to the motion's position.
    #[must_use]
    pub const fn position(&self) -> &NotNan<Precision> {
        &self.position
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CubicCurveMotion {
    //TODO check size right
    control1: SmallVec<[Motion; MOTION_SIZE]>,
    control2: SmallVec<[Motion; MOTION_SIZE]>,
    point: SmallVec<[Motion; MOTION_SIZE]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CubicCurveMotionOG {
    control1: Vector<Motion>,
    control2: Vector<Motion>,
    point: Vector<Motion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuadraticCurveMotion {
    control: SmallVec<[Motion; MOTION_SIZE]>,
    point: SmallVec<[Motion; MOTION_SIZE]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuadraticCurveMotionOG {
    control: Vector<Motion>,
    point: Vector<Motion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcMotion {
    x: Motion,
    y: Motion,
    radius: Motion,
    start_angle: Motion,
    end_angle: Motion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathCommand {
    Move(SmallVec<[Motion; MOTION_SIZE]>),
    MoveTo(SmallVec<[Motion; MOTION_SIZE]>),
    Line(SmallVec<[Motion; MOTION_SIZE]>),
    LineTo(SmallVec<[Motion; MOTION_SIZE]>),
    Horizontal(Motion),
    HorizontalTo(Motion),
    Vertical(Motion),
    VerticalTo(Motion),
    Curve(CubicCurveMotion),
    CurveTo(CubicCurveMotion),
    Quadratic(QuadraticCurveMotion),
    QuadraticTo(QuadraticCurveMotion),
    SmoothQuadratic(SmallVec<[[Motion; DIM2]; MOTION_SIZE]>),
    SmoothQuadraticTo(SmallVec<[[Motion; DIM2]; MOTION_SIZE]>),
    Smooth(SmallVec<[[Motion; DIM2]; MOTION_SIZE]>),
    SmoothTo(SmallVec<[[Motion; DIM2]; MOTION_SIZE]>),
    ClockwiseArc(ArcMotion),
    AntiClockwiseArc(ArcMotion),
    Close,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathCommandOG {
    Move(Vector<Motion>),
    MoveTo(Vector<Motion>),
    Line(Vector<Motion>),
    LineTo(Vector<Motion>),
    Horizontal(Motion),
    HorizontalTo(Motion),
    Vertical(Motion),
    VerticalTo(Motion),
    Curve(CubicCurveMotionOG),
    CurveTo(CubicCurveMotionOG),
    Quadratic(QuadraticCurveMotionOG),
    QuadraticTo(QuadraticCurveMotionOG),
    SmoothQuadratic(Vector<[Motion; DIM2]>),
    SmoothQuadraticTo(Vector<[Motion; DIM2]>),
    Smooth(Vector<[Motion; DIM2]>),
    SmoothTo(Vector<[Motion; DIM2]>),
    ClockwiseArc(ArcMotion),
    AntiClockwiseArc(ArcMotion),
    Close,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowMotion {
    offset_x: Motion,
    offset_y: Motion,
    size: Motion,
    blur: Motion,
    red: Motion,
    green: Motion,
    blue: Motion,
    alpha: Motion,
}
pub type PropName = TypeName;

#[derive(Display, Clone, Debug, PartialEq, Eq)]
#[display(fmt = "{}")]
pub enum Property {
    #[display(fmt = "({},motion:{})", _0, _1)]
    Exact(PropName, String),

    #[display(fmt = "({},motion:{:?})", _0, _1)]
    Color(PropName, Box<[Motion; 4]>),

    #[display(fmt = "({},motion:{})", _0, _1)]
    Shadow(PropName, bool, Box<ShadowMotion>),

    #[display(fmt = "Prop({},motion:{})", _0, _1)]
    Prop(PropName, Motion),

    #[display(fmt = "Prop2({},motion:{:?})", _0, _1)]
    Prop2(PropName, Box<[Motion; 2]>),

    #[display(fmt = "Prop3({},motion:{:?})", _0, _1)]
    Prop3(PropName, Box<[Motion; 3]>),

    #[display(fmt = "Prop4({_0},motion:{_1:?})")]
    Prop4(PropName, Box<[Motion; 4]>),

    #[display(fmt = "Angle({},motion:{})", _0, _1)]
    Angle(PropName, Motion),

    #[display(fmt = "Points(Points,motion:{:?})", _0)]
    Points(Box<SmallVec<[[Motion; DIM2]; 1]>>),

    #[display(fmt = "Path(Path,motion:{:?})", _0)]
    Path(Box<SmallVec<[PathCommand; 1]>>),
    // Anchor(Rc<String>, StateAnchor<GenericSize>),
}

impl Property {
    #[must_use]
    pub fn name(&self) -> TypeName {
        use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};

        match self {
            Exact(name, ..)
            | Color(name, ..)
            | Shadow(name, ..)
            | Prop(name, ..)
            | Prop2(name, ..)
            | Prop3(name, ..)
            | Prop4(name, ..)
            | Angle(name, ..) => name.clone(),

            Points(_point) => TypeName::new(IdStr::new_inline("points")),

            Path(_path) => TypeName::new(IdStr::new_inline("Path")),
        }
    }
}

#[derive(Display, Clone, Debug, PartialEq, Eq)]
#[display(fmt = "{}")]
pub enum PropertyOG {
    #[display(fmt = "({},motion:{})", _0, _1)]
    Exact(PropName, String),

    #[display(fmt = "({},motion:{:?})", _0, _1)]
    Color(PropName, Vector<Motion>),

    #[display(fmt = "({},motion:{})", _0, _1)]
    Shadow(PropName, bool, Box<ShadowMotion>),

    #[display(fmt = "Prop({},motion:{})", _0, _1)]
    Prop(PropName, Motion),

    #[display(fmt = "Prop2({},motion:{:?})", _0, _1)]
    Prop2(PropName, Vector<Motion>),

    #[display(fmt = "Prop3({},motion:{:?})", _0, _1)]
    Prop3(PropName, Vector<Motion>),

    #[display(fmt = "Prop4({},motion:{:?})", _0, _1)]
    Prop4(PropName, Vector<Motion>),

    #[display(fmt = "Angle({},motion:{})", _0, _1)]
    Angle(PropName, Motion),

    #[display(fmt = "Points(Points,motion:{:?})", _0)]
    Points(Vector<[Motion; DIM2]>),

    #[display(fmt = "Path(Path,motion:{:?})", _0)]
    Path(Vector<PathCommandOG>),
    // Anchor(Rc<String>, StateAnchor<GenericSize>),
}

impl PropertyOG {
    #[must_use]
    pub fn name(&self) -> TypeName {
        use PropertyOG::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};

        match self {
            Exact(name, ..)
            | Color(name, ..)
            | Shadow(name, ..)
            | Prop(name, ..)
            | Prop2(name, ..)
            | Prop3(name, ..)
            | Prop4(name, ..)
            | Angle(name, ..) => name.clone(),

            Points(_point) => TypeName::new(IdStr::new_inline("points")),

            Path(_path) => TypeName::new(IdStr::new_inline("Path")),
        }
    }
}
// propertyName : Property -> String

#[derive(Clone, Debug, PartialEq)]
pub enum StepOG<Message>
where
    Message: Clone,
{
    _Step,
    To(Vector<PropertyOG>),
    ToWith(Vector<PropertyOG>),
    Set(Vector<PropertyOG>),
    Wait(Duration),
    Send(Message),
    Repeat(u32, Vector<StepOG<Message>>),
    Loop(Vector<StepOG<Message>>),
}
#[derive(Clone, Debug, PartialEq)]
pub enum Step<Message>
where
    Message: Clone,
{
    _Step,
    To(SmallVec<[Property; PROP_SIZE]>),
    ToWith(SmallVec<[Property; PROP_SIZE]>),
    Set(SmallVec<[Property; PROP_SIZE]>),
    Wait(Duration),
    Send(Message),
    Repeat(u32, VecDeque<Step<Message>>),
    Loop(VecDeque<Step<Message>>),
}

impl<Message> Step<Message>
where
    Message: Clone,
{
    /// # Errors
    ///
    /// Will return `Err` if `self` does not is 'Step::Wait(Duration)'
    /// permission to read it.
    pub fn try_into_wait(self) -> Result<Duration, Self> {
        if let Self::Wait(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Timing {
    pub(crate) current: Duration,
    pub(crate) dt: Duration,
}

impl Timing {
    #[must_use]
    pub const fn new(current: Duration, dt: Duration) -> Self {
        Self { current, dt }
    }

    /// Get a reference to the timing's current.
    #[must_use]
    pub const fn current(&self) -> Duration {
        self.current
    }

    /// Get a reference to the timing's dt.
    #[must_use]
    pub const fn dt(&self) -> Duration {
        self.dt
    }

    /// Set the timing's current.
    pub fn set_current(&mut self, current: Duration) {
        self.current = current;
    }

    /// Set the timing's dt.
    pub fn set_dt(&mut self, dt: Duration) {
        self.dt = dt;
    }

    /// Get a mutable reference to the timing's current.
    pub fn current_mut(&mut self) -> &mut Duration {
        &mut self.current
    }

    /// Get a mutable reference to the timing's dt.
    pub fn dt_mut(&mut self) -> &mut Duration {
        &mut self.dt
    }
}

pub type StepTimeVector<Message> = Vector<(Duration, VecDeque<Step<Message>>)>;

pub type StepTimeVectorOG<Message> = Vector<(Duration, Vector<StepOG<Message>>)>;

#[derive(Debug)]
pub struct Animation<Message>
where
    Message: Clone,
    // (Duration, Vec<Step<Message>>): Clone,
{
    pub(crate) steps: VecDeque<Step<Message>>,
    pub(crate) props: SmallVec<[Property; PROP_SIZE]>,
    pub(crate) timing: Timing,
    pub(crate) running: bool,
    pub(crate) interruption: StepTimeVector<Message>,
}

impl<Message> Animation<Message>
where
    Message: Clone,
{
    /// # Panics
    /// temp fn ,
    /// Will panic if p not prop
    #[must_use]
    pub fn get_position(&self, prop_index: usize) -> Precision {
        let p = self.props.get(prop_index).unwrap();
        match p {
            Property::Prop(_name, m) => m.position.into_inner(),
            _ => todo!("not implemented"),
        }
    }
}

#[derive(Debug)]
pub struct AnimationOG<Message>
where
    Message: Clone,
    // (Duration, Vec<Step<Message>>): Clone,
{
    pub(crate) steps: Vector<StepOG<Message>>,
    pub(crate) props: Vector<PropertyOG>,
    pub(crate) timing: Timing,
    pub(crate) running: bool,
    pub(crate) interruption: StepTimeVectorOG<Message>,
}

impl<Message> AnimationOG<Message>
where
    Message: Clone,
{
    /// # Panics
    /// temp fn ,
    /// Will panic if p not prop
    #[must_use]
    pub fn get_position(&self, prop_index: usize) -> Precision {
        let p = self.props.get(prop_index).unwrap();
        match p {
            PropertyOG::Prop(_name, m) => m.position.into_inner(),
            _ => todo!("not implemented"),
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tick(pub Duration);

impl Tick {
    #[must_use]
    pub const fn subsec_millis(&self) -> u32 {
        self.0.subsec_millis()
    }
    #[must_use]
    pub fn new(millisecond: Precision) -> Self {
        Self(Duration::from_micros(unsafe {
            (millisecond * 1000.).trunc().to_int_unchecked::<u64>()
        }))
    }
}

// mapPathMotion : (Motion -> Motion) -> PathCommand -> PathCommand
fn map_path_motion(func: &impl Fn(&mut Motion), cmd: &mut PathCommand) {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    // let func_clone = func.clone();
    let map_coords = |coords: &mut SmallVec<[[Motion; DIM2]; MOTION_SIZE]>| {
        coords
            .iter_mut()
            .for_each(move |m| m.iter_mut().for_each(func))
    };
    // use arraymap::ArrayMap;

    match cmd {
        Move(m) | MoveTo(m) | Line(m) | LineTo(m) => m.iter_mut().for_each(func),

        Horizontal(motion) | HorizontalTo(motion) | Vertical(motion) | VerticalTo(motion) => {
            func(motion)
        }

        Curve(CubicCurveMotion {
            control1,
            control2,
            point,
        })
        | CurveTo(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => {
            control1.iter_mut().for_each(func);
            control2.iter_mut().for_each(func);
            point.iter_mut().for_each(func);
        }

        Quadratic(QuadraticCurveMotion { control, point })
        | QuadraticTo(QuadraticCurveMotion { control, point }) => {
            control.iter_mut().for_each(func);
            point.iter_mut().for_each(func);
        }

        SmoothQuadratic(coords) | SmoothQuadraticTo(coords) | Smooth(coords) | SmoothTo(coords) => {
            map_coords(coords)
        }
        // SmoothQuadratic <| map_coords coords
        ClockwiseArc(arc) | AntiClockwiseArc(arc) => {
            let ArcMotion {
                x,
                y,
                radius,
                start_angle,
                end_angle,
            } = arc;

            func(x);
            func(y);
            func(radius);
            func(start_angle);
            func(end_angle);
        }

        Close => (),
    }
}

fn map_path_motion_og(func: &dyn Fn(Motion) -> Motion, cmd: PathCommandOG) -> PathCommandOG {
    use PathCommandOG::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    // let func_clone = func.clone();
    let map_coords = move |coords: Vector<[Motion; DIM2]>| -> Vector<[Motion; DIM2]> {
        coords.into_iter().map(|m| m.map(func)).collect()
    };
    // use arraymap::ArrayMap;

    match cmd {
        Move(m) => Move(m.into_iter().map(func).collect()),

        MoveTo(m) => MoveTo(m.into_iter().map(func).collect()),

        Line(m) => Line(m.into_iter().map(func).collect()),

        LineTo(m) => LineTo(m.into_iter().map(func).collect()),

        Horizontal(motion) => Horizontal(func(motion)),

        HorizontalTo(motion) => HorizontalTo(func(motion)),

        Vertical(motion) => Vertical(func(motion)),

        VerticalTo(motion) => VerticalTo(func(motion)),

        Curve(CubicCurveMotionOG {
            control1,
            control2,
            point,
        }) => Curve(CubicCurveMotionOG {
            control1: control1.into_iter().map(func).collect(),
            control2: control2.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
        CurveTo(CubicCurveMotionOG {
            control1,
            control2,
            point,
        }) => CurveTo(CubicCurveMotionOG {
            control1: control1.into_iter().map(func).collect(),
            control2: control2.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
        Quadratic(QuadraticCurveMotionOG { control, point }) => Quadratic(QuadraticCurveMotionOG {
            control: control.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
        QuadraticTo(QuadraticCurveMotionOG { control, point }) => {
            QuadraticTo(QuadraticCurveMotionOG {
                control: control.into_iter().map(func).collect(),
                point: point.into_iter().map(func).collect(),
            })
        }
        SmoothQuadratic(coords) => SmoothQuadratic(map_coords(coords)),
        // SmoothQuadratic <| map_coords coords
        SmoothQuadraticTo(coords) => SmoothQuadraticTo(map_coords(coords)),
        Smooth(coords) => Smooth(map_coords(coords)),
        SmoothTo(coords) => SmoothTo(map_coords(coords)),
        ClockwiseArc(arc) => ClockwiseArc({
            let ArcMotion {
                x,
                y,
                radius,
                start_angle,
                end_angle,
            } = arc;

            ArcMotion {
                x: func(x),
                y: func(y),
                radius: func(radius),
                start_angle: func(start_angle),
                end_angle: func(end_angle),
            }
        }),
        AntiClockwiseArc(arc) => AntiClockwiseArc({
            let ArcMotion {
                x,
                y,
                radius,
                start_angle,
                end_angle,
            } = arc;

            ArcMotion {
                x: func(x),
                y: func(y),
                radius: func(radius),
                start_angle: func(start_angle),
                end_angle: func(end_angle),
            }
        }),
        Close => Close,
    }
}

// mapToMotion : (Motion -> Motion) -> Property -> Property
pub fn map_to_motion(ref func: impl Fn(&mut Motion), prop: &mut Property) {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    match prop {
        Exact(..) => (),

        Color(_, m) => m.iter_mut().for_each(func),

        Shadow(_, _, box shadow) => {
            let ShadowMotion {
                offset_x,
                offset_y,
                size,
                blur,
                red,
                green,
                blue,
                alpha,
            } = shadow;

            func(offset_x);
            func(offset_y);
            func(size);
            func(blur);
            func(red);
            func(green);
            func(blue);
            func(alpha);
        }
        Prop(_, m) => func(m),

        Prop2(_, m) => m.iter_mut().for_each(func),

        Prop3(_, m) => m.iter_mut().for_each(func),

        Prop4(_, m) => m.iter_mut().for_each(func),

        Angle(_, m) => func(m),

        Points(ms) => ms.iter_mut().for_each(|m| m.iter_mut().for_each(func)),

        Path(cmds) => cmds
            .iter_mut()
            .for_each(|p_cmd| map_path_motion(func, p_cmd)),
    }
}

pub fn map_to_motion_og(ref func: impl Fn(Motion) -> Motion, prop: PropertyOG) -> PropertyOG {
    use PropertyOG::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    match prop {
        Exact(..) => prop,

        Color(name, m) => Color(name, m.into_iter().map(func).collect()),

        Shadow(name, inset, shadow) => {
            let ShadowMotion {
                offset_x,
                offset_y,
                size,
                blur,
                red,
                green,
                blue,
                alpha,
            } = *shadow;

            Shadow(
                name,
                inset,
                Box::new(ShadowMotion {
                    offset_x: func(offset_x),
                    offset_y: func(offset_y),
                    size: func(size),
                    blur: func(blur),
                    red: func(red),
                    green: func(green),
                    blue: func(blue),
                    alpha: func(alpha),
                }),
            )
        }
        Prop(name, m) => Prop(name, func(m)),

        Prop2(name, m) => Prop2(name, m.into_iter().map(func).collect()),

        Prop3(name, m) => Prop3(name, m.into_iter().map(func).collect()),

        Prop4(name, m) => Prop4(name, m.into_iter().map(func).collect()),

        Angle(name, m) => Angle(name, func(m)),

        Points(ms) => Points(ms.into_iter().map(|m| m.map(func)).collect()),

        Path(cmds) => Path(
            cmds.into_iter()
                .map(|p_cmd| map_path_motion_og(func, p_cmd))
                .collect(),
        ),
    }
}
fn refresh_timing(now: Duration, timing: Timing) -> Timing {
    let dt = {
        let dt_tmp = now.saturating_sub(timing.current);

        if timing.current == Duration::ZERO || dt_tmp.as_millis() > 34 {
            Duration::from_micros(16666)
        } else {
            dt_tmp
        }
    };

    Timing { current: now, dt }
}

pub fn update_animation<Message: std::clone::Clone + std::fmt::Debug>(
    Tick(now): Tick,
    model: &mut Animation<Message>,
) {
    let timing = refresh_timing(now, model.timing);

    //
    let (mut ready_interruption, queued_interruptions): (
        StepTimeVector<Message>,
        StepTimeVector<Message>,
    ) = model
        .interruption
        .clone()
        .into_iter()
        .map(|(wait, steps)| (wait.saturating_sub(timing.dt), steps))
        .partition(|(wait, _)| wait.is_zero());

    // if there is more than one matching interruptions,
    // we only take the first, which is the one that was most recently assigned.
    // If an interruption does occur, we need to clear any interpolation overrides.
    trace!("ready_interruption: {:?}", &ready_interruption);

    let mut new_props = model.props.clone();

    // TODO: check this right
    let mut new_steps = match ready_interruption.pop_front() {
        Some((_ /* is zero */, interrupt_steps)) => {
            new_props.iter_mut().for_each(|prop| {
                map_to_motion(
                    |m: &mut Motion| {
                        m.interpolation_override = None;
                    },
                    prop,
                )
            });
            interrupt_steps
        }
        None => model.steps.clone(),
    };

    let mut sent_messages = MsgBackIsNew::default();

    resolve_steps(
        &mut new_props,
        &mut new_steps,
        &mut sent_messages,
        timing.dt,
    );

    model.timing = timing;
    model.running = !new_steps.is_empty() || !queued_interruptions.is_empty();
    model.interruption = queued_interruptions;
    model.steps = new_steps;
    model.props = new_props;

    //TODO: cmd send message
}
pub fn update_animation_og<Message: std::clone::Clone + std::fmt::Debug>(
    Tick(now): Tick,
    model: &mut AnimationOG<Message>,
) {
    let timing = refresh_timing(now, model.timing);

    //
    let (mut ready_interruption, queued_interruptions): (
        StepTimeVectorOG<Message>,
        StepTimeVectorOG<Message>,
    ) = model
        .interruption
        .clone()
        .into_iter()
        .map(|(wait, steps)| (wait.saturating_sub(timing.dt), steps))
        .partition(|(wait, _)| wait.is_zero());

    // if there is more than one matching interruptions,
    // we only take the first, which is the one that was most recently assigned.
    // If an interruption does occur, we need to clear any interpolation overrides.
    trace!("ready_interruption: {:?}", &ready_interruption);
    // TODO: check this right
    let (steps, style) = {
        match ready_interruption.pop_front() {
            Some((_ /* is zero */, interrupt_steps)) => (
                interrupt_steps,
                model
                    .props
                    .clone()
                    .into_iter()
                    .map(|prop| {
                        map_to_motion_og(
                            Rc::new(|mut m: Motion| {
                                m.interpolation_override = None;
                                m
                            })
                            .as_ref(),
                            prop,
                        )
                    })
                    .collect::<Vector<_>>(),
            ),
            None => (model.steps.clone(), model.props.clone()),
        }
    };

    let (revised_style, _sent_messages, revised_steps) = resolve_steps_og(style, steps, timing.dt);

    model.timing = timing;
    model.running = !revised_steps.is_empty() || !queued_interruptions.is_empty();
    model.interruption = queued_interruptions;
    model.steps = revised_steps;
    model.props = revised_style;

    //TODO: cmd send message
}

// resolveSteps : List Property -> List (Step msg) -> Time.Posix -> ( List Property, List msg, List (Step msg) )

// resolveSteps : List Property -> List (Step msg) -> Time.Posix -> ( List Property, List msg, List (Step msg) )
#[derive(Debug, Clone, PartialEq)]
pub struct MsgBackIsNew<Message>(SmallVec<[Message; 1]>);

impl<Message> Default for MsgBackIsNew<Message> {
    fn default() -> Self {
        Self(SmallVec::new())
    }
}
impl<Message> Deref for MsgBackIsNew<Message> {
    type Target = SmallVec<[Message; 1]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Message> DerefMut for MsgBackIsNew<Message> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod resolve_steps_test {
    use std::{
        collections::{vec_deque, VecDeque},
        time::Duration,
    };

    use emg_core::{into_smvec, into_vector, vector, SmallVec, Vector};
    use seed_styles::{px, width};

    use crate::{models::resolve_steps, to, to_og, PROP_SIZE};

    use super::{resolve_steps_og, MsgBackIsNew, Property, PropertyOG, Step, StepOG};

    #[test]
    fn test_resolve_steps2() {
        let mut initial_props: Vector<PropertyOG> = into_vector![width(px(0.04))];
        let mut steps: Vector<StepOG<i32>> = vector![to_og(into_vector![width(px(0))])];

        let mut props2: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(0.04))];
        let mut steps2: VecDeque<Step<i32>> = [to(into_smvec![width(px(0))])].into();
        let mut msg = MsgBackIsNew::default();
        for i in 0..50 {
            println!("== {}", i);

            let (p, m, s) = resolve_steps_og(initial_props, steps, Duration::from_millis(16));
            resolve_steps(
                &mut props2,
                &mut steps2,
                &mut msg,
                Duration::from_millis(16),
            );
            for pp in &p {
                println!("\t{}", pp);
            }
            for p2v in &props2 {
                println!("mut--\t{}", p2v);
            }
            for (p1, p2) in (&p).iter().zip(&props2) {
                if let (PropertyOG::Prop(_, a), Property::Prop(_, b)) = (p1, p2) {
                    assert_eq!(a, b);
                }
            }
            println!("=============================");
            if s.len() == 0 {
                break;
            } else {
                initial_props = p;
                steps = s;
            }
        }
    }
    #[test]
    fn test_resolve_steps() {
        let mut initial_props: Vector<PropertyOG> = into_vector![width(px(0.05))];
        let mut steps: Vector<StepOG<i32>> = vector![to_og(into_vector![width(px(0))])];

        let mut props2: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(0.05))];
        let mut steps2: VecDeque<Step<i32>> = [to(into_smvec![width(px(0))])].into();
        let mut msg = MsgBackIsNew::default();
        for i in 0..50 {
            println!("== {}", i);

            let (p, m, s) = resolve_steps_og(initial_props, steps, Duration::from_millis(16));
            resolve_steps(
                &mut props2,
                &mut steps2,
                &mut msg,
                Duration::from_millis(16),
            );
            for pp in &p {
                println!("\t{}", pp);
            }
            for p2v in &props2 {
                println!("mut--\t{}", p2v);
            }
            for (p1, p2) in (&p).iter().zip(&props2) {
                if let (PropertyOG::Prop(_, a), Property::Prop(_, b)) = (p1, p2) {
                    assert_eq!(a, b);
                }
            }
            println!("=============================");
            if s.len() == 0 {
                break;
            } else {
                initial_props = p;
                steps = s;
            }
        }
    }
}

pub fn resolve_steps<Message>(
    current_style: &mut SmallVec<[Property; PROP_SIZE]>,
    steps: &mut VecDeque<Step<Message>>,
    msgs: &mut MsgBackIsNew<Message>,
    dt: Duration,
) where
    Message: Clone,
{
    match steps.pop_front() {
        None => (),
        Some(current_step) => match current_step {
            Step::Wait(n) => {
                if n.is_zero() {
                    resolve_steps(current_style, steps, msgs, dt)
                } else {
                    steps.push_front(Step::Wait(n.saturating_sub(dt)));
                }
            }
            Step::Send(msg) => {
                resolve_steps(current_style, steps, msgs, dt);

                msgs.push(msg);
            }
            Step::To(target) => {
                for x in &target {
                    warn!("to {x}");
                }

                let mut test_current_style = current_style.clone();
                start_towards_mut(false, &mut test_current_style, target.clone());
                step(Duration::ZERO, &mut test_current_style);
                // assert_eq!(current_style, &mut x);
                let done = test_current_style.iter().all(is_done_sm);

                if !done {
                    warn!("not done yet");
                    steps.push_front(Step::_Step);
                    start_towards_mut(false, current_style, target);

                    resolve_steps(current_style, steps, msgs, dt);
                } else {
                    warn!("step::to , done!");
                }
            }
            Step::ToWith(target) => {
                start_towards_mut(false, current_style, target.clone());
                step(Duration::ZERO, current_style);
                let done = current_style.iter().all(is_done_sm);

                if !done {
                    steps.push_front(Step::_Step);
                    // TODO ("check start_towards(true ");
                    start_towards_mut(true, current_style, target);

                    // resolve_steps_mut(start_towards_mut(true, current_style, target), steps, dt);
                    resolve_steps(current_style, steps, msgs, dt);
                }

                // if already_there(current_style.clone(), target.clone()) {
                //     (current_style, vector![], steps)
                // } else {
                //     steps.push_front(StepSM::_Step);

                //     resolve_steps(start_towards(true, current_style, target), steps, dt)
                // }
            }
            Step::Set(props) => {
                replace_props_mut(current_style, props);
                resolve_steps(current_style, steps, msgs, dt);
            }
            Step::_Step => {
                step(dt, current_style);
                if current_style.iter().all(is_done_sm) {
                    current_style.iter_mut().for_each(|prop| {
                        map_to_motion(
                            |m: &mut Motion| {
                                m.interpolation_override = None;
                            },
                            prop,
                        );
                    });
                } else {
                    steps.push_front(Step::_Step);
                }
            }
            Step::Loop(mut sub_steps) => {
                //TODO  opt : find way no clone
                let old_steps = sub_steps.clone();
                sub_steps.push_back(Step::Loop(old_steps));
                *steps = sub_steps;
                resolve_steps(current_style, steps, msgs, dt);
            }
            Step::Repeat(n, mut sub_steps) => {
                if n == 0 {
                    resolve_steps(current_style, steps, msgs, dt);
                } else {
                    let old_steps = sub_steps.clone();
                    sub_steps.push_back(Step::Repeat(n - 1, old_steps));
                    sub_steps.append(steps);
                    *steps = sub_steps;

                    resolve_steps(current_style, steps, msgs, dt);
                }
            }
        },
    }
}

#[must_use]
pub fn resolve_steps_og<Message>(
    current_style: Vector<PropertyOG>,
    mut steps: Vector<StepOG<Message>>,
    dt: Duration,
) -> (Vector<PropertyOG>, Vector<Message>, Vector<StepOG<Message>>)
where
    Message: Clone,
{
    match steps.pop_front() {
        None => (current_style, vector![], steps),
        Some(current_step) => match current_step {
            StepOG::Wait(n) => {
                if n.is_zero() {
                    resolve_steps_og(current_style, steps, dt)
                } else {
                    steps.push_front(StepOG::Wait(n.saturating_sub(dt)));
                    (current_style, vector![], steps)
                }
            }
            StepOG::Send(msg) => {
                let (new_style, mut msgs, remaining_steps) =
                    resolve_steps_og(current_style, steps, dt);

                msgs.push_front(msg);

                (new_style, msgs, remaining_steps)
            }
            StepOG::To(target) => {
                //TODO 优化, 目前 alreadyThere 内部会 start_towards 然后判断 all(is_done)

                let x = start_towards_og(false, current_style, target);
                // assert_eq!(x, current_style);
                //NOTE  px 0.05 会直接变
                let x0 = step_og(Duration::ZERO, x);
                // assert_eq!(x0, x);
                let done = x0.iter().all(is_done_og);

                // if already_there(current_style.clone(), target.clone()) {
                if done {
                    (x0, vector![], steps)
                } else {
                    steps.push_front(StepOG::_Step);
                    // for x00 in &x0 {
                    //     println!("not done= {}", &x00);
                    // }

                    resolve_steps_og(x0, steps, dt)
                }
            }
            StepOG::ToWith(target) => {
                let x = start_towards_og(false, current_style, target.clone());
                // assert_eq!(x, current_style);
                //NOTE  px 0.05 会直接变
                let x0 = step_og(Duration::ZERO, x);
                // assert_eq!(x0, x);
                let done = x0.iter().all(is_done_og);

                //TODO 优化, 目前 alreadyThere 内部会 start_towards 然后判断 all(is_done)
                if done {
                    (x0, vector![], steps)
                } else {
                    steps.push_front(StepOG::_Step);

                    resolve_steps_og(start_towards_og(true, x0, target), steps, dt)
                }
            }
            StepOG::Set(props) => resolve_steps_og(replace_props(current_style, &props), steps, dt),
            StepOG::_Step => {
                let stepped = step_og(dt, current_style);
                if stepped.iter().all(is_done_og) {
                    (
                        stepped
                            .into_iter()
                            .map(|prop| {
                                map_to_motion_og(
                                    &|mut m: Motion| {
                                        m.interpolation_override = None;
                                        m
                                    },
                                    prop,
                                )
                            })
                            .collect(),
                        vector![],
                        steps,
                    )
                } else {
                    steps.push_front(StepOG::_Step);
                    (stepped, vector![], steps)
                }
            }
            StepOG::Loop(mut sub_steps) => {
                let old_steps = sub_steps.clone();
                sub_steps.push_back(StepOG::Loop(old_steps));
                resolve_steps_og(current_style, sub_steps, dt)
            }
            StepOG::Repeat(n, mut sub_steps) => {
                if n == 0 {
                    resolve_steps_og(current_style, steps, dt)
                } else {
                    let old_steps = sub_steps.clone();
                    sub_steps.push_back(StepOG::Repeat(n - 1, old_steps));
                    sub_steps.append(steps);

                    resolve_steps_og(current_style, sub_steps, dt)
                }
            }
        },
    }
}

#[cfg(test)]
mod replace_props_test {
    use emg_core::{into_smvec, into_vector, SmallVec, Vector};
    use seed_styles::{height, px, width};

    use crate::{models::Property, PROP_SIZE};

    use super::{replace_props, replace_props_mut, PropertyOG};

    #[test]
    fn replace_props_test1() {
        let mut props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(2)), width(px(0)), width(px(1))];
        let replacements: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![height(px(99)), width(px(99)), width(px(199))];
        replace_props_mut(&mut props, replacements.clone());

        let props2: Vector<PropertyOG> = into_vector![width(px(2)), width(px(0)), width(px(1))];
        let replacements2: Vector<PropertyOG> =
            into_vector![height(px(99)), width(px(99)), width(px(199))];
        let n2 = replace_props(props2, &replacements2);

        // for i in &props {
        // println!("{}", i);
        // }
        let mut i = props.iter();
        let mut r = replacements.iter();
        assert_eq!(i.next(), r.next());
        assert_eq!(i.next(), r.next());
        assert_eq!(i.next(), r.next());
        // println!("===================");

        let mut i2 = n2.iter();
        let mut r2 = replacements2.iter();
        // for i2 in &n2 {
        //     println!("{}", i2);
        // }
        assert_eq!(i2.next(), r2.next());
        assert_eq!(i2.next(), r2.next());
        assert_eq!(i2.next(), r2.next());
    }
    #[test]
    fn replace_props_test2() {
        let mut props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![height(px(1)), width(px(2)), width(px(0)), width(px(1))];
        let replacements: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(99))];
        replace_props_mut(&mut props, replacements);

        let props2: Vector<PropertyOG> =
            into_vector![height(px(1)), width(px(2)), width(px(0)), width(px(1))];
        let replacements2: Vector<PropertyOG> = into_vector![width(px(99))];
        let n2 = replace_props(props2, &replacements2);

        let res: SmallVec<[Property; PROP_SIZE]> = into_smvec![height(px(1)), width(px(99))];
        let res2: Vector<PropertyOG> = into_vector![height(px(1)), width(px(99))];

        assert_eq!(props, res);
        assert_eq!(n2, res2);

        for i in &props {
            println!("{}", i);
        }
        println!("===================");
        for i2 in &n2 {
            println!("{}", i2);
        }
    }
}
fn replace_props_mut(
    props: &mut SmallVec<[Property; PROP_SIZE]>,
    replacements: SmallVec<[Property; PROP_SIZE]>,
) {
    //TODO deep opt use sorted name
    let replacement_names: SmallVec<[PropName; PROP_SIZE * 2]> =
        replacements.iter().map(Property::name).collect();
    // for r in &replacement_names {
    //     println!("replacement_names --:{}", r);
    // }

    props.retain(|p| {
        // println!("--:{}", &p.name());
        //TODO deep opt use sorted name
        !replacement_names.contains(&p.name())
    });
    // for p in props.iter() {
    //     println!("==== : {}", &p.name());
    // }
    props.extend(replacements);
}
fn replace_props(
    props: Vector<PropertyOG>,
    replacements: &Vector<PropertyOG>,
) -> Vector<PropertyOG> {
    let replacement_names: Vec<PropName> = replacements.iter().map(PropertyOG::name).collect();
    let removed = props
        .into_iter()
        .filter(|prop| !replacement_names.contains(&prop.name()));
    removed.chain(replacements.clone()).collect()
}
/// alreadyThere : List Property -> List Property -> Bool
// fn already_there(current: Vector<Property>, target: Vector<Property>) -> bool {
//     let x = start_towards(false, current, target);
//     step(Duration::ZERO, x).iter().all(is_done)
// }
pub type Precision = f64;
const VELOCITY_ERROR_MARGIN: Precision = 0.01;
const PROGRESS_ERROR_MARGIN: Precision = 0.005;

#[allow(clippy::match_same_arms)]
const fn position_error_margin(motion: &Motion) -> Precision {
    (match motion.unit {
        Unit::Px => 0.05,
        Unit::Pc => 0.005,
        Unit::Rem | Unit::Em | Unit::Cm | Unit::Vw | Unit::Vh | Unit::Empty => 0.001,
    }) as Precision
}
fn motion_is_done(motion: &Motion) -> bool {
    let running_interpolation = motion
        .interpolation_override
        .as_ref()
        .unwrap_or(&motion.interpolation);

    let position_error_margin = position_error_margin(motion);

    match running_interpolation {
        Interpolation::Spring { .. } => {
            (motion.velocity - 0.).abs() < VELOCITY_ERROR_MARGIN
                && (motion.position - motion.target).abs() < position_error_margin
        }
        Interpolation::Easing(eased) => {
            (eased.progress - 1.).abs() < PROGRESS_ERROR_MARGIN
                || ((eased.progress - 0.).abs() < PROGRESS_ERROR_MARGIN
                    && (motion.position - motion.target).abs() < position_error_margin)
        }
        Interpolation::AtSpeed { .. } => {
            (motion.position - motion.target).abs() < position_error_margin
        }
    }
}
fn motion_is_done_for_cmd(motion: &Motion) -> bool {
    motion.velocity == 0. && (motion.position - motion.target).abs() < position_error_margin(motion)
}
fn is_done_sm(property: &Property) -> bool {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    match property {
        Exact(..) => true,
        Prop(_, m) | Angle(_, m) => motion_is_done(m),
        Prop2(_, m) => m.iter().all(motion_is_done),
        Prop3(_, m) => m.iter().all(motion_is_done),

        Prop4(_, m) => m.iter().all(motion_is_done),
        Color(_, m) => m.iter().all(motion_is_done),
        Shadow(_, _, box shadow) => [
            &shadow.offset_x,
            &shadow.offset_y,
            &shadow.size,
            &shadow.blur,
            &shadow.red,
            &shadow.green,
            &shadow.blue,
            &shadow.alpha,
        ]
        .iter()
        .all(|&m| motion_is_done(m)),

        Points(ms) => ms
            .iter()
            .all(|[x, y]| motion_is_done(x) && motion_is_done(y)),
        Path(cmds) => cmds.iter().all(is_cmd_done_sm),
    }
}

fn is_done_og(property: &PropertyOG) -> bool {
    use PropertyOG::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    match property {
        Exact(..) => true,
        Prop(_, m) | Angle(_, m) => motion_is_done(m),
        Prop2(_, m) | Prop3(_, m) | Prop4(_, m) | Color(_, m) => m.iter().all(motion_is_done),
        Shadow(_, _, shadow) => [
            &shadow.offset_x,
            &shadow.offset_y,
            &shadow.size,
            &shadow.blur,
            &shadow.red,
            &shadow.green,
            &shadow.blue,
            &shadow.alpha,
        ]
        .iter()
        .all(|&m| motion_is_done(m)),
        Points(ms) => ms
            .iter()
            .all(|[x, y]| motion_is_done(x) && motion_is_done(y)),
        Path(cmds) => cmds.iter().all(is_cmd_done_og),
    }
}

fn is_cmd_done_sm(cmd: &PathCommand) -> bool {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    match cmd {
        Move(m) | MoveTo(m) | Line(m) | LineTo(m) => m.iter().all(motion_is_done_for_cmd),
        Horizontal(m) | HorizontalTo(m) | Vertical(m) | VerticalTo(m) => motion_is_done_for_cmd(m),
        Curve(CubicCurveMotion {
            control1,
            control2,
            point,
        })
        | CurveTo(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => {
            control1.iter().all(motion_is_done_for_cmd)
                && control2.iter().all(motion_is_done_for_cmd)
                && point.iter().all(motion_is_done_for_cmd)
        }

        Quadratic(QuadraticCurveMotion { control, point })
        | QuadraticTo(QuadraticCurveMotion { control, point }) => {
            control.iter().all(motion_is_done_for_cmd) && point.iter().all(motion_is_done_for_cmd)
        }

        SmoothQuadratic(coords) | SmoothQuadraticTo(coords) | Smooth(coords) | SmoothTo(coords) => {
            coords
                .iter()
                .all(|[x, y]| motion_is_done_for_cmd(x) && motion_is_done_for_cmd(y))
        }

        ClockwiseArc(arc) | AntiClockwiseArc(arc) => {
            motion_is_done_for_cmd(&arc.x)
                && motion_is_done_for_cmd(&arc.y)
                && motion_is_done_for_cmd(&arc.radius)
                && motion_is_done_for_cmd(&arc.start_angle)
                && motion_is_done_for_cmd(&arc.end_angle)
        }
        Close => true,
    }
}

fn is_cmd_done_og(cmd: &PathCommandOG) -> bool {
    use PathCommandOG::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    match cmd {
        Move(m) | MoveTo(m) | Line(m) | LineTo(m) => m.iter().all(motion_is_done_for_cmd),
        Horizontal(m) | HorizontalTo(m) | Vertical(m) | VerticalTo(m) => motion_is_done_for_cmd(m),
        Curve(CubicCurveMotionOG {
            control1,
            control2,
            point,
        })
        | CurveTo(CubicCurveMotionOG {
            control1,
            control2,
            point,
        }) => {
            control1.iter().all(motion_is_done_for_cmd)
                && control2.iter().all(motion_is_done_for_cmd)
                && point.iter().all(motion_is_done_for_cmd)
        }

        Quadratic(QuadraticCurveMotionOG { control, point })
        | QuadraticTo(QuadraticCurveMotionOG { control, point }) => {
            control.iter().all(motion_is_done_for_cmd) && point.iter().all(motion_is_done_for_cmd)
        }

        SmoothQuadratic(coords) | SmoothQuadraticTo(coords) => coords
            .iter()
            .all(|[x, y]| motion_is_done_for_cmd(x) && motion_is_done_for_cmd(y)),

        Smooth(coords) | SmoothTo(coords) => coords
            .iter()
            .all(|[x, y]| motion_is_done_for_cmd(x) && motion_is_done_for_cmd(y)),

        ClockwiseArc(arc) | AntiClockwiseArc(arc) => {
            motion_is_done_for_cmd(&arc.x)
                && motion_is_done_for_cmd(&arc.y)
                && motion_is_done_for_cmd(&arc.radius)
                && motion_is_done_for_cmd(&arc.start_angle)
                && motion_is_done_for_cmd(&arc.end_angle)
        }
        Close => true,
    }
}

pub fn step(dt: Duration, props: &mut SmallVec<[Property; PROP_SIZE]>) {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    props
        .iter_mut()
        // .into_iter() //TODO iter_mut
        .for_each(|property| match property {
            Exact(..) => (),
            Prop(_, motion) => step_interpolation_mut(dt, motion),
            Prop2(_, m) => m
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion)),

            Prop3(_, m) => m
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion)),

            Prop4(_, m) => m
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion)),

            Angle(_, m) => step_interpolation_mut(dt, m),
            Color(_, m) => m
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion)),
            Shadow(_, _, box shadow) => {
                step_interpolation_mut(dt, &mut shadow.offset_x);
                step_interpolation_mut(dt, &mut shadow.offset_y);
                step_interpolation_mut(dt, &mut shadow.size);
                step_interpolation_mut(dt, &mut shadow.blur);
                step_interpolation_mut(dt, &mut shadow.red);
                step_interpolation_mut(dt, &mut shadow.green);
                step_interpolation_mut(dt, &mut shadow.blue);
                step_interpolation_mut(dt, &mut shadow.alpha);
            }
            Points(points) => points.iter_mut().for_each(|[x, y]| {
                step_interpolation_mut(dt, x);
                step_interpolation_mut(dt, y);
            }),

            Path(cmds) => cmds.iter_mut().for_each(|cmd| step_path_mut(dt, cmd)),
        });
}

pub fn step_og(dt: Duration, props: Vector<PropertyOG>) -> Vector<PropertyOG> {
    use PropertyOG::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    props
        .into_iter() //TODO iter_mut
        .map(|property| match property {
            Exact(..) => property,
            Prop(name, motion) => Prop(name, step_interpolation_og(dt, motion)),
            Prop2(name, m) => Prop2(
                name,
                m.into_iter() //TODO iter_mut
                    .map(|motion| step_interpolation_og(dt, motion))
                    .collect(),
            ),
            Prop3(name, m) => Prop3(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation_og(dt, motion))
                    .collect(),
            ),
            Prop4(name, m) => Prop4(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation_og(dt, motion))
                    .collect(),
            ),
            Angle(name, m) => Angle(name, step_interpolation_og(dt, m)),
            Color(name, m) => Color(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation_og(dt, motion))
                    .collect(),
            ),
            Shadow(name, inset, shadow) => Shadow(
                name,
                inset,
                Box::new(ShadowMotion {
                    offset_x: step_interpolation_og(dt, shadow.offset_x),
                    offset_y: step_interpolation_og(dt, shadow.offset_y),
                    size: step_interpolation_og(dt, shadow.size),
                    blur: step_interpolation_og(dt, shadow.blur),
                    red: step_interpolation_og(dt, shadow.red),
                    green: step_interpolation_og(dt, shadow.green),
                    blue: step_interpolation_og(dt, shadow.blue),
                    alpha: step_interpolation_og(dt, shadow.alpha),
                }),
            ),
            Points(points) => Points(
                points
                    .into_iter()
                    .map(|[x, y]| [step_interpolation_og(dt, x), step_interpolation_og(dt, y)])
                    .collect(),
            ),
            Path(cmds) => Path(cmds.into_iter().map(|cmd| step_path_og(dt, cmd)).collect()),
        })
        .collect()
}
fn step_coords_mut(dt: Duration, coords: &mut SmallVec<[[Motion; DIM2]; MOTION_SIZE]>) {
    coords.iter_mut().for_each(|[x, y]| {
        step_interpolation_mut(dt, x);
        step_interpolation_mut(dt, y)
    });
}
// fn step_coords_mut(dt: Duration, coords: &mut Vector<[Motion; DIM2]>) {
//     coords.iter_mut().for_each(|[x, y]| {
//         step_interpolation_mut(dt, x);
//         step_interpolation_mut(dt, y)
//     });
// }

fn step_coords_og(dt: Duration, coords: Vector<[Motion; DIM2]>) -> Vector<[Motion; DIM2]> {
    coords
        .into_iter()
        .map(|[x, y]| [step_interpolation_og(dt, x), step_interpolation_og(dt, y)])
        .collect()
}
#[allow(clippy::too_many_lines)]
fn step_path_mut(dt: Duration, cmd: &mut PathCommand) {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    match cmd {
        Move(m) | MoveTo(m) | Line(m) | LineTo(m) => m
            .iter_mut()
            .for_each(|motion| step_interpolation_mut(dt, motion)),
        Horizontal(m) | HorizontalTo(m) | Vertical(m) | VerticalTo(m) => {
            step_interpolation_mut(dt, m)
        }
        Curve(CubicCurveMotion {
            control1,
            control2,
            point,
        })
        | CurveTo(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => {
            control1
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion));
            control2
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion));
            point
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion));
        }

        Quadratic(QuadraticCurveMotion { control, point })
        | QuadraticTo(QuadraticCurveMotion { control, point }) => {
            control
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion));
            point
                .iter_mut()
                .for_each(|motion| step_interpolation_mut(dt, motion));
        }

        SmoothQuadratic(coords) | SmoothQuadraticTo(coords) | Smooth(coords) | SmoothTo(coords) => {
            step_coords_mut(dt, coords)
        }
        ClockwiseArc(arc) | AntiClockwiseArc(arc) => {
            step_interpolation_mut(dt, &mut arc.x);
            step_interpolation_mut(dt, &mut arc.y);
            step_interpolation_mut(dt, &mut arc.radius);
            step_interpolation_mut(dt, &mut arc.start_angle);
            step_interpolation_mut(dt, &mut arc.end_angle);
        }

        Close => (),
    };
}
#[allow(clippy::too_many_lines)]

fn step_path_og(dt: Duration, cmd: PathCommandOG) -> PathCommandOG {
    use PathCommandOG::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    match cmd {
        Move(m) => Move(
            m.into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        ),
        MoveTo(m) => MoveTo(
            m.into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        ),
        Line(m) => Line(
            m.into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        ),
        LineTo(m) => LineTo(
            m.into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        ),
        Horizontal(m) => Horizontal(step_interpolation_og(dt, m)),
        HorizontalTo(m) => HorizontalTo(step_interpolation_og(dt, m)),
        Vertical(m) => Vertical(step_interpolation_og(dt, m)),
        VerticalTo(m) => VerticalTo(step_interpolation_og(dt, m)),
        Curve(CubicCurveMotionOG {
            control1,
            control2,
            point,
        }) => Curve(CubicCurveMotionOG {
            control1: control1
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
            control2: control2
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
            point: point
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        }),
        CurveTo(CubicCurveMotionOG {
            control1,
            control2,
            point,
        }) => CurveTo(CubicCurveMotionOG {
            control1: control1
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
            control2: control2
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),

            point: point
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        }),
        Quadratic(QuadraticCurveMotionOG { control, point }) => Quadratic(QuadraticCurveMotionOG {
            control: control
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
            point: point
                .into_iter()
                .map(|motion| step_interpolation_og(dt, motion))
                .collect(),
        }),
        QuadraticTo(QuadraticCurveMotionOG { control, point }) => {
            QuadraticTo(QuadraticCurveMotionOG {
                control: control
                    .into_iter()
                    .map(|motion| step_interpolation_og(dt, motion))
                    .collect(),
                point: point
                    .into_iter()
                    .map(|motion| step_interpolation_og(dt, motion))
                    .collect(),
            })
        }
        SmoothQuadratic(coords) => SmoothQuadratic(step_coords_og(dt, coords)),
        SmoothQuadraticTo(coords) => SmoothQuadraticTo(step_coords_og(dt, coords)),
        Smooth(coords) => Smooth(step_coords_og(dt, coords)),
        SmoothTo(coords) => SmoothTo(step_coords_og(dt, coords)),
        ClockwiseArc(arc) => ClockwiseArc(ArcMotion {
            x: (step_interpolation_og(dt, arc.x)),
            y: (step_interpolation_og(dt, arc.y)),
            radius: (step_interpolation_og(dt, arc.radius)),
            start_angle: (step_interpolation_og(dt, arc.start_angle)),
            end_angle: (step_interpolation_og(dt, arc.end_angle)),
        }),
        AntiClockwiseArc(arc) => AntiClockwiseArc(ArcMotion {
            x: (step_interpolation_og(dt, arc.x)),
            y: (step_interpolation_og(dt, arc.y)),
            radius: (step_interpolation_og(dt, arc.radius)),
            start_angle: (step_interpolation_og(dt, arc.start_angle)),
            end_angle: (step_interpolation_og(dt, arc.end_angle)),
        }),
        Close => Close,
    }
}

fn step_interpolation_mut(dt: Duration, motion: &mut Motion) {
    // let has_interpolation_override = motion.interpolation_override.is_some();
    let interpolation_to_use = motion
        .interpolation_override
        .clone()
        .unwrap_or_else(|| motion.interpolation.clone());
    match interpolation_to_use {
        Interpolation::AtSpeed { per_second } => {
            let (new_pos, finished) = {
                if motion.position < motion.target {
                    let new = per_second.mul_add(dt.as_secs_f64() as Precision, *motion.position);
                    (new, new >= *motion.target)
                } else {
                    // let new = motion.position - (per_second * (dt.as_secs_f64() as Precision));
                    let new =
                        (-*per_second).mul_add(dt.as_secs_f64() as Precision, *motion.position);
                    (new, new <= *motion.target)
                }
            };
            if finished {
                motion.position = motion.target;
                motion.velocity = NotNan::default();
            } else {
                motion.position = NotNan::new(new_pos).unwrap();
                motion.velocity = per_second * 1000.; // pos/ms,  dis per millisecond
            };
        }
        Interpolation::Spring { stiffness, damping } => {
            let dt_sec = dt.as_secs_f64() as Precision;
            let f_spring = stiffness * (motion.target - motion.position);

            let f_damper = (damping * -1.) * motion.velocity;

            let a = f_spring + f_damper;
            let new_velocity = a.mul_add(dt_sec, *motion.velocity);
            let new_pos = new_velocity.mul_add(dt_sec, *motion.position);

            let dx = (motion.target - new_pos).abs();
            if dx < position_error_margin(&motion) && new_velocity.abs() < VELOCITY_ERROR_MARGIN {
                motion.position = motion.target;
                motion.velocity = NotNan::default();
            } else {
                motion.position = NotNan::new(new_pos).unwrap();
                motion.velocity = NotNan::new(new_velocity).unwrap();
            }
        }
        Interpolation::Easing(easing) => {
            let Easing {
                progress,
                duration,
                ease,
                start,
            } = easing;

            let new_progress = (progress + dt.div_duration_f64(duration))
                .into_inner()
                .min(1.);
            // let eased = ease(new_progress);
            let eased = (**ease)(new_progress);

            let distance = motion.target - start;
            let new_pos = (eased.mul_add(*distance, *start) * 10000.).trunc() * 0.0001;
            let new_velocity = if (new_progress - 1.).abs() < PROGRESS_ERROR_MARGIN {
                0.
            } else {
                Duration::from_micros(unsafe {
                    ((new_pos - motion.position.into_inner()).abs() * 1000.)
                        .round()
                        .to_int_unchecked()
                })
                .div_duration_f64(dt) as Precision
            };

            motion.position = NotNan::new(new_pos).unwrap();
            motion.velocity = NotNan::new(new_velocity).unwrap();

            if motion.interpolation_override.is_some() {
                if let Some(Interpolation::Easing(e)) = &mut motion.interpolation_override {
                    e.progress = NotNan::new(new_progress).unwrap();
                }
            } else {
                if let Interpolation::Easing(e) = &mut motion.interpolation {
                    e.progress = NotNan::new(new_progress).unwrap();
                }
            }
        }
    }
}

fn step_interpolation_og(dt: Duration, mut motion: Motion) -> Motion {
    let has_interpolation_override = motion.interpolation_override.is_some();
    let interpolation_to_use = motion
        .interpolation_override
        .clone()
        .unwrap_or_else(|| motion.interpolation.clone());
    match interpolation_to_use {
        Interpolation::AtSpeed { per_second } => {
            let (new_pos, finished) = {
                if motion.position < motion.target {
                    let new = per_second.mul_add(dt.as_secs_f64() as Precision, *motion.position);
                    (new, new >= *motion.target)
                } else {
                    // let new = motion.position - (per_second * (dt.as_secs_f64() as Precision));
                    let new =
                        (-per_second).mul_add(dt.as_secs_f64() as Precision, *motion.position);
                    (new, new <= *motion.target)
                }
            };
            if finished {
                motion.position = motion.target;
                motion.velocity = NotNan::default();
            } else {
                motion.position = NotNan::new(new_pos).unwrap();
                motion.velocity = per_second * 1000.; // pos/ms,  dis per millisecond
            }
            motion
        }
        Interpolation::Spring { stiffness, damping } => {
            let dt_sec = dt.as_secs_f64() as Precision;
            let f_spring = stiffness * (motion.target - motion.position);

            let f_damper = (damping * -1.) * motion.velocity;

            let a = f_spring + f_damper;
            let new_velocity = a.mul_add(dt_sec, *motion.velocity);
            let new_pos = new_velocity.mul_add(dt_sec, *motion.position);

            let dx = (motion.target - new_pos).abs();
            if dx < position_error_margin(&motion) && new_velocity.abs() < VELOCITY_ERROR_MARGIN {
                // println!(
                //     "small! - dx :{} , pe:{}, v:{}",
                //     dx,
                //     position_error_margin(&motion),
                //     new_velocity.abs()
                // );
                motion.position = motion.target;
                motion.velocity = NotNan::default();
            } else {
                // println!(
                //     "big! - dx :{} , pe:{}, v:{}",
                //     dx,
                //     position_error_margin(&motion),
                //     new_velocity.abs()
                // );

                motion.position = NotNan::new(new_pos).unwrap();
                motion.velocity = NotNan::new(new_velocity).unwrap();
            }
            motion
        }
        Interpolation::Easing(easing) => {
            let Easing {
                progress,
                duration,
                ease,
                start,
            } = easing;

            let new_progress = (progress + dt.div_duration_f64(duration))
                .into_inner()
                .min(1.);
            // let eased = ease(new_progress);
            let eased = (**ease)(new_progress);

            let distance = motion.target - start;
            let new_pos = (eased.mul_add(*distance, *start) * 10000.).trunc() * 0.0001;
            let new_velocity = if (new_progress - 1.).abs() < PROGRESS_ERROR_MARGIN {
                0.
            } else {
                Duration::from_micros(unsafe {
                    ((new_pos - motion.position.into_inner()).abs() * 1000.)
                        .round()
                        .to_int_unchecked()
                })
                .div_duration_f64(dt) as Precision
            };

            motion.position = NotNan::new(new_pos).unwrap();
            motion.velocity = NotNan::new(new_velocity).unwrap();

            if has_interpolation_override {
                motion.interpolation_override = Some(Interpolation::Easing(Easing {
                    progress: NotNan::new(new_progress).unwrap(),
                    duration,
                    ease,
                    start,
                }));
            } else {
                motion.interpolation = Interpolation::Easing(Easing {
                    progress: NotNan::new(new_progress).unwrap(),
                    duration,
                    ease,
                    start,
                });
            }

            motion
        }
    }
}
///Set a new target for a style.
///If a property doesn't exist in the current style, issue a warning and do nothing with that property.
///If a property doesn't exist as a target, then leave it as is.
///Order matters (mostly for transformation stacking)
// fn start_towards_mut(
//     override_interpolation: bool,
//     current: &mut Vector<Property>,
//     target: Vector<Property>,
// ) {
//     // List.filterMap
//     //     (\propPair ->
//     //         case propPair of
//     //             ( cur, Just to ) ->
//     //                 Just <| setTarget override_interpolation cur to

//     //             ( prop, Nothing ) ->
//     //                 Just prop
//     //     )
//     //     (zip_properties_greedy current target)

//     let zipped = zip_properties_greedy(current, target);
//     zipped
//         .into_iter()
//         .map(|prop_pair| match prop_pair {
//             (cur, Some(to)) => set_target(override_interpolation, cur, to),
//             (prop, None) => prop,
//         })
//         .collect()
// }

// fn start_towards_mut(
//     override_interpolation: bool,
//     current: &mut TinyVec<[Property; PROP_SIZE]>,
//     target: Vector<Property>,
// ) -> Vector<Property> {
//     // List.filterMap
//     //     (\propPair ->
//     //         case propPair of
//     //             ( cur, Just to ) ->
//     //                 Just <| setTarget override_interpolation cur to

//     //             ( prop, Nothing ) ->
//     //                 Just prop
//     //     )
//     //     (zip_properties_greedy current target)

//     let zipped = zip_properties_greedy_mut(current, target);
//     zipped
//         .into_iter()
//         .map(|prop_pair| match prop_pair {
//             (cur, Some(to)) => set_target(override_interpolation, cur, to),
//             (prop, None) => prop,
//         })
//         .collect()
// }

//TODO work here
fn start_towards_mut(
    override_interpolation: bool,
    current: &mut SmallVec<[Property; PROP_SIZE]>,
    target: SmallVec<[Property; PROP_SIZE]>,
) {
    // List.filterMap
    //     (\propPair ->
    //         case propPair of
    //             ( cur, Just to ) ->
    //                 Just <| setTarget override_interpolation cur to

    //             ( prop, Nothing ) ->
    //                 Just prop
    //     )
    //     (zip_properties_greedy current target)

    let matched_target = zip_properties_greedy_mut(current, target);
    let zipped = current.iter_mut().zip(matched_target);
    for (a, b) in zipped {
        if let Some(to) = b {
            set_target_mut(override_interpolation, a, to);
        }
    }
}

fn start_towards_og(
    override_interpolation: bool,
    current: Vector<PropertyOG>,
    target: Vector<PropertyOG>,
) -> Vector<PropertyOG> {
    // List.filterMap
    //     (\propPair ->
    //         case propPair of
    //             ( cur, Just to ) ->
    //                 Just <| setTarget override_interpolation cur to

    //             ( prop, Nothing ) ->
    //                 Just prop
    //     )
    //     (zip_properties_greedy current target)

    let zipped = zip_properties_greedy_og(current, target);
    zipped
        .into_iter()
        .map(|prop_pair| match prop_pair {
            (cur, Some(to)) => set_target_og(override_interpolation, cur, to),
            (prop, None) => prop,
        })
        .collect()
}

#[cfg(test)]
mod tests_zip_all {
    use emg_core::{into_smvec, into_vector, SmallVec, Vector};
    use seed_styles::{height, px, width};

    use crate::{
        models::{zip_properties_greedy_og, Property},
        PROP_SIZE,
    };

    use super::zip_properties_greedy_mut;

    use super::PropertyOG;

    #[test]
    fn zip_test0() {
        let mut initial_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(2)), width(px(0)), width(px(1))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![height(px(1)), width(px(0)), width(px(1))];
        let res = zip_properties_greedy_mut(&mut initial_props, new_target_props);

        for i in &initial_props {
            println!("{}", i);
        }
        println!("===================");
        for r in &res {
            if let Some(prop) = r {
                println!("{}", prop);
            } else {
                println!("None");
            }
        }
        let mut res_zip = initial_props.iter().zip(&res);

        let match_a: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(2)), width(px(0)), width(px(1))];
        let match_b: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(0)), width(px(1))];

        assert_eq!(
            res_zip.next(),
            Some((&match_a[0], &Some(match_b[0].clone())))
        );
        assert_eq!(
            res_zip.next(),
            Some((&match_a[1], &Some(match_b[1].clone())))
        );
        assert_eq!(res_zip.next(), Some((&match_a[2], &None)));
    }
    #[test]
    fn zip_test() {
        let mut initial_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(0)), height(px(1))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(0)), height(px(1))];
        let res = zip_properties_greedy_mut(&mut initial_props, new_target_props);

        for i in initial_props {
            println!("{:#?}", i);
        }
        println!("===================");
        for r in res {
            println!("{:#?}", r.map(|x| x.name()));
        }
    }
    #[test]
    fn zip_test21() {
        let mut initial_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![height(px(0))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(0)), height(px(1))];
        let res = zip_properties_greedy_mut(&mut initial_props, new_target_props);

        for i in initial_props {
            println!("{:#?}", i.name());
        }
        println!("===================");
        for r in res {
            println!("{:#?}", r.map(|x| x.name()));
        }
    }
    #[test]
    fn zip_test2() {
        let mut initial_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(0))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(0)), height(px(1))];
        let res = zip_properties_greedy_mut(&mut initial_props, new_target_props);

        for i in initial_props {
            println!("{:#?}", i.name());
        }
        println!("===================");
        for r in res {
            println!("{:#?}", r.map(|x| x.name()));
        }
    }
    #[test]
    fn zip_test3() {
        let mut initial_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![width(px(0))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![height(px(1))];
        let res = zip_properties_greedy_mut(&mut initial_props, new_target_props);

        for i in initial_props {
            println!("{:#?}", i.name());
        }
        println!("===================");
        for r in res {
            println!("{:#?}", r.map(|x| x.name()));
        }
    }
    #[test]
    fn zip_test4() {
        let mut initial_props: SmallVec<[Property; PROP_SIZE]> =
            into_smvec![width(px(0)), height(px(1))];
        let new_target_props: SmallVec<[Property; PROP_SIZE]> = into_smvec![height(px(1))];
        let res = zip_properties_greedy_mut(&mut initial_props, new_target_props);

        for i in initial_props {
            println!("{:#?}", i.name());
        }
        println!("===================");
        for r in res {
            println!("{:#?}", r.map(|x| x.name()));
        }
    }
    #[test]
    fn zip_test_og_1() {
        let initial_props: Vector<PropertyOG> =
            into_vector![width(px(2)), width(px(0)), width(px(1))];
        let new_target_props: Vector<PropertyOG> =
            into_vector![height(px(1)), width(px(0)), width(px(1))];
        let res = zip_properties_greedy_og(initial_props, new_target_props);

        println!("===================");
        for (o, r) in res {
            println!("{}", o);
            println!("||||||||||||||||||||");

            if let Some(prop) = r {
                println!("{}", prop);
            } else {
                println!("None");
            }
        }
    }
}
/// We match two sets of properties.
/// If a property is trying to be animated but has no initial value, a warning is logged.
/// Order from the original list is preserved.

// pub fn zip_properties_greedy_mut(
//     initial_props: &mut TinyVec<[Property; PROP_SIZE]>,
//     mut new_target_props: TinyVec<[Property; PROP_SIZE]>,
pub fn zip_properties_greedy_mut(
    initial_props: &mut SmallVec<[Property; PROP_SIZE]>,
    mut new_target_props: SmallVec<[Property; PROP_SIZE]>,
) -> SmallVec<[Option<Property>; PROP_SIZE]> {
    // println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@===================");
    // for x in initial_props.iter() {
    //     println!("{}", x);
    // }
    // println!(".................................");

    // for x in new_target_props.iter() {
    //     println!("{}", x);
    // }
    // println!("---------------------------------------------------");
    initial_props.sort_by(|left, right| left.name().cmp(&right.name()));
    new_target_props.sort_by(|left, right| left.name().cmp(&right.name()));
    // for x in initial_props.iter() {
    //     println!("{}", x);
    // }
    // println!(".................................");

    // for x in new_target_props.iter() {
    //     println!("{}", x);
    // }
    // println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@===================");

    let mut res_props = smallvec![];
    let mut b_iter = new_target_props.into_iter().peekable();

    for current_a in initial_props.iter() {
        let a_name = current_a.name();

        loop {
            if let Some(current_b) = b_iter.peek() {
                if current_b.name() < a_name {
                    b_iter.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if let Some(current_b) = b_iter.peek() {
            if current_b.name() == a_name {
                res_props.push(b_iter.next());
                continue;
            }
        }
        res_props.push(None);
    }

    for b in b_iter {
        warn!(
            "{} has no initial value and therefore will not be animated.",
            b.name()
        );
    }
    res_props
}

pub fn zip_properties_greedy_og(
    initial_props: Vector<PropertyOG>,
    new_target_props: Vector<PropertyOG>,
) -> Vector<(PropertyOG, Option<PropertyOG>)> {
    let (_, warnings, props) = {
        [0, initial_props.len()].iter().fold(
            (initial_props, new_target_props, Vector::new()),
            |(mut stack_a, stack_b, mut result), _| {
                match stack_a.pop_front() {
                    Some(a) => {
                        let (mut matching_b_s, non_matching_b_s): (
                            Vector<PropertyOG>,
                            Vector<PropertyOG>,
                        ) = stack_b.into_iter().partition(|b| a.name() == b.name());
                        //
                        let b_head = matching_b_s.pop_front();
                        let new_stack_b = {
                            if b_head.is_some() {
                                matching_b_s.append(non_matching_b_s);
                                matching_b_s
                            } else {
                                non_matching_b_s
                            }
                        };
                        // let (b_head, new_stack_b) = {
                        //     match matching_b_s.as_slice() {
                        //         [b, remaining_b_s @ ..] => (
                        //             Some(*b),
                        //             [remaining_b_s, non_matching_b_s.as_slice()]
                        //                 .concat()
                        //                 .as_slice(),
                        //         ),
                        //         _ => (None, non_matching_b_s.as_slice()),
                        //     }
                        // };
                        //
                        //TODO: check use [result, [(*a, b_head)].as_ref()]  no need reverse
                        result.push_back((a, b_head));
                        // let new_result = [[(*a, b_head)].as_ref(), result].concat().as_slice();
                        (stack_a, new_stack_b, result)
                    }
                    None => (stack_a, stack_b, result),
                }

                // This is in reverse to avoid creating a new list each iteration
                // We instead have to do a reverse later.
            },
        )
    };
    for b in &warnings {
        warn!(
            "{} has no initial value and therefore will not be animated.",
            b.name()
        );
    }
    // props.reverse();
    props
}

#[allow(clippy::too_many_lines)]
fn set_target_mut(override_interpolation: bool, current: &mut Property, new_target: Property) {
    use Property::{Angle, Color, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};

    let set_motion_target = |(motion, target_motion): (&mut Motion, Motion)| {
        if override_interpolation {
            motion.interpolation_override = Some(target_motion.interpolation);
        }

        motion.target = target_motion.position;

        match motion.interpolation_override {
            None => {
                if let Interpolation::Easing(ref mut ease) = motion.interpolation {
                    ease.start = motion.position;
                    ease.progress = NotNan::default();
                }
            }
            Some(ref mut override_interpolation) => {
                if let Interpolation::Easing(ease) = override_interpolation {
                    ease.start = motion.position;
                    ease.progress = NotNan::default();
                }
            }
        }
    };

    match (current, new_target) {
        (Shadow(_, _, box shadow), Shadow(_, _, box target_shadow)) => {
            set_motion_target((&mut shadow.offset_x, target_shadow.offset_x));
            set_motion_target((&mut shadow.offset_y, target_shadow.offset_y));
            set_motion_target((&mut shadow.size, target_shadow.size));
            set_motion_target((&mut shadow.blur, target_shadow.blur));
            set_motion_target((&mut shadow.red, target_shadow.red));
            set_motion_target((&mut shadow.green, target_shadow.green));
            set_motion_target((&mut shadow.blue, target_shadow.blue));
            set_motion_target((&mut shadow.alpha, target_shadow.alpha));
        }

        (Prop(_, m), Prop(_, t)) | (Angle(_, m), Angle(_, t)) => set_motion_target((m, t)),
        (Prop2(_, box m), Prop2(_, box t)) => m.iter_mut().zip(t).for_each(set_motion_target),
        (Prop3(_, box m), Prop3(_, box t)) => m.iter_mut().zip(t).for_each(set_motion_target),
        (Prop4(_, box m), Prop4(_, box t)) => m.iter_mut().zip(t).for_each(set_motion_target),
        (Color(_, box m), Color(_, box t)) => m.iter_mut().zip(t).for_each(set_motion_target),

        (Points(box current_pts), Points(box mut target_pts)) => {
            match_points_refmut(current_pts, &mut target_pts);

            current_pts
                .iter_mut()
                .zip(target_pts)
                .for_each(|([mx, my], [tx, ty])| {
                    set_motion_target((mx, tx));
                    set_motion_target((my, ty));
                });
        }
        (Path(box cmds), Path(box targets)) => {
            cmds.iter_mut().zip(targets).for_each(set_path_target_mut)
        }
        (a, b) => panic!("{:?} \n and {:?} \n not match any set target", a, b),
    };
}

#[allow(clippy::too_many_lines)]

fn set_target_og(
    override_interpolation: bool,
    current: PropertyOG,
    new_target: PropertyOG,
) -> PropertyOG {
    use PropertyOG::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};

    let set_motion_target = |(mut motion, target_motion): (Motion, Motion)| {
        let mut new_motion = {
            if override_interpolation {
                motion.interpolation_override = Some(target_motion.interpolation);
            }
            motion
        };
        new_motion.target = target_motion.position;

        match new_motion.interpolation_override {
            None => {
                if let Interpolation::Easing(mut ease) = new_motion.interpolation {
                    ease.start = new_motion.position;
                    ease.progress = NotNan::default();
                    new_motion.interpolation = Interpolation::Easing(ease);
                }
            }
            Some(ref mut override_interpolation) => {
                if let Interpolation::Easing(ease) = override_interpolation {
                    ease.start = new_motion.position;
                    ease.progress = NotNan::default();
                    new_motion.interpolation_override = Some(Interpolation::Easing(ease.clone()));
                }
            }
        }
        new_motion
    };
    match current {
        Exact(..) => current,
        Color(ref name, ref m) => match new_target {
            Color(_, t) => Color(
                name.clone(),
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target)
                    .collect(),
            ),
            _ => current,
        },
        Shadow(ref name, inset, ref shadow) => match new_target {
            Shadow(_, _, target_shadow) => Shadow(
                name.clone(),
                inset,
                Box::new(ShadowMotion {
                    offset_x: set_motion_target((shadow.offset_x.clone(), target_shadow.offset_x)),
                    offset_y: set_motion_target((shadow.offset_y.clone(), target_shadow.offset_y)),
                    size: set_motion_target((shadow.size.clone(), target_shadow.size)),
                    blur: set_motion_target((shadow.blur.clone(), target_shadow.blur)),
                    red: set_motion_target((shadow.red.clone(), target_shadow.red)),
                    green: set_motion_target((shadow.green.clone(), target_shadow.green)),
                    blue: set_motion_target((shadow.blue.clone(), target_shadow.blue)),
                    alpha: set_motion_target((shadow.alpha.clone(), target_shadow.alpha)),
                }),
            ),
            _ => current,
        }, //TODO all like prop , no clone/ref
        Prop(name, m) => match new_target {
            Prop(_, t) => Prop(name, set_motion_target((m, t))),
            _ => Prop(name, m),
        },
        Prop2(ref name, ref m) => match new_target {
            Prop2(_, t) => Prop2(
                name.clone(),
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target)
                    .collect(),
            ),
            _ => current,
        },
        Prop3(ref name, ref m) => match new_target {
            Prop3(_, t) => Prop3(
                name.clone(),
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target)
                    .collect(),
            ),
            _ => current,
        },
        Prop4(ref name, ref m) => match new_target {
            Prop4(_, t) => Prop4(
                name.clone(),
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target)
                    .collect(),
            ),
            _ => current,
        },
        Angle(ref name, ref m) => match new_target {
            Angle(_, t) => Angle(name.clone(), set_motion_target((m.clone(), t))),
            _ => current,
        },
        Points(ref current_pts) => match new_target {
            Points(target_pts) => {
                let (m1s, m2s) = match_points(current_pts.clone(), target_pts);
                Points(
                    m1s.into_iter()
                        .zip(m2s)
                        .map(|([mx, my], [tx, ty])| {
                            [set_motion_target((mx, tx)), set_motion_target((my, ty))]
                        })
                        .collect(),
                )
            }
            _ => current,
        },
        Path(ref cmds) => match new_target {
            Path(targets) => Path(
                cmds.clone()
                    .into_iter()
                    .zip(targets)
                    .map(set_path_target_og)
                    .collect(),
            ),
            _ => current,
        },
    }
}
#[allow(clippy::too_many_lines)]
fn set_path_target_mut((cmd, target_cmd): (&mut PathCommand, PathCommand)) {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };

    let set_motion_target_in_path = |(motion, target_motion): (&mut Motion, Motion)| {
        motion.target = target_motion.position;
        if let Interpolation::Easing(ease) = &mut motion.interpolation {
            ease.start = motion.position;
            //TODO check no need motion.interpolation =xx
            // motion.interpolation = Interpolation::Easing(ease.clone());
        }
    };
    match (cmd, target_cmd) {
        (Move(m), Move(t))
        | (MoveTo(m), MoveTo(t))
        | (Line(m), Line(t))
        | (LineTo(m), LineTo(t)) => m.iter_mut().zip(t).for_each(set_motion_target_in_path),

        (Horizontal(m), Horizontal(t))
        | (HorizontalTo(m), HorizontalTo(t))
        | (Vertical(m), Vertical(t))
        | (VerticalTo(m), VerticalTo(t)) => set_motion_target_in_path((m, t)),

        (Curve(m), Curve(t)) | (CurveTo(m), CurveTo(t)) => {
            m.control1
                .iter_mut()
                .zip(t.control1)
                .for_each(set_motion_target_in_path);

            m.control2
                .iter_mut()
                .zip(t.control2)
                .for_each(set_motion_target_in_path);

            m.point
                .iter_mut()
                .zip(t.point)
                .for_each(set_motion_target_in_path);
        }

        (Quadratic(m), Quadratic(t)) | (QuadraticTo(m), QuadraticTo(t)) => {
            m.control
                .iter_mut()
                .zip(t.control)
                .for_each(set_motion_target_in_path);

            m.point
                .iter_mut()
                .zip(t.point)
                .for_each(set_motion_target_in_path);
        }

        (SmoothQuadratic(m), SmoothQuadratic(t))
        | (SmoothQuadraticTo(m), SmoothQuadraticTo(t))
        | (Smooth(m), Smooth(t))
        | (SmoothTo(m), SmoothTo(t)) => m.iter_mut().zip(t).for_each(|([mx, my], [tx, ty])| {
            set_motion_target_in_path((mx, tx));
            set_motion_target_in_path((my, ty));
        }),

        (ClockwiseArc(m), ClockwiseArc(t)) | (AntiClockwiseArc(m), AntiClockwiseArc(t)) => {
            set_motion_target_in_path((&mut m.x, t.x));
            set_motion_target_in_path((&mut m.y, t.y));
            set_motion_target_in_path((&mut m.radius, t.radius));
            set_motion_target_in_path((&mut m.start_angle, t.start_angle));
            set_motion_target_in_path((&mut m.end_angle, t.end_angle));
        }
        (Close, Close) => (),
        (a, b) => panic!("{:?} \n and {:?} \n not match any set path target", a, b),
    }
}

#[allow(clippy::too_many_lines)]

fn set_path_target_og((cmd, target_cmd): (PathCommandOG, PathCommandOG)) -> PathCommandOG {
    use PathCommandOG::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };

    let set_motion_target_in_path = |(mut motion, target_motion): (Motion, Motion)| {
        motion.target = target_motion.position;
        if let Interpolation::Easing(ease) = &mut motion.interpolation {
            ease.start = motion.position;
            //TODO check no need motion.interpolation =xx
            // motion.interpolation = Interpolation::Easing(ease.clone());
        }

        motion
    };
    match cmd {
        Move(ref m) => match target_cmd {
            Move(t) => Move(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target_in_path)
                    .collect(),
            ),
            _ => cmd,
        },
        MoveTo(ref m) => match target_cmd {
            MoveTo(t) => MoveTo(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target_in_path)
                    .collect(),
            ),
            _ => cmd,
        },
        Line(ref m) => match target_cmd {
            Line(t) => Line(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target_in_path)
                    .collect(),
            ),
            _ => cmd,
        },
        LineTo(ref m) => match target_cmd {
            LineTo(t) => LineTo(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(set_motion_target_in_path)
                    .collect(),
            ),
            _ => cmd,
        },
        Horizontal(ref m) => match target_cmd {
            Horizontal(t) => Horizontal(set_motion_target_in_path((m.clone(), t))),
            _ => cmd,
        },
        HorizontalTo(ref m) => match target_cmd {
            HorizontalTo(t) => HorizontalTo(set_motion_target_in_path((m.clone(), t))),
            _ => cmd,
        },
        Vertical(ref m) => match target_cmd {
            Vertical(t) => Vertical(set_motion_target_in_path((m.clone(), t))),
            _ => cmd,
        },
        VerticalTo(ref m) => match target_cmd {
            VerticalTo(t) => VerticalTo(set_motion_target_in_path((m.clone(), t))),
            _ => cmd,
        },
        Curve(ref m) => match target_cmd {
            Curve(t) => Curve(CubicCurveMotionOG {
                control1: m
                    .control1
                    .clone()
                    .into_iter()
                    .zip(t.control1)
                    .map(set_motion_target_in_path)
                    .collect(),
                control2: m
                    .control2
                    .clone()
                    .into_iter()
                    .zip(t.control2)
                    .map(set_motion_target_in_path)
                    .collect(),
                point: m
                    .point
                    .clone()
                    .into_iter()
                    .zip(t.point)
                    .map(set_motion_target_in_path)
                    .collect(),
            }),
            _ => cmd,
        },
        CurveTo(ref m) => match target_cmd {
            CurveTo(t) => CurveTo(CubicCurveMotionOG {
                control1: m
                    .control1
                    .clone()
                    .into_iter()
                    .zip(t.control1)
                    .map(set_motion_target_in_path)
                    .collect(),
                control2: m
                    .control2
                    .clone()
                    .into_iter()
                    .zip(t.control2)
                    .map(set_motion_target_in_path)
                    .collect(),
                point: m
                    .point
                    .clone()
                    .into_iter()
                    .zip(t.point)
                    .map(set_motion_target_in_path)
                    .collect(),
            }),
            _ => cmd,
        },
        Quadratic(ref m) => match target_cmd {
            Quadratic(t) => Quadratic(QuadraticCurveMotionOG {
                control: m
                    .control
                    .clone()
                    .into_iter()
                    .zip(t.control)
                    .map(set_motion_target_in_path)
                    .collect(),
                point: m
                    .point
                    .clone()
                    .into_iter()
                    .zip(t.point)
                    .map(set_motion_target_in_path)
                    .collect(),
            }),
            _ => cmd,
        },
        QuadraticTo(ref m) => match target_cmd {
            QuadraticTo(t) => QuadraticTo(QuadraticCurveMotionOG {
                control: m
                    .control
                    .clone()
                    .into_iter()
                    .zip(t.control)
                    .map(set_motion_target_in_path)
                    .collect(),
                point: m
                    .point
                    .clone()
                    .into_iter()
                    .zip(t.point)
                    .map(set_motion_target_in_path)
                    .collect(),
            }),
            _ => cmd,
        },
        SmoothQuadratic(ref m) => match target_cmd {
            SmoothQuadratic(t) => SmoothQuadratic(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(|([mx, my], [tx, ty])| {
                        [
                            set_motion_target_in_path((mx, tx)),
                            set_motion_target_in_path((my, ty)),
                        ]
                    })
                    .collect(),
            ),
            _ => cmd,
        },
        SmoothQuadraticTo(ref m) => match target_cmd {
            SmoothQuadraticTo(t) => SmoothQuadraticTo(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(|([mx, my], [tx, ty])| {
                        [
                            set_motion_target_in_path((mx, tx)),
                            set_motion_target_in_path((my, ty)),
                        ]
                    })
                    .collect(),
            ),
            _ => cmd,
        },
        Smooth(ref m) => match target_cmd {
            Smooth(t) => Smooth(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(|([mx, my], [tx, ty])| {
                        [
                            set_motion_target_in_path((mx, tx)),
                            set_motion_target_in_path((my, ty)),
                        ]
                    })
                    .collect(),
            ),
            _ => cmd,
        },
        SmoothTo(ref m) => match target_cmd {
            SmoothTo(t) => SmoothTo(
                m.clone()
                    .into_iter()
                    .zip(t)
                    .map(|([mx, my], [tx, ty])| {
                        [
                            set_motion_target_in_path((mx, tx)),
                            set_motion_target_in_path((my, ty)),
                        ]
                    })
                    .collect(),
            ),
            _ => cmd,
        },
        ClockwiseArc(ref ref_m) => {
            let m = ref_m.clone();
            match target_cmd {
                ClockwiseArc(t) => ClockwiseArc(ArcMotion {
                    x: set_motion_target_in_path((m.x, t.x)),
                    y: set_motion_target_in_path((m.y, t.y)),
                    radius: set_motion_target_in_path((m.radius, t.radius)),
                    start_angle: set_motion_target_in_path((m.start_angle, t.start_angle)),
                    end_angle: set_motion_target_in_path((m.end_angle, t.end_angle)),
                }),
                _ => cmd,
            }
        }
        AntiClockwiseArc(ref ref_m) => {
            let m = ref_m.clone();
            match target_cmd {
                AntiClockwiseArc(t) => AntiClockwiseArc(ArcMotion {
                    x: set_motion_target_in_path((m.x, t.x)),
                    y: set_motion_target_in_path((m.y, t.y)),
                    radius: set_motion_target_in_path((m.radius, t.radius)),
                    start_angle: set_motion_target_in_path((m.start_angle, t.start_angle)),
                    end_angle: set_motion_target_in_path((m.end_angle, t.end_angle)),
                }),
                _ => cmd,
            }
        }
        Close => Close,
    }
}

/// Ensure that two lists of points have the same number
/// of points by duplicating the last point of the smaller list.
/// matchPoints : List ( Motion, Motion ) -> List ( Motion, Motion ) -> ( List ( Motion, Motion ), List ( Motion, Motion ) )
fn match_points_refmut(
    points1: &mut SmallVec<[[Motion; DIM2]; 1]>,
    points2: &mut SmallVec<[[Motion; DIM2]; 1]>,
) {
    let ordering = points1
        .len()
        .partial_cmp(&points2.len())
        .expect("len size partial_cmp");
    match ordering {
        std::cmp::Ordering::Greater => match points2.last() {
            None => (),
            Some(last2) => {
                let diff = points1.len() - points2.len();
                let repeat_last2 = vec![last2.clone(); diff].into_iter();
                points2.extend(repeat_last2);
            }
        },
        std::cmp::Ordering::Less => match points1.last() {
            None => (),
            Some(last1) => {
                let diff = points2.len() - points1.len();
                let repeat_last1 = vec![last1.clone(); diff].into_iter();
                points1.extend(repeat_last1);
            }
        },
        std::cmp::Ordering::Equal => (),
    }
}
fn match_points(
    mut points1: Vector<[Motion; DIM2]>,
    mut points2: Vector<[Motion; DIM2]>,
) -> (Vector<[Motion; DIM2]>, Vector<[Motion; DIM2]>) {
    let ordering = points1
        .len()
        .partial_cmp(&points2.len())
        .expect("len size partial_cmp");
    match ordering {
        std::cmp::Ordering::Greater => match points2.last() {
            None => (points1, points2),
            Some(last2) => {
                let diff = points1.len() - points2.len();
                let repeat_last2 = vec![last2.clone(); diff].into_iter();
                points2.extend(repeat_last2);
                (points1, points2)
            }
        },
        std::cmp::Ordering::Less => match points1.last() {
            None => (points1, points2),
            Some(last1) => {
                let diff = points2.len() - points1.len();
                let repeat_last1 = vec![last1.clone(); diff].into_iter();
                points1.extend(repeat_last1);
                (points1, points2)
            }
        },
        std::cmp::Ordering::Equal => (points1, points2),
    }
}
