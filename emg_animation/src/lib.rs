#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(slice_concat_ext)]
#![feature(div_duration)]
#![feature(extend_one)]
// #![feature(array_map)]
#![feature(box_patterns)]
// #![feature(adt_const_params)]
// #![feature(generic_const_exprs)]
#![feature(slice_take)]
#![allow(clippy::used_underscore_binding)]

pub mod func;
pub mod models;
pub mod props;
mod render;
use std::collections::VecDeque;
// ────────────────────────────────────────────────────────────────────────────────
// use emg_debuggable::dbg4;
pub use emg_common;
use emg_common::{animation::Tick, vector, SmallVec};
use emg_common::{
    num_traits::{cast, AsPrimitive, NumCast},
    Precision,
};
use models::update_animation;
use models::{map_to_motion, Animation, Interpolation, Property, Step};
use ordered_float::NotNan;
use props::warn_for_double_listed_properties;
use seed_styles::Unit;
use std::{f64::consts::PI, fmt, rc::Rc, time::Duration};
pub const PROP_SIZE: usize = 1;
pub const STEP_SIZE: usize = 3;
pub const MOTION_SIZE: usize = 3;
// ────────────────────────────────────────────────────────────────────────────────
pub use crate::models::opacity::opacity;
pub use crate::models::Timing;
// ────────────────────────────────────────────────────────────────────────────────
use crate::models::{Easing, Motion};

#[allow(clippy::enum_glob_use)]
use crate::models::Interpolation::*;
// ────────────────────────────────────────────────────────────────────────────────

pub type Msg = Tick;
pub type AmState<Message> = models::Animation<Message>;

/// # Panics
///
/// Will panic if 'position' is NaN
pub fn init_motion(position: impl NumCast, unit: Unit) -> Motion
// where
    // T: TryInto<NotNan<Precision>>,
{
    let p: NotNan<Precision> = NotNan::new(cast(position).unwrap()).unwrap();

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
pub fn initial_state<Message>(current: SmallVec<[Property; PROP_SIZE]>) -> Animation<Message>
where
    Message: Clone,
{
    Animation {
        steps: VecDeque::new(),
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
fn speed(speed_value: Precision) -> Interpolation {
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
            progress: NotNan::new(1.).unwrap(),
            start: NotNan::default(),
            duration,
            ease: Rc::new(dbg4!(Box::new(std::convert::identity::<Precision>))),
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
                speed(PI.as_())
            } else {
                default_spring
            }
        }

        Angle(_, _) => speed(PI.as_()),
    }
}

pub fn set_default_interpolation(prop: &mut Property) {
    let i = default_interpolation_by_property(prop);

    map_to_motion(
        &|m: &mut Motion| {
            m.interpolation = i.clone();
        },
        prop,
    );
}

// style : List Animation.Model.Property -> Animation msg

#[must_use]
pub fn style<Message>(mut props: SmallVec<[Property; PROP_SIZE]>) -> Animation<Message>
where
    Message: Clone,
{
    //
    warn_for_double_listed_properties(&props);
    props.iter_mut().for_each(set_default_interpolation);
    initial_state(props)
}

// ────────────────────────────────────────────────────────────────────────────────

///Sums all leading `Wait` steps and removes them from the animation.
///This is used because the wait at the start of an interruption works differently than a normal wait.

//    extractInitialWait : List (Animation.Model.Step msg) -> ( Time.Posix, List (Animation.Model.Step msg) )

/// # Panics
///
/// Never panic
#[must_use]
pub fn extract_initial_wait<Message>(
    mut steps: VecDeque<Step<Message>>,
) -> (Duration, VecDeque<Step<Message>>)
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

    if let Some(Step::Wait(dt)) = steps.front().cloned() {
        let (additional_time, remaining_steps) = extract_initial_wait(steps.split_off(1));
        (dt + additional_time, remaining_steps)
    } else {
        (Duration::ZERO, steps)
    }

    // use Step::Wait;
    // let front = steps.front();
    // if let Some(Wait(_)) = front {
    //     let till = steps
    //         .pop_front()
    //         .and_then(|x| x.try_into_wait().ok())
    //         .unwrap();
    //     let (additional_time, remaining_steps) = extract_initial_wait(steps);
    //     (till + additional_time, remaining_steps)
    // } else {
    //     (Duration::ZERO, steps)
    // }
}

///Interrupt any running animations with the following animation.
// interrupt : List (Animation.Model.Step msg) -> Animation msg -> Animation msg

pub fn interrupt<Message>(
    steps: impl Into<VecDeque<Step<Message>>>,
    model: &mut Animation<Message>,
) -> &mut Animation<Message>
where
    Message: Clone,
{
    model
        .interruption
        .push_front(extract_initial_wait(steps.into()));
    model.running = true;
    model
}
pub fn replace<Message>(
    steps: impl Into<VecDeque<Step<Message>>>,
    model: &mut Animation<Message>,
) -> &mut Animation<Message>
where
    Message: Clone,
{
    model.interruption = vector![extract_initial_wait(steps.into())];
    model.running = true;
    model
}

// {-| Repeat a number of steps until interrupted.
//     -}
//     loop : List (Animation.Model.Step msg) -> Animation.Model.Step msg
//     loop steps =
//         Loop steps
pub fn loop_am<Message>(steps: impl Into<VecDeque<Step<Message>>>) -> Step<Message>
where
    Message: Clone,
{
    Step::Loop(steps.into())
}

#[must_use]
pub const fn to<Message>(props: SmallVec<[Property; PROP_SIZE]>) -> Step<Message>
where
    Message: Clone,
{
    Step::To(props)
}
#[macro_export]
macro_rules! to {

    ( $( $element:expr ) , * ) => {
        {
            $crate::models::Step::To( $crate::emg_common::smallvec![ $( $element.into() ),*])

        }
    };
}

// custom : String -> Float -> String -> Animation.Model.Property
fn custom(name: &str, value: f64, unit: Unit) -> Property {
    Property::Prop(name.into(), init_motion(value, unit))
}

/// Update an animation.
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
impl<T> Eq for Debuggable<T> {}
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
    use std::{collections::VecDeque, time::Duration};

    use emg_common::{animation::Tick, smallvec, vector};

    use crate::{
        extract_initial_wait, interrupt,
        models::{
            color::{fill, Color},
            update_animation, Step,
        },
        opacity, style, AmState,
    };

    #[derive(Clone, Debug)]
    enum Message {
        A,
        B,
    }
    #[test]
    fn it_works() {
        let styles: AmState<Message> = style(smallvec![fill(Color::new(0, 0, 0, 1.))]);
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
        let ff = extract_initial_wait(VecDeque::from([
            Step::Wait(Duration::from_millis(16)),
            Step::_Step,
            Step::Send(Message::A),
        ]));
        println!("{:#?}", &ff);
        let v = (
            Duration::from_millis(16),
            vec![Step::_Step, Step::Send(Message::A)],
        );
        assert_eq!(format!("{:?}", v), format!("{:?}", ff))
    }
    #[test]
    fn test_update_animation() {
        let mut am_state: AmState<Message> = style(smallvec![opacity(1.)]);
        insta::assert_debug_snapshot!("init", &am_state);

        interrupt([to![opacity(0.)], to![opacity(1.)]], &mut am_state);
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
}
