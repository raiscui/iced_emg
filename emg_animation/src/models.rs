pub mod color;
pub mod opacity;
use std::{rc::Rc, time::Duration};

use im::Vector;

use crate::Debuggable;

/*
 * @Author: Rais
 * @Date: 2021-05-07 19:19:51
 * @LastEditTime: 2021-05-12 19:32:47
 * @LastEditors: Rais
 * @Description:
 */
// 1秒(s) ＝1000毫秒(ms)
// 1毫秒(ms)＝1000微秒(us)
// 1微秒 micro (us)＝1000纳秒(ns)
#[derive(Clone, Debug)]
pub(crate) enum Interpolation {
    Spring {
        stiffness: f64,
        damping: f64,
    },
    Easing {
        progress: f64,
        duration: Duration,
        start: f64,
        ease: Rc<Debuggable<dyn Fn(f64) -> f64>>,
    },
    AtSpeed {
        per_second: f64,
    },
}

#[derive(Clone, Debug)]
pub struct Motion {
    pub(crate) position: f64,
    pub(crate) velocity: f64,
    pub(crate) target: f64,
    pub(crate) interpolation: Interpolation,
    pub(crate) unit: String,
    pub(crate) interpolation_override: Option<Interpolation>,
}

#[derive(Clone, Debug)]
pub struct CubicCurveMotion {
    control1: (Motion, Motion),
    control2: (Motion, Motion),
    point: (Motion, Motion),
}

#[derive(Debug, Clone)]
pub struct QuadraticCurveMotion {
    control: (Motion, Motion),
    point: (Motion, Motion),
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
    Move(Motion, Motion),
    MoveTo(Motion, Motion),
    Line(Motion, Motion),
    LineTo(Motion, Motion),
    Horizontal(Motion),
    HorizontalTo(Motion),
    Vertical(Motion),
    VerticalTo(Motion),
    Curve(CubicCurveMotion),
    CurveTo(CubicCurveMotion),
    Quadratic(QuadraticCurveMotion),
    QuadraticTo(QuadraticCurveMotion),
    SmoothQuadratic(Vec<(Motion, Motion)>),
    SmoothQuadraticTo(Vec<(Motion, Motion)>),
    Smooth(Vec<(Motion, Motion)>),
    SmoothTo(Vec<(Motion, Motion)>),
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
    Exact(String, String),
    Color(String, Motion, Motion, Motion, Motion),
    Shadow(String, bool, Box<ShadowMotion>),
    Prop(String, Motion),
    Prop2(String, Motion, Motion),
    Prop3(String, Motion, Motion, Motion),
    Prop4(String, Motion, Motion, Motion, Motion),
    Angle(String, Motion),
    Points(Vec<(Motion, Motion)>),
    Path(Vec<PathCommand>),
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
pub enum Step<Message> {
    _Step,
    To(Vec<Property>),
    ToWith(Vec<Property>),
    Set(Vec<Property>),
    Wait(Duration),
    Send(Message),
    Repeat(u32, Vec<Step<Message>>),
    Loop(Vec<Step<Message>>),
}

#[derive(Debug)]
pub struct Timing {
    pub(crate) current: Duration,
    pub(crate) dt: Duration,
}

#[derive(Debug)]
pub struct Animation<Message>
where
    Message: Clone,
{
    pub(crate) steps: Vec<Step<Message>>,
    pub(crate) style: Vec<Property>,
    pub(crate) timing: Timing,
    pub(crate) running: bool,
    pub(crate) interruption: Vector<(Duration, Vec<Step<Message>>)>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tick(pub Duration);

impl Tick {
    #[must_use]
    pub const fn subsec_millis(&self) -> u32 {
        self.0.subsec_millis()
    }
    #[must_use]
    pub fn new(millisecond: f64) -> Self {
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
    let map_coords = move |coords: Vec<(Motion, Motion)>| -> Vec<(Motion, Motion)> {
        // let func_clone2 = func_clone.clone();

        coords
            .into_iter()
            .map(|(x, y)| (func(x), func(y)))
            .collect()

        // List.map
        //     (\( x, y ) ->
        //         ( fn x
        //         , fn y
        //         )
        //     )
        //     coords
    };

    match cmd {
        Move(m1, m2) => Move(func(m1), func(m2)),

        MoveTo(m1, m2) => MoveTo(func(m1), func(m2)),

        Line(m1, m2) => Line(func(m1), func(m2)),

        LineTo(m1, m2) => LineTo(func(m1), func(m2)),

        Horizontal(motion) => Horizontal(func(motion)),

        HorizontalTo(motion) => HorizontalTo(func(motion)),

        Vertical(motion) => Vertical(func(motion)),

        VerticalTo(motion) => VerticalTo(func(motion)),

        Curve(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => Curve(CubicCurveMotion {
            control1: (func(control1.0), func(control1.1)),
            control2: (func(control2.0), func(control2.1)),
            point: (func(point.0), func(point.1)),
        }),
        CurveTo(CubicCurveMotion {
            control1,
            control2,
            point,
        }) => CurveTo(CubicCurveMotion {
            control1: (func(control1.0), func(control1.1)),
            control2: (func(control2.0), func(control2.1)),
            point: (func(point.0), func(point.1)),
        }),
        Quadratic(QuadraticCurveMotion { control, point }) => Quadratic(QuadraticCurveMotion {
            control: (func(control.0), func(control.1)),
            point: (func(point.0), func(point.1)),
        }),
        QuadraticTo(QuadraticCurveMotion { control, point }) => QuadraticTo(QuadraticCurveMotion {
            control: (func(control.0), func(control.1)),
            point: (func(point.0), func(point.1)),
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
        Exact(name, value) => Exact(name, value),

        Color(name, m1, m2, m3, m4) => Color(name, func(m1), func(m2), func(m3), func(m4)),

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
        Prop(name, m1) => Prop(name, func(m1)),

        Prop2(name, m1, m2) => Prop2(name, func(m1), func(m2)),

        Prop3(name, m1, m2, m3) => Prop3(name, func(m1), func(m2), func(m3)),

        Prop4(name, m1, m2, m3, m4) => Prop4(name, func(m1), func(m2), func(m3), func(m4)),

        Angle(name, m1) => Angle(name, func(m1)),

        Points(ms) => Points(ms.into_iter().map(|(x, y)| (func(x), func(y))).collect()),
        // Points <|
        //     List.map
        //         (\( x, y ) =>
        //             ( func x
        //             , func y
        //             )
        //         )
        //         ms
        Path(cmds) => Path(
            cmds.into_iter()
                .map(|p_cmd| map_path_motion(func, p_cmd))
                .collect(),
        ), // Path <|
           //     List.map
           //         (mapPathMotion func)
           //         cmds
           //
    }
}
