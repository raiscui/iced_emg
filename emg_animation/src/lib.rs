#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(slice_concat_ext)]
#![feature(div_duration)]
#![feature(extend_one)]
pub mod func;
pub mod models;
pub mod props;
mod render;
use std::convert::TryInto;
// ────────────────────────────────────────────────────────────────────────────────
// use emg_debuggable::dbg4;
use std::{f64::consts::PI, fmt, rc::Rc, time::Duration};

use im::{vector, Vector};
use models::{map_to_motion, update_animation, Animation, Interpolation, Property, Step};
use ordered_float::NotNan;
use props::warn_for_double_listed_properties;
use seed_styles::Unit;

// ────────────────────────────────────────────────────────────────────────────────
pub use crate::models::color::fill;
pub use crate::models::opacity::opacity;
pub use crate::models::Tick;
pub use crate::models::Timing;
// ────────────────────────────────────────────────────────────────────────────────
use crate::models::{Easing, Motion};

#[allow(clippy::enum_glob_use)]
use crate::models::Interpolation::*;
// ────────────────────────────────────────────────────────────────────────────────

pub type Msg = models::Tick;
pub type State<Message> = models::Animation<Message>;

fn init_motion(position: impl TryInto<NotNan<f64>>, unit: Unit) -> Motion {
    let p: NotNan<f64> = position.try_into().ok().unwrap();
    Motion {
        position: p,
        velocity: NotNan::default(),
        target: p,
        interpolation: Spring {
            stiffness: NotNan::new(170.).unwrap(),
            damping: NotNan::new(26.).unwrap(),
        },
        unit,
        interpolation_override: None,
    }
}

// initialState : List Animation.Model.Property -> Animation msg
#[must_use]
pub fn initial_state<Message>(current: Vector<Property>) -> Animation<Message>
where
    Message: Clone,
{
    Animation {
        steps: vector![],
        props: current,
        timing: Timing {
            current: Duration::ZERO,
            dt: Duration::ZERO,
        },
        running: false,
        interruption: vector![],
    }
}

// speed : { perSecond : Float } -> Animation.Model.Interpolation
fn speed(speed_value: f64) -> Interpolation {
    AtSpeed {
        per_second: NotNan::new(speed_value).unwrap(),
    }
}

/// # Panics
///
/// Will panic if number is nan
#[must_use]
pub fn default_interpolation_by_property(prop: &Property) -> Interpolation {
    use Property::{Angle, Color, Exact, Path, Points, Prop, Prop2, Prop3, Prop4, Shadow};
    // -- progress is set to 1 because it is changed to 0 when the animation actually starts
    // -- This is analogous to the spring starting at rest.
    let linear = |duration: Duration| {
        Easing(Easing {
            progress: 1.,
            start: NotNan::default(),
            duration,
            ease: Rc::new(dbg4!(Box::new(std::convert::identity::<f64>))),
        })
    };

    let default_spring = Spring {
        stiffness: NotNan::new(170.).unwrap(),
        damping: NotNan::new(26.).unwrap(),
    };

    match prop {
        Exact(..) | Shadow(..) | Prop(..) | Prop2(..) | Prop4(..) | Points(..) | Path(..) => {
            default_spring
        }

        Color(..) => linear(Duration::from_millis(400)),

        Prop3(name, ..) => {
            if name.as_str() == "rotate3d" {
                speed(PI)
            } else {
                default_spring
            }
        }

        Angle(_, _) => speed(PI),
    }
}

// setDefaultInterpolation : Animation.Model.Property -> Animation.Model.Property
#[must_use]
pub fn set_default_interpolation(prop: Property) -> Property {
    let interp = default_interpolation_by_property(&prop);

    map_to_motion(
        &move |mut m: Motion| -> Motion {
            m.interpolation = interp.clone();
            m
        },
        prop,
    )
}

// style : List Animation.Model.Property -> Animation msg
#[must_use]
pub fn style<Message>(props: Vector<Property>) -> Animation<Message>
where
    Message: Clone,
{
    //
    warn_for_double_listed_properties(&props);
    initial_state(
        props
            .into_iter()
            .map(set_default_interpolation)
            .collect::<Vector<Property>>(),
    )
}

// ────────────────────────────────────────────────────────────────────────────────

///Sums all leading `Wait` steps and removes them from the animation.
///This is used because the wait at the start of an interruption works differently than a normal wait.

