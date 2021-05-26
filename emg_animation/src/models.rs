pub mod color;
pub mod opacity;
use im::{vector, Vector};
use iter_fixed::IntoIteratorFixed;
use std::{rc::Rc, time::Duration};
use tracing::{trace, warn};

use crate::Debuggable;

// use emg_debuggable::{dbg4, Debuggable};

#[derive(Clone, Debug)]
pub struct Easing {
    pub progress: Precision,
    pub duration: Duration,
    pub start: Precision,
    pub ease: Rc<Debuggable<dyn Fn(Precision) -> Precision>>,
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
#[derive(Clone, Debug)]
pub enum Interpolation {
    Spring {
        stiffness: Precision,
        damping: Precision,
    },
    Easing(Easing),
    AtSpeed {
        per_second: Precision,
    },
}

#[derive(Clone, Debug)]
pub struct Motion {
    pub(crate) position: Precision,
    pub(crate) velocity: Precision,
    pub(crate) target: Precision,
    pub(crate) interpolation: Interpolation,
    pub(crate) unit: String,
    pub(crate) interpolation_override: Option<Interpolation>,
}

#[derive(Clone, Debug)]
pub struct CubicCurveMotion {
    control1: Vector<Motion>,
    control2: Vector<Motion>,
    point: Vector<Motion>,
}

#[derive(Debug, Clone)]
pub struct QuadraticCurveMotion {
    control: Vector<Motion>,
    point: Vector<Motion>,
}

#[derive(Debug, Clone)]
pub struct ArcMotion {
    x: Motion,
    y: Motion,
    radius: Motion,
    start_angle: Motion,
    end_angle: Motion,
}

#[derive(Clone, Debug)]
pub enum PathCommand {
    Move(Vector<Motion>),
    MoveTo(Vector<Motion>),
    Line(Vector<Motion>),
    LineTo(Vector<Motion>),
    Horizontal(Motion),
    HorizontalTo(Motion),
    Vertical(Motion),
    VerticalTo(Motion),
    Curve(CubicCurveMotion),
    CurveTo(CubicCurveMotion),
    Quadratic(QuadraticCurveMotion),
    QuadraticTo(QuadraticCurveMotion),
    SmoothQuadratic(Vector<[Motion; 2]>),
    SmoothQuadraticTo(Vector<[Motion; 2]>),
    Smooth(Vector<[Motion; 2]>),
    SmoothTo(Vector<[Motion; 2]>),
    ClockwiseArc(ArcMotion),
    AntiClockwiseArc(ArcMotion),
    Close,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum Property {
    Exact(Rc<String>, String),
    Color(Rc<String>, Vector<Motion>),
    Shadow(Rc<String>, bool, Box<ShadowMotion>),
    Prop(Rc<String>, Motion),
    Prop2(Rc<String>, Vector<Motion>),
    Prop3(Rc<String>, Vector<Motion>),
    Prop4(Rc<String>, Vector<Motion>),
    Angle(Rc<String>, Motion),
    Points(Vector<[Motion; 2]>),
    Path(Vector<PathCommand>),
}
// propertyName : Property -> String
pub fn property_name(prop: &Property) -> &str {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};

