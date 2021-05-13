#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]

// ────────────────────────────────────────────────────────────────────────────────

mod models;
mod render;
// ────────────────────────────────────────────────────────────────────────────────

use std::{f64::consts::PI, fmt, rc::Rc, time::Duration};

use im::vector;
use models::{map_to_motion, Animation, Interpolation, Property, Step};
use render::warn_for_double_listed_properties;

// ────────────────────────────────────────────────────────────────────────────────
pub use crate::models::color::fill;
pub use crate::models::Tick;
pub use crate::models::Timing;
// ────────────────────────────────────────────────────────────────────────────────
use crate::models::{CubicCurveMotion, Motion, QuadraticCurveMotion, ShadowMotion};

#[allow(clippy::enum_glob_use)]
use crate::models::Interpolation::*;
// ────────────────────────────────────────────────────────────────────────────────

pub type Msg = models::Tick;
pub type State<Message> = models::Animation<Message>;

const fn init_motion(position: f64, unit: String) -> Motion {
    Motion {
        position,
        velocity: 0.,
        target: position,
        interpolation: Spring {
            stiffness: 170.,
            damping: 26.,
        },
        unit,
        interpolation_override: None,
    }
}

// initialState : List Animation.Model.Property -> Animation msg
fn initial_state<Message>(current: Vec<Property>) -> Animation<Message>
where
    Message: Clone,
{
    Animation {
        steps: vec![],
        style: current,
        timing: Timing {
            current: Duration::ZERO,
            dt: Duration::ZERO,
        },
        running: false,
        interruption: vector![],
    }
}

fn identity<T>(x: T) -> T {
    x
}

// speed : { perSecond : Float } -> Animation.Model.Interpolation
fn speed(speed_value: f64) -> Interpolation {
    AtSpeed {
        per_second: speed_value,
    }
}

// defaultInterpolationByProperty : Animation.Model.Property -> Animation.Model.Interpolation
fn default_interpolation_by_property(prop: Property) -> Interpolation {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    // -- progress is set to 1 because it is changed to 0 when the animation actually starts
    // -- This is analagous to the spring starting at rest.
    let linear = |duration: Duration| Easing {
        progress: 1.,
        start: 0.,
        duration,
        ease: Rc::new(dbg2!(identity::<f64>)),
    };

    let default_spring = Spring {
        stiffness: 170.,
        damping: 26.,
    };

    match prop {
        Exact(_, _)
        | Shadow(_, _, _)
        | Prop(_, _)
        | Prop2(_, _, _)
        | Prop4(_, _, _, _, _)
        | Points(_)
        | Path(_) => default_spring,

        Color(_, _, _, _, _) => linear(Duration::from_millis(400)),

        Prop3(name, _, _, _) => {
            if name == "rotate3d" {
                speed(PI)
            } else {
                default_spring
            }
        }

        Angle(_, _) => speed(PI),
    }
}

// setDefaultInterpolation : Animation.Model.Property -> Animation.Model.Property
fn set_default_interpolation(prop: Property) -> Property {
    let interp = default_interpolation_by_property(prop.clone());

    map_to_motion(
        Rc::new(move |mut m: Motion| -> Motion {
            m.interpolation = interp.clone();
            m
        })
        .as_ref(),
        prop,
    )
}

// style : List Animation.Model.Property -> Animation msg
#[must_use]
pub fn style<Message>(props: &[Property]) -> Animation<Message>
where
    Message: Clone,
{
    //
    warn_for_double_listed_properties(props);
    let props = props.to_vec();
    initial_state(props.into_iter().map(set_default_interpolation).collect())
}

// ────────────────────────────────────────────────────────────────────────────────

///Sums all leading `Wait` steps and removes them from the animation.
///This is used because the wait at the start of an interruption works differently than a normal wait.

//    extractInitialWait : List (Animation.Model.Step msg) -> ( Time.Posix, List (Animation.Model.Step msg) )
fn extract_initial_wait<Message>(steps: &[Step<Message>]) -> (Duration, Vec<Step<Message>>)
where
    Message: Clone,
{
    // case List.head steps of
    // Nothing ->
    //     ( Time.millisToPosix 0, [] )

    // Just step ->
    //     case step of
    //         Wait till ->
    //             let
    //                 ( additionalTime, remainingSteps ) =
    //                     extractInitialWait (List.drop 1 steps)
    //             in
    //             ( Time.millisToPosix (Time.posixToMillis till + Time.posixToMillis additionalTime), remainingSteps )

    //         _ ->
    //             ( Time.millisToPosix 0, steps )
    use Step::Wait;
    match steps {
        [] => (Duration::ZERO, vec![]),
        [step, tail @ ..] => {
            if let Wait(till) = step {
                let (additional_time, remaining_steps) = extract_initial_wait(tail);
                return (*till + additional_time, remaining_steps);
            }
            (Duration::ZERO, steps.to_vec())
        } // [step] => (Duration::ZERO, steps),
    }
}

///Interrupt any running animations with the following animation.
// interrupt : List (Animation.Model.Step msg) -> Animation msg -> Animation msg
fn interrupt<'a, Message>(
    steps: &[Step<Message>],
    model: &'a mut Animation<Message>,
) -> &'a mut Animation<Message>
where
    Message: Clone,
{
    model.interruption.push_front(extract_initial_wait(steps));
    model.running = true;
    model
}

fn to<Message>(props: &[Property]) -> Step<Message> {
    Step::To(props.to_vec())
}

// custom : String -> Float -> String -> Animation.Model.Property
fn custom(name: String, value: f64, unit: String) -> Property {
    Property::Prop(name, init_motion(value, unit))
}
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

/// Extends a (possibly unsized) value with a Debug string.
// (This type is unsized when T is unsized)
#[derive(Clone)]
pub struct Debuggable<T: ?Sized> {
    text: &'static str,
    value: T,
}

impl<T: ?Sized> std::ops::Deref for Debuggable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Produce a Debuggable<T> from an expression for T
#[macro_export]
macro_rules! dbg2 {
    ($($body:tt)+) => {
        Debuggable {
            text: stringify!($($body)+),
            value: $($body)+,
        }
    };
}

// Note: this type is unsized

impl<T: ?Sized> fmt::Debug for Debuggable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        extract_initial_wait, fill, interrupt,
        models::{color::Color, opacity::opacity, Step},
        style, to, State,
    };

    #[derive(Clone, Debug)]
    enum Message {
        A,
        B,
    }
    #[test]
    fn it_works() {
        let styles: State<Message> = style(&[fill(Color::new(0, 0, 0, 1.))]);
        println!("{:#?}", styles);
    }
    #[test]
    fn test_extract_initial_wait() {
        let ff = extract_initial_wait(&[
            Step::Wait(Duration::from_millis(16)),
            Step::_Step,
            Step::Send(Message::A),
        ]);
        println!("{:#?}", &ff);
        let v = (
            Duration::from_millis(16),
            vec![Step::_Step, Step::Send(Message::A)],
        );
        assert_eq!(format!("{:?}", v), format!("{:?}", ff))
    }
    #[test]
    fn test_interrupt() {
        let mut am_state: State<Message> = style(&[opacity(1.)]);
        let interrupt1 = interrupt(&[to(&[opacity(0.)]), to(&[opacity(1.)])], &mut am_state);
        println!("{:#?}", interrupt1);
    }
}