//    extractInitialWait : List (Animation.Model.Step msg) -> ( Time.Posix, List (Animation.Model.Step msg) )
#[must_use]
pub fn extract_initial_wait<Message>(
    steps: Vector<Step<Message>>,
) -> (Duration, Vector<Step<Message>>)
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
    let front = steps.front().cloned();
    match front {
        None => (Duration::ZERO, steps),
        Some(step) => {
            if let Wait(till) = step {
                let (additional_time, remaining_steps) = extract_initial_wait(steps.skip(1));
                (till + additional_time, remaining_steps)
            } else {
                (Duration::ZERO, steps)
            }
        } // [step] => (Duration::ZERO, steps),
    }
}

///Interrupt any running animations with the following animation.
// interrupt : List (Animation.Model.Step msg) -> Animation msg -> Animation msg
pub fn interrupt<Message>(
    steps: Vector<Step<Message>>,
    model: &mut Animation<Message>,
) -> &mut Animation<Message>
where
    Message: Clone,
{
    model.interruption.push_front(extract_initial_wait(steps));
    model.running = true;
    model
}

#[must_use]
pub fn to<Message>(props: Vector<Property>) -> Step<Message>
where
    Message: Clone,
{
    Step::To(props)
}

// custom : String -> Float -> String -> Animation.Model.Property
fn custom(name: String, value: f64, unit: Unit) -> Property {
    Property::Prop(Rc::new(name), init_motion(value, unit))
}

/// Update an animation.
//TODO : Implement this
pub fn update<Message: std::clone::Clone + std::fmt::Debug>(
    tick: Tick,
    animation: &mut Animation<Message>,
) {
    update_animation(tick, animation);
}
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct Debuggable<T> {
    text: &'static str,
    value: T,
}

impl<T> PartialEq for Debuggable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.text.eq(other.text)
    }
}

impl<T> std::ops::Deref for Debuggable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Produce a Debuggable<T> from an expression for T
#[macro_export]
macro_rules! dbg4 {
    ($($body:tt)+) => {
        Debuggable {
            text: stringify!($($body)+),
            value: $($body)+,
        }
    };
}

// Note: this type is unsized

impl<T> fmt::Debug for Debuggable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
// ────────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use im::vector;

    use crate::{
        extract_initial_wait, fill, interrupt,
        models::{color::Color, opacity::opacity, update_animation, Step},
        style, to, State, Tick,
    };

    #[derive(Clone, Debug)]
    enum Message {
        A,
        B,
    }
    #[test]
    fn it_works() {
        let styles: State<Message> = style(vector![fill(Color::new(0, 0, 0, 1.))]);
        println!("{:#?}", styles);
    }
    #[test]
    fn test_extract_initial_wait() {
        let xx = vector![
            Step::Wait(Duration::from_millis(16)),
            Step::_Step,
            Step::Send(Message::A),
        ];
        println!("{:#?}", xx);
        let ff = extract_initial_wait(vector![
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
    fn test_update_animation() {
        let mut am_state: State<Message> = style(vector![opacity(1.)]);
        insta::assert_debug_snapshot!("init", &am_state);

        interrupt(
            vector![to(vector![opacity(0.)]), to(vector![opacity(1.)])],
            &mut am_state,
        );
        insta::assert_debug_snapshot!("interrupt", &am_state);

        let mut now = Duration::from_millis(10000);
        update_animation(Tick(now), &mut am_state);
        insta::assert_debug_snapshot!("am1-first", &am_state);

        now += Duration::from_millis(16);
        update_animation(Tick(now), &mut am_state);
        insta::assert_debug_snapshot!("am2", &am_state);

        now += Duration::from_millis(17);
        update_animation(Tick(now), &mut am_state);
        insta::assert_debug_snapshot!("am3", &am_state);

        for _ in 0..180 {
            now += Duration::from_millis(17);
            update_animation(Tick(now), &mut am_state);
        }
        println!("{:#?}", &am_state);
        insta::assert_debug_snapshot!("am_last", &am_state);
    }
    #[test]
    fn test_interrupt() {
        let mut am_state: State<Message> = style(vector![opacity(1.)]);
        let interrupt1 = interrupt(
            vector![to(vector![opacity(0.)]), to(vector![opacity(1.)])],
            &mut am_state,
        );
        println!("{:#?}", interrupt1);
    }
}