    match prop {
        Exact(name, ..)
        | Color(name, ..)
        | Shadow(name, ..)
        | Prop(name, ..)
        | Prop2(name, ..)
        | Prop3(name, ..)
        | Prop4(name, ..)
        | Angle(name, ..) => name,

        Points(_) => "points",

        Path(_) => "path",
    }
}

#[allow(clippy::pub_enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Step<Message>
where
    Message: Clone,
{
    _Step,
    To(Vector<Property>),
    ToWith(Vector<Property>),
    Set(Vector<Property>),
    Wait(Duration),
    Send(Message),
    Repeat(u32, Vector<Step<Message>>),
    Loop(Vector<Step<Message>>),
}

#[derive(Copy, Clone, Debug)]
pub struct Timing {
    pub(crate) current: Duration,
    pub(crate) dt: Duration,
}

type StepTimeVector<Message> = Vector<(Duration, Vector<Step<Message>>)>;

#[derive(Debug)]
pub struct Animation<Message>
where
    Message: Clone,
    // (Duration, Vec<Step<Message>>): Clone,
{
    pub(crate) steps: Vector<Step<Message>>,
    pub(crate) style: Vector<Property>,
    pub(crate) timing: Timing,
    pub(crate) running: bool,
    pub(crate) interruption: StepTimeVector<Message>,
}

impl<Message> Animation<Message>
where
    Message: Clone,
{
    pub fn get_position(&self, style_i: usize) -> Precision {
        let p = self.style.get(style_i).unwrap();
        match p {
            Property::Prop(_name, m) => m.position,
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
fn map_path_motion(func: &dyn Fn(Motion) -> Motion, cmd: PathCommand) -> PathCommand {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    // let func_clone = func.clone();
    let map_coords = move |coords: Vector<[Motion; 2]>| -> Vector<[Motion; 2]> {
        coords
            .into_iter()
            .map(|m| m.into_iter_fixed().map(func).collect())
            .collect()
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

        Curve(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => Curve(CubicCurveMotion {
            control1: control1.into_iter().map(func).collect(),
            control2: control2.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
        CurveTo(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => CurveTo(CubicCurveMotion {
            control1: control1.into_iter().map(func).collect(),
            control2: control2.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
        Quadratic(QuadraticCurveMotion { control, point }) => Quadratic(QuadraticCurveMotion {
            control: control.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
        QuadraticTo(QuadraticCurveMotion { control, point }) => QuadraticTo(QuadraticCurveMotion {
            control: control.into_iter().map(func).collect(),
            point: point.into_iter().map(func).collect(),
        }),
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
pub fn map_to_motion(func: &dyn Fn(Motion) -> Motion, prop: Property) -> Property {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
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

        Points(ms) => Points(
            ms.into_iter()
                .map(|m| m.into_iter_fixed().map(func).collect())
                .collect(),
        ),

        Path(cmds) => Path(
            cmds.into_iter()
                .map(|p_cmd| map_path_motion(func, p_cmd))
                .collect(),
        ),
    }
}
fn refresh_timing(now: Duration, timing: Timing) -> Timing {
    let dt = {
        let dt_tmp = now - timing.current;
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
    // TODO: check this right
    let (steps, style) = {
        match ready_interruption.pop_front() {
            Some((_ /* is zero */, interrupt_steps)) => (
                interrupt_steps,
                model
                    .style
                    .clone()
                    .into_iter()
                    .map(|prop| {
                        map_to_motion(
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
            None => (model.steps.clone(), model.style.clone()),
        }
    };

    let (revised_style, _sent_messages, revised_steps) = resolve_steps(style, steps, timing.dt);

    model.timing = timing;
    model.running = !revised_steps.is_empty() || !queued_interruptions.is_empty();
    model.interruption = queued_interruptions;
    model.steps = revised_steps;
    model.style = revised_style

    //TODO: cmd send message
}

// resolveSteps : List Property -> List (Step msg) -> Time.Posix -> ( List Property, List msg, List (Step msg) )
fn resolve_steps<Message>(
    current_style: Vector<Property>,
    mut steps: Vector<Step<Message>>,
    dt: Duration,
) -> (Vector<Property>, Vector<Message>, Vector<Step<Message>>)
where
    Message: Clone,
{
    match steps.pop_front() {
        None => (current_style, vector![], steps),
        Some(current_step) => match current_step {
            Step::Wait(n) => {
                if n.is_zero() {
                    resolve_steps(current_style, steps, dt)
                } else {
                    steps.push_front(Step::Wait(n.saturating_sub(dt)));
                    (current_style, vector![], steps)
                }
            }
            Step::Send(msg) => {
                let (new_style, mut msgs, remaining_steps) =
                    resolve_steps(current_style, steps, dt);

                msgs.push_front(msg);

                (new_style, msgs, remaining_steps)
            }
            Step::To(target) => {
                //TODO 优化, 目前 alreadyThere 内部会 start_towards 然后判断 all(is_done)
                if already_there(current_style.clone(), target.clone()) {
                    (current_style, vector![], steps)
                } else {
                    steps.push_front(Step::_Step);

                    resolve_steps(start_towards(false, current_style, target), steps, dt)
                }
            }
            Step::ToWith(target) => {
                //TODO 优化, 目前 alreadyThere 内部会 start_towards 然后判断 all(is_done)
                if already_there(current_style.clone(), target.clone()) {
                    (current_style, vector![], steps)
                } else {
                    steps.push_front(Step::_Step);

                    resolve_steps(start_towards(true, current_style, target), steps, dt)
                }
            }
            Step::Set(props) => resolve_steps(replace_props(current_style, &props), steps, dt),
            Step::_Step => {
                let stepped = step(dt, current_style);
                if stepped.iter().all(is_done) {
                    (
                        stepped
                            .into_iter()
                            .map(|prop| {
                                map_to_motion(
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
                    steps.push_front(Step::_Step);
                    (stepped, vector![], steps)
                }
            }
            Step::Loop(mut sub_steps) => {
                let old_steps = sub_steps.clone();
                sub_steps.push_back(Step::Loop(old_steps));
                resolve_steps(current_style, sub_steps, dt)
            }
            Step::Repeat(n, mut sub_steps) => {
                if n == 0 {
                    resolve_steps(current_style, steps, dt)
                } else {
                    let old_steps = sub_steps.clone();
                    sub_steps.push_back(Step::Repeat(n - 1, old_steps));
                    sub_steps.append(steps);

                    resolve_steps(current_style, sub_steps, dt)
                }
            }
        },
    }
}
fn replace_props(props: Vector<Property>, replacements: &Vector<Property>) -> Vector<Property> {
    let replacement_names: Vec<&str> = replacements.iter().map(property_name).collect();
    let removed = props
        .into_iter()
        .filter(|prop| replacement_names.contains(&property_name(prop)));
    removed.chain(replacements.clone()).collect()
}
/// alreadyThere : List Property -> List Property -> Bool
fn already_there(current: Vector<Property>, target: Vector<Property>) -> bool {
    let x = start_towards(false, current, target);
    step(Duration::ZERO, x).iter().all(is_done)
}
type Precision = f64;
const VELOCITY_ERROR_MARGIN: Precision = 0.01;
const PROGRESS_ERROR_MARGIN: Precision = 0.005;

#[allow(clippy::match_same_arms)]
fn position_error_margin(motion: &Motion) -> Precision {
    (match motion.unit.as_str() {
        "px" => 0.05,
        "%" => 0.005,
        _ => 0.001,
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
fn is_done(property: &Property) -> bool {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
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
        Path(cmds) => cmds.iter().all(is_cmd_done),
    }
}

fn is_cmd_done(cmd: &PathCommand) -> bool {
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
fn step(dt: Duration, props: Vector<Property>) -> Vector<Property> {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    props
        .into_iter()
        .map(|property| match property {
            Exact(..) => property,
            Prop(name, motion) => Prop(name, step_interpolation(dt, motion)),
            Prop2(name, m) => Prop2(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation(dt, motion))
                    .collect(),
            ),
            Prop3(name, m) => Prop3(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation(dt, motion))
                    .collect(),
            ),
            Prop4(name, m) => Prop4(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation(dt, motion))
                    .collect(),
            ),
            Angle(name, m) => Angle(name, step_interpolation(dt, m)),
            Color(name, m) => Color(
                name,
                m.into_iter()
                    .map(|motion| step_interpolation(dt, motion))
                    .collect(),
            ),
            Shadow(name, inset, shadow) => Shadow(
                name,
                inset,
                Box::new(ShadowMotion {
                    offset_x: step_interpolation(dt, shadow.offset_x),
                    offset_y: step_interpolation(dt, shadow.offset_y),
                    size: step_interpolation(dt, shadow.size),
                    blur: step_interpolation(dt, shadow.blur),
                    red: step_interpolation(dt, shadow.red),
                    green: step_interpolation(dt, shadow.green),
                    blue: step_interpolation(dt, shadow.blue),
                    alpha: step_interpolation(dt, shadow.alpha),
                }),
            ),
            Points(points) => Points(
                points
                    .into_iter()
                    .map(|[x, y]| [step_interpolation(dt, x), step_interpolation(dt, y)])
                    .collect(),
            ),
            Path(cmds) => Path(cmds.into_iter().map(|cmd| step_path(dt, cmd)).collect()),
        })
        .collect()
}
fn step_coords(dt: Duration, coords: Vector<[Motion; 2]>) -> Vector<[Motion; 2]> {
    coords
        .into_iter()
        .map(|[x, y]| [step_interpolation(dt, x), step_interpolation(dt, y)])
        .collect()
}
#[allow(clippy::too_many_lines)]
fn step_path(dt: Duration, cmd: PathCommand) -> PathCommand {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };
    match cmd {
        Move(m) => Move(
            m.into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        ),
        MoveTo(m) => MoveTo(
            m.into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        ),
        Line(m) => Line(
            m.into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        ),
        LineTo(m) => LineTo(
            m.into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        ),
        Horizontal(m) => Horizontal(step_interpolation(dt, m)),
        HorizontalTo(m) => HorizontalTo(step_interpolation(dt, m)),
        Vertical(m) => Vertical(step_interpolation(dt, m)),
        VerticalTo(m) => VerticalTo(step_interpolation(dt, m)),
        Curve(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => Curve(CubicCurveMotion {
            control1: control1
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
            control2: control2
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
            point: point
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        }),
        CurveTo(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => CurveTo(CubicCurveMotion {
            control1: control1
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
            control2: control2
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),

            point: point
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        }),
        Quadratic(QuadraticCurveMotion { control, point }) => Quadratic(QuadraticCurveMotion {
            control: control
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
            point: point
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        }),
        QuadraticTo(QuadraticCurveMotion { control, point }) => QuadraticTo(QuadraticCurveMotion {
            control: control
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
            point: point
                .into_iter()
                .map(|motion| step_interpolation(dt, motion))
                .collect(),
        }),
        SmoothQuadratic(coords) => SmoothQuadratic(step_coords(dt, coords)),
        SmoothQuadraticTo(coords) => SmoothQuadraticTo(step_coords(dt, coords)),
        Smooth(coords) => Smooth(step_coords(dt, coords)),
        SmoothTo(coords) => SmoothTo(step_coords(dt, coords)),
        ClockwiseArc(arc) => ClockwiseArc(ArcMotion {
            x: (step_interpolation(dt, arc.x)),
            y: (step_interpolation(dt, arc.y)),
            radius: (step_interpolation(dt, arc.radius)),
            start_angle: (step_interpolation(dt, arc.start_angle)),
            end_angle: (step_interpolation(dt, arc.end_angle)),
        }),
        AntiClockwiseArc(arc) => AntiClockwiseArc(ArcMotion {
            x: (step_interpolation(dt, arc.x)),
            y: (step_interpolation(dt, arc.y)),
            radius: (step_interpolation(dt, arc.radius)),
            start_angle: (step_interpolation(dt, arc.start_angle)),
            end_angle: (step_interpolation(dt, arc.end_angle)),
        }),
        Close => Close,
    }
}

fn step_interpolation(dt: Duration, mut motion: Motion) -> Motion {
    let has_interpolation_override = motion.interpolation_override.is_some();
    let interpolation_to_use = motion
        .interpolation_override
        .clone()
        .unwrap_or_else(|| motion.interpolation.clone());
    match interpolation_to_use {
        Interpolation::AtSpeed { per_second } => {
            let (new_pos, finished) = {
                if motion.position < motion.target {
                    let new = per_second.mul_add(dt.as_secs_f64() as Precision, motion.position);
                    (new, new >= motion.target)
                } else {
                    // let new = motion.position - (per_second * (dt.as_secs_f64() as Precision));
                    let new = (-per_second).mul_add(dt.as_secs_f64() as Precision, motion.position);
                    (new, new <= motion.target)
                }
            };
            if finished {
                motion.position = motion.target;
                motion.velocity = 0.;
            } else {
                motion.position = new_pos;
                motion.velocity = per_second * 1000.; // pos/ms,  dis per millisecond
            }
            motion
        }
        Interpolation::Spring { stiffness, damping } => {
            let dt_sec = dt.as_secs_f64() as Precision;
            let f_spring = stiffness * (motion.target - motion.position);

            let f_damper = (-1. * damping) * motion.velocity;

            let a = f_spring + f_damper;
            let new_velocity = a.mul_add(dt_sec, motion.velocity);
            let new_pos = new_velocity.mul_add(dt_sec, motion.position);

            let dx = (motion.target - new_pos).abs();
            if dx < position_error_margin(&motion) && new_velocity.abs() < VELOCITY_ERROR_MARGIN {
                motion.position = motion.target;
                motion.velocity = 0.;
            } else {
                motion.position = new_pos;
                motion.velocity = new_velocity;
            }
            motion
        }
        Interpolation::Easing(Easing {
            progress,
            duration,
            ease,
            start,
        }) => {
            let new_progress =
                (dt.div_duration_f64(duration) + (progress as f64)).min(1.) as Precision;
            let eased = ease(new_progress);

            let distance = motion.target - start;
            let new_pos = (eased.mul_add(distance, start) * 10000.).trunc() * 0.0001;
            let new_velocity = if (new_progress - 1.).abs() < PROGRESS_ERROR_MARGIN {
                0.
            } else {
                Duration::from_micros(unsafe {
                    ((new_pos - motion.position).abs() * 1000.)
                        .round()
                        .to_int_unchecked()
                })
                .div_duration_f64(dt) as Precision
            };

            motion.position = new_pos;
            motion.velocity = new_velocity;

            if has_interpolation_override {
                motion.interpolation_override = Some(Interpolation::Easing(Easing {
                    progress: new_progress,
                    duration,
                    ease,
                    start,
                }));
            } else {
                motion.interpolation = Interpolation::Easing(Easing {
                    progress: new_progress,
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
fn start_towards(
    override_interpolation: bool,
    current: Vector<Property>,
    target: Vector<Property>,
) -> Vector<Property> {
    // List.filterMap
    //     (\propPair ->
    //         case propPair of
    //             ( cur, Just to ) ->
    //                 Just <| setTarget override_interpolation cur to

    //             ( prop, Nothing ) ->
    //                 Just prop
    //     )
    //     (zip_properties_greedy current target)

    let zipped = zip_properties_greedy(current, target);
    zipped
        .into_iter()
        .map(|prop_pair| match prop_pair {
            (cur, Some(to)) => set_target(override_interpolation, cur, to),
            (prop, None) => prop,
        })
        .collect()
}
/// We match two sets of properties.
/// If a property is trying to be animated but has no initial value, a warning is logged.
/// Order from the original list is preserved.
fn zip_properties_greedy(
    initial_props: Vector<Property>,
    new_target_props: Vector<Property>,
) -> Vector<(Property, Option<Property>)> {
    let (_, warnings, props) = {
        [0, initial_props.len()].iter().fold(
            (initial_props, new_target_props, Vector::new()),
            |(mut stack_a, stack_b, mut result), _| {
                match stack_a.pop_front() {
                    Some(a) => {
                        let (mut matching_b_s, non_matching_b_s): (
                            Vector<Property>,
                            Vector<Property>,
                        ) = stack_b
                            .into_iter()
                            .partition(|b| property_name(&a) == property_name(b));
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
            property_name(b)
        );
    }
    // props.reverse();
    props
}

#[allow(clippy::too_many_lines)]
fn set_target(override_interpolation: bool, current: Property, new_target: Property) -> Property {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};

    let set_motion_target = |(mut motion, target_motion): (Motion, Motion)| {
        let mut new_motion = {
            if override_interpolation {
                motion.interpolation_override = Some(target_motion.interpolation);
            }
            motion
        };
        match new_motion.interpolation_override {
            None => {
                if let Interpolation::Easing(mut ease) = new_motion.interpolation {
                    new_motion.target = target_motion.position;
                    ease.start = new_motion.position;
                    ease.progress = 0.;
                    new_motion.interpolation = Interpolation::Easing(ease);
                    new_motion
                } else {
                    new_motion.target = target_motion.position;
                    new_motion
                }
            }
            Some(ref mut override_interpolation) => {
                if let Interpolation::Easing(ease) = override_interpolation {
                    new_motion.target = target_motion.position;
                    ease.start = new_motion.position;
                    ease.progress = 0.;
                    new_motion.interpolation_override = Some(Interpolation::Easing(ease.clone()));
                    new_motion
                } else {
                    new_motion.target = target_motion.position;
                    new_motion
                }
            }
        }
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
        },
        Prop(ref name, ref m) => match new_target {
            Prop(_, t) => Prop(name.clone(), set_motion_target((m.clone(), t))),
            _ => current,
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
                    .map(set_path_target)
                    .collect(),
            ),
            _ => current,
        },
    }
}
#[allow(clippy::too_many_lines)]
fn set_path_target((cmd, target_cmd): (PathCommand, PathCommand)) -> PathCommand {
    use PathCommand::{
        AntiClockwiseArc, ClockwiseArc, Close, Curve, CurveTo, Horizontal, HorizontalTo, Line,
        LineTo, Move, MoveTo, Quadratic, QuadraticTo, Smooth, SmoothQuadratic, SmoothQuadraticTo,
        SmoothTo, Vertical, VerticalTo,
    };

    let set_motion_target_in_path = |(mut motion, target_motion): (Motion, Motion)| {
        if let Interpolation::Easing(ease) = &mut motion.interpolation {
            motion.target = target_motion.position;
            ease.start = motion.position;
            motion.interpolation = Interpolation::Easing(ease.clone());
            motion
        } else {
            motion.target = target_motion.position;
            motion
        }
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
            Curve(t) => Curve(CubicCurveMotion {
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
            CurveTo(t) => CurveTo(CubicCurveMotion {
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
            Quadratic(t) => Quadratic(QuadraticCurveMotion {
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
            QuadraticTo(t) => QuadraticTo(QuadraticCurveMotion {
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
fn match_points(
    mut points1: Vector<[Motion; 2]>,
    mut points2: Vector<[Motion; 2]>,
) -> (Vector<[Motion; 2]>, Vector<[Motion; 2]>) {
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
