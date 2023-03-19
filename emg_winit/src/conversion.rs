/*
 * @Author: Rais
 * @Date: 2022-08-11 18:19:27
 * @LastEditTime: 2023-03-17 18:44:02
 * @LastEditors: Rais
 * @Description:
 */

//TODO use shaping instead of
//! Convert [`winit`] types into [`iced_native`] types, and viceversa.
//!
//! [`winit`]: https://github.com/rust-windowing/winit
//! [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
use emg_common::{na, smallvec, Affine, SmallVec};
use emg_native::{
    drag,
    event::{EventFlag, EventIdentify, EventWithFlagType},
};
use tracing::{debug, debug_span};

use crate::keyboard;
use crate::mouse;
use crate::touch;
use crate::window;
use crate::{Event, Mode, Pos, SemanticPosition};

pub mod ev {
    use emg_common::Affine;

    use emg_common;

    use crate::Pos;

    #[derive(Default)]
    pub struct EventState {
        prior_mouse_down: bool,
        mouse_down: bool,
        pub(crate) prior_position: Option<Pos>, //logic
        //NOTE 非增量
        pub(crate) transform: Affine, //logic
    }

    impl EventState {
        pub fn set_mouse_down(&mut self, mouse_down: bool) {
            if mouse_down {
                println!("----按下");
            }

            self.prior_mouse_down = self.mouse_down;
            self.mouse_down = mouse_down;
        }

        pub fn mouse_down_info(&self) -> (bool, bool) {
            (self.prior_mouse_down, self.mouse_down)
        }
    }
}

/// Converts a winit window event into an iced event.
pub fn window_event(
    event: winit::event::WindowEvent<'_>,
    scale_factor: f64,
    modifiers: winit::event::ModifiersState,
    event_state: &mut ev::EventState,
) -> Option<SmallVec<[EventWithFlagType; 2]>> {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::Resized(new_size) => {
            let logical_size = new_size.to_logical(scale_factor);

            Some(smallvec![(
                EventIdentify::new(EventFlag::WINDOW, window::RESIZED),
                Event::Window(window::Event::Resized {
                    width: logical_size.width,
                    height: logical_size.height,
                }),
            )])
        }
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            let logical_size = new_inner_size.to_logical(scale_factor);

            Some(smallvec![(
                EventIdentify::new(EventFlag::WINDOW, window::RESIZED),
                Event::Window(window::Event::Resized {
                    width: logical_size.width,
                    height: logical_size.height,
                }),
            )])
        }
        WindowEvent::CloseRequested => Some(smallvec![(
            EventIdentify::new(EventFlag::WINDOW, window::CLOSE_REQUESTED),
            Event::Window(window::Event::CloseRequested),
        )]),
        WindowEvent::CursorMoved { position, .. } => {
            let position = position.to_logical::<f32>(scale_factor);
            let position = Pos::new(position.x, position.y);
            let (prior_mouse_down, mouse_down) = event_state.mouse_down_info();

            let mut evs = SmallVec::new();

            let mut moved = false;

            //TODO move to event_state function
            if mouse_down {
                // let pos_is_same = event_state.prior_position.is_some_and(|pp| pp == position);
                // let _span = debug_span!("DRAG",pos_is_same, %position ).entered();

                if let Some(prior) = event_state.prior_position {
                    if prior == position {
                        //按压 静止
                        // if !prior_mouse_down {
                        //     let _span = debug_span!("DRAG", "-----start").entered();

                        //     evs.push((
                        //         EventIdentify::new(EventFlag::DND, drag::DRAG_START),
                        //         Event::DragDrop(drag::Event::DragStart { position }),
                        //     ));
                        //     event_state.set_mouse_down(true); // 持续更改 ,prior_mouse_down 变更
                        //     println!("----按下 1");
                        // }
                    } else {
                        moved = true;

                        let offset = na::Translation2::<f32>::from(position - prior);
                        event_state.transform = offset * event_state.transform;

                        if !prior_mouse_down {
                            //fast move , first

                            evs.push((
                                EventIdentify::new(EventFlag::DND, drag::DRAG_START),
                                Event::DragDrop(drag::Event::DragStart { prior, position }),
                            ));

                            event_state.set_mouse_down(true); // 持续更改 ,prior_mouse_down 变更
                            println!("----按下 2");
                        }

                        evs.push((
                            EventIdentify::new(EventFlag::DND, drag::DRAG),
                            Event::DragDrop(drag::Event::Drag(drag::Drag {
                                prior,
                                position,
                                trans: event_state.transform,
                                offset: na::convert(offset),
                            })),
                        ));
                    }
                }
            } else {
                moved = event_state.prior_position.is_none()
                    || event_state.prior_position.contains(&position);
            }
            event_state.prior_position = Some(position);

            if moved {
                evs.push((
                    EventIdentify::new(EventFlag::MOUSE, mouse::CURSOR_MOVED),
                    Event::Mouse(mouse::Event::CursorMoved { position }),
                ));
                Some(evs)
            } else {
                None
            }
        }
        WindowEvent::CursorEntered { .. } => Some(smallvec![(
            EventIdentify::new(EventFlag::MOUSE, mouse::CURSOR_ENTERED),
            Event::Mouse(mouse::Event::CursorEntered),
        )]),
        WindowEvent::CursorLeft { .. } => {
            //TODO move to event_state function

            event_state.prior_position = None;

            Some(smallvec![(
                EventIdentify::new(EventFlag::MOUSE, mouse::CURSOR_LEFT),
                Event::Mouse(mouse::Event::CursorLeft),
            )])
        }
        WindowEvent::MouseInput { button, state, .. } => {
            let button = mouse_button(button);
            //TODO 当前使用 flag 也使用 event::ButtonPressed 等, 可能不需要同时使用
            Some(match (button, state) {
                (mouse::Button::Left, winit::event::ElementState::Pressed) => {
                    //TODO move to event_state function

                    event_state.set_mouse_down(true);
                    println!("----按下 3");

                    event_state.transform = Default::default();

                    smallvec![(
                        EventIdentify::new(EventFlag::MOUSE, mouse::LEFT_PRESSED),
                        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    )]
                }
                (mouse::Button::Left, winit::event::ElementState::Released) => {
                    //TODO move to event_state function
                    let (pm, _m) = event_state.mouse_down_info();
                    event_state.set_mouse_down(false);
                    event_state.transform = Default::default();
                    if !pm {
                        //按下就释放
                        println!("按下就释放");
                        smallvec![(
                            EventIdentify::new(EventFlag::MOUSE, mouse::LEFT_RELEASED),
                            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                        )]
                    } else {
                        println!("按下+移动过..");

                        smallvec![
                            (
                                EventIdentify::new(EventFlag::DND, drag::DRAG_END),
                                Event::DragDrop(drag::Event::DragEnd),
                            ),
                            (
                                EventIdentify::new(EventFlag::MOUSE, mouse::LEFT_RELEASED),
                                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                            )
                        ]
                    }
                }
                (mouse::Button::Right, winit::event::ElementState::Pressed) => smallvec![(
                    EventIdentify::new(EventFlag::MOUSE, mouse::RIGHT_PRESSED),
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
                )],
                (mouse::Button::Right, winit::event::ElementState::Released) => smallvec![(
                    EventIdentify::new(EventFlag::MOUSE, mouse::RIGHT_RELEASED),
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)),
                )],
                (mouse::Button::Middle, winit::event::ElementState::Pressed) => smallvec![(
                    EventIdentify::new(EventFlag::MOUSE, mouse::MIDDLE_PRESSED),
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)),
                )],
                (mouse::Button::Middle, winit::event::ElementState::Released) => smallvec![(
                    EventIdentify::new(EventFlag::MOUSE, mouse::MIDDLE_RELEASED),
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)),
                )],
                (mouse::Button::Other(x), winit::event::ElementState::Pressed) => smallvec![(
                    EventIdentify::new(EventFlag::MOUSE, mouse::OTHER_PRESSED),
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Other(x))),
                )],
                (mouse::Button::Other(x), winit::event::ElementState::Released) => smallvec![(
                    EventIdentify::new(EventFlag::MOUSE, mouse::OTHER_RELEASED),
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Other(x))),
                )],
            })
        }
        WindowEvent::MouseWheel { delta, .. } => match delta {
            winit::event::MouseScrollDelta::LineDelta(delta_x, delta_y) => Some(smallvec![(
                EventIdentify::new(EventFlag::MOUSE, mouse::WHEEL_SCROLLED),
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines {
                        x: delta_x,
                        y: delta_y,
                    },
                }),
            )]),
            winit::event::MouseScrollDelta::PixelDelta(position) => Some(smallvec![(
                EventIdentify::new(EventFlag::MOUSE, mouse::WHEEL_SCROLLED),
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels {
                        x: position.x as f32,
                        y: position.y as f32,
                    },
                }),
            )]),
        },
        WindowEvent::ReceivedCharacter(c) if !is_private_use_character(c) => Some(smallvec![(
            EventIdentify::new(EventFlag::KEYBOARD, keyboard::CHARACTER_RECEIVED),
            Event::Keyboard(keyboard::Event::CharacterReceived(c)),
        )]),
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(virtual_keycode),
                    state,
                    ..
                },
            ..
        } => {
            let key_code = key_code(virtual_keycode);
            let modifiers = self::modifiers(modifiers);

            match state {
                winit::event::ElementState::Pressed => Some(smallvec![(
                    EventIdentify::new(EventFlag::KEYBOARD, keyboard::KEY_PRESSED),
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code,
                        modifiers,
                    }),
                )]),
                winit::event::ElementState::Released => Some(smallvec![(
                    EventIdentify::new(EventFlag::KEYBOARD, keyboard::KEY_RELEASED),
                    Event::Keyboard(keyboard::Event::KeyReleased {
                        key_code,
                        modifiers,
                    }),
                )]),
            }
        }
        WindowEvent::ModifiersChanged(new_modifiers) => Some(smallvec![(
            EventIdentify::new(EventFlag::KEYBOARD, keyboard::MODIFIERS_CHANGED),
            Event::Keyboard(keyboard::Event::ModifiersChanged(self::modifiers(
                new_modifiers,
            ))),
        )]),
        WindowEvent::Focused(focused) => Some(smallvec![if focused {
            (
                EventIdentify::new(EventFlag::WINDOW, window::FOCUSED),
                Event::Window(window::Event::Focused),
            )
        } else {
            (
                EventIdentify::new(EventFlag::WINDOW, window::UNFOCUSED),
                Event::Window(window::Event::Unfocused),
            )
        }]),
        WindowEvent::HoveredFile(path) => Some(smallvec![(
            EventIdentify::new(EventFlag::WINDOW, window::FILE_HOVERED),
            Event::Window(window::Event::FileHovered(path)),
        )]),
        WindowEvent::DroppedFile(path) => Some(smallvec![(
            EventIdentify::new(EventFlag::WINDOW, window::FILE_DROPPED),
            Event::Window(window::Event::FileDropped(path)),
        )]),
        WindowEvent::HoveredFileCancelled => Some(smallvec![(
            EventIdentify::new(EventFlag::WINDOW, window::FILES_HOVERED_LEFT),
            Event::Window(window::Event::FilesHoveredLeft),
        )]),
        WindowEvent::Touch(_touch) => {
            // let (touch_flag, touch_event) = touch_event(*touch, scale_factor);
            // Some(((EventFlag::TOUCH, touch_flag), Event::Touch(touch_event)))
            todo!()
        }
        WindowEvent::TouchpadPressure {
            device_id: _,
            pressure: _,
            stage: _,
        } => None,
        WindowEvent::Moved(position) => {
            let winit::dpi::LogicalPosition { x, y } = position.to_logical(scale_factor);

            Some(smallvec![(
                EventIdentify::new(EventFlag::WINDOW, window::MOVED),
                Event::Window(window::Event::Moved { x, y }),
            )])
        }
        _ => None,
    }
}

/// Converts a [`Position`] to a [`winit`] logical position for a given monitor.
///
/// [`winit`]: https://github.com/rust-windowing/winit
///logical_size
pub fn position(
    monitor: Option<&winit::monitor::MonitorHandle>,
    (width, height): (u32, u32),
    position: SemanticPosition,
    user_scale_factor: f64,
) -> Option<winit::dpi::Position> {
    match position {
        SemanticPosition::Default => None,
        SemanticPosition::Specific(x, y) => {
            Some(winit::dpi::Position::Logical(winit::dpi::LogicalPosition {
                x: f64::from(x) * user_scale_factor,
                y: f64::from(y) * user_scale_factor,
            }))
        }
        SemanticPosition::Centered => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: winit::dpi::LogicalSize<f64> =
                    monitor.size().to_logical(monitor.scale_factor());

                let centered: winit::dpi::PhysicalPosition<i32> = winit::dpi::LogicalPosition {
                    x: (resolution.width - f64::from(width) * user_scale_factor) / 2.0,
                    y: (resolution.height - f64::from(height) * user_scale_factor) / 2.0,
                }
                .to_physical(monitor.scale_factor());

                Some(winit::dpi::Position::Physical(
                    winit::dpi::PhysicalPosition {
                        x: start.x + centered.x,
                        y: start.y + centered.y,
                    },
                ))
            } else {
                None
            }
        }
    }
}

/// Converts a [`Mode`] to a [`winit`] fullscreen mode.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn fullscreen(
    monitor: Option<winit::monitor::MonitorHandle>,
    mode: Mode,
) -> Option<winit::window::Fullscreen> {
    match mode {
        Mode::Windowed | Mode::Hidden => None,
        Mode::Fullscreen => Some(winit::window::Fullscreen::Borderless(monitor)),
    }
}

/// Converts a [`Mode`] to a visibility flag.
pub fn visible(mode: Mode) -> bool {
    match mode {
        Mode::Windowed | Mode::Fullscreen => true,
        Mode::Hidden => false,
    }
}

/// Converts a `MouseCursor` from [`iced_native`] to a [`winit`] cursor icon.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
pub fn mouse_interaction(interaction: mouse::Interaction) -> winit::window::CursorIcon {
    use mouse::Interaction;

    match interaction {
        Interaction::Idle => winit::window::CursorIcon::Default,
        Interaction::Pointer => winit::window::CursorIcon::Hand,
        Interaction::Working => winit::window::CursorIcon::Progress,
        Interaction::Grab => winit::window::CursorIcon::Grab,
        Interaction::Grabbing => winit::window::CursorIcon::Grabbing,
        Interaction::Crosshair => winit::window::CursorIcon::Crosshair,
        Interaction::Text => winit::window::CursorIcon::Text,
        Interaction::ResizingHorizontally => winit::window::CursorIcon::EwResize,
        Interaction::ResizingVertically => winit::window::CursorIcon::NsResize,
    }
}

/// Converts a `MouseButton` from [`winit`] to an [`iced_native`] mouse button.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
pub fn mouse_button(mouse_button: winit::event::MouseButton) -> mouse::Button {
    match mouse_button {
        winit::event::MouseButton::Left => mouse::Button::Left,
        winit::event::MouseButton::Right => mouse::Button::Right,
        winit::event::MouseButton::Middle => mouse::Button::Middle,
        winit::event::MouseButton::Other(other) => mouse::Button::Other(other as u8),
    }
}

/// Converts some `ModifiersState` from [`winit`] to an [`iced_native`]
/// modifiers state.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
pub fn modifiers(modifiers: winit::event::ModifiersState) -> keyboard::Modifiers {
    let mut result = keyboard::Modifiers::empty();

    result.set(keyboard::Modifiers::SHIFT, modifiers.shift());
    result.set(keyboard::Modifiers::CTRL, modifiers.ctrl());
    result.set(keyboard::Modifiers::ALT, modifiers.alt());
    result.set(keyboard::Modifiers::LOGO, modifiers.logo());

    result
}

/// Converts a physical cursor position to a logical `Point`.
pub fn cursor_position(position: &winit::dpi::PhysicalPosition<f64>, scale_factor: f64) -> Pos {
    let logical_position = position.to_logical(scale_factor);

    Pos::new(logical_position.x, logical_position.y)
}
pub fn cursor_na_position(position: &Pos<f64>, scale_factor: f64) -> Pos {
    assert!(winit::dpi::validate_scale_factor(scale_factor));

    let logical = (position / scale_factor).cast::<f32>();
    debug!(
        "cursor point=====scale_factor:{} physical:{} logical:{} ",
        &scale_factor, &position, &logical
    );
    logical
}

/// Converts a `Touch` from [`winit`] to an [`iced_native`] touch event.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
pub fn touch_event(touch: winit::event::Touch, scale_factor: f64) -> (u32, touch::Event) {
    let id = touch::Finger(touch.id);
    let position = {
        let location = touch.location.to_logical::<f64>(scale_factor);

        Pos::new(location.x as f32, location.y as f32)
    };

    match touch.phase {
        winit::event::TouchPhase::Started => (
            touch::FINGER_PRESSED.bits(),
            touch::Event::FingerPressed { id, position },
        ),
        winit::event::TouchPhase::Moved => (
            touch::FINGER_MOVED.bits(),
            touch::Event::FingerMoved { id, position },
        ),
        winit::event::TouchPhase::Ended => (
            touch::FINGER_LIFTED.bits(),
            touch::Event::FingerLifted { id, position },
        ),
        winit::event::TouchPhase::Cancelled => (
            touch::FINGER_LOST.bits(),
            touch::Event::FingerLost { id, position },
        ),
    }
}

/// Converts a `VirtualKeyCode` from [`winit`] to an [`iced_native`] key code.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
pub fn key_code(virtual_keycode: winit::event::VirtualKeyCode) -> keyboard::KeyCode {
    use keyboard::KeyCode;

    match virtual_keycode {
        winit::event::VirtualKeyCode::Key1 => KeyCode::Key1,
        winit::event::VirtualKeyCode::Key2 => KeyCode::Key2,
        winit::event::VirtualKeyCode::Key3 => KeyCode::Key3,
        winit::event::VirtualKeyCode::Key4 => KeyCode::Key4,
        winit::event::VirtualKeyCode::Key5 => KeyCode::Key5,
        winit::event::VirtualKeyCode::Key6 => KeyCode::Key6,
        winit::event::VirtualKeyCode::Key7 => KeyCode::Key7,
        winit::event::VirtualKeyCode::Key8 => KeyCode::Key8,
        winit::event::VirtualKeyCode::Key9 => KeyCode::Key9,
        winit::event::VirtualKeyCode::Key0 => KeyCode::Key0,
        winit::event::VirtualKeyCode::A => KeyCode::A,
        winit::event::VirtualKeyCode::B => KeyCode::B,
        winit::event::VirtualKeyCode::C => KeyCode::C,
        winit::event::VirtualKeyCode::D => KeyCode::D,
        winit::event::VirtualKeyCode::E => KeyCode::E,
        winit::event::VirtualKeyCode::F => KeyCode::F,
        winit::event::VirtualKeyCode::G => KeyCode::G,
        winit::event::VirtualKeyCode::H => KeyCode::H,
        winit::event::VirtualKeyCode::I => KeyCode::I,
        winit::event::VirtualKeyCode::J => KeyCode::J,
        winit::event::VirtualKeyCode::K => KeyCode::K,
        winit::event::VirtualKeyCode::L => KeyCode::L,
        winit::event::VirtualKeyCode::M => KeyCode::M,
        winit::event::VirtualKeyCode::N => KeyCode::N,
        winit::event::VirtualKeyCode::O => KeyCode::O,
        winit::event::VirtualKeyCode::P => KeyCode::P,
        winit::event::VirtualKeyCode::Q => KeyCode::Q,
        winit::event::VirtualKeyCode::R => KeyCode::R,
        winit::event::VirtualKeyCode::S => KeyCode::S,
        winit::event::VirtualKeyCode::T => KeyCode::T,
        winit::event::VirtualKeyCode::U => KeyCode::U,
        winit::event::VirtualKeyCode::V => KeyCode::V,
        winit::event::VirtualKeyCode::W => KeyCode::W,
        winit::event::VirtualKeyCode::X => KeyCode::X,
        winit::event::VirtualKeyCode::Y => KeyCode::Y,
        winit::event::VirtualKeyCode::Z => KeyCode::Z,
        winit::event::VirtualKeyCode::Escape => KeyCode::Escape,
        winit::event::VirtualKeyCode::F1 => KeyCode::F1,
        winit::event::VirtualKeyCode::F2 => KeyCode::F2,
        winit::event::VirtualKeyCode::F3 => KeyCode::F3,
        winit::event::VirtualKeyCode::F4 => KeyCode::F4,
        winit::event::VirtualKeyCode::F5 => KeyCode::F5,
        winit::event::VirtualKeyCode::F6 => KeyCode::F6,
        winit::event::VirtualKeyCode::F7 => KeyCode::F7,
        winit::event::VirtualKeyCode::F8 => KeyCode::F8,
        winit::event::VirtualKeyCode::F9 => KeyCode::F9,
        winit::event::VirtualKeyCode::F10 => KeyCode::F10,
        winit::event::VirtualKeyCode::F11 => KeyCode::F11,
        winit::event::VirtualKeyCode::F12 => KeyCode::F12,
        winit::event::VirtualKeyCode::F13 => KeyCode::F13,
        winit::event::VirtualKeyCode::F14 => KeyCode::F14,
        winit::event::VirtualKeyCode::F15 => KeyCode::F15,
        winit::event::VirtualKeyCode::F16 => KeyCode::F16,
        winit::event::VirtualKeyCode::F17 => KeyCode::F17,
        winit::event::VirtualKeyCode::F18 => KeyCode::F18,
        winit::event::VirtualKeyCode::F19 => KeyCode::F19,
        winit::event::VirtualKeyCode::F20 => KeyCode::F20,
        winit::event::VirtualKeyCode::F21 => KeyCode::F21,
        winit::event::VirtualKeyCode::F22 => KeyCode::F22,
        winit::event::VirtualKeyCode::F23 => KeyCode::F23,
        winit::event::VirtualKeyCode::F24 => KeyCode::F24,
        winit::event::VirtualKeyCode::Snapshot => KeyCode::Snapshot,
        winit::event::VirtualKeyCode::Scroll => KeyCode::Scroll,
        winit::event::VirtualKeyCode::Pause => KeyCode::Pause,
        winit::event::VirtualKeyCode::Insert => KeyCode::Insert,
        winit::event::VirtualKeyCode::Home => KeyCode::Home,
        winit::event::VirtualKeyCode::Delete => KeyCode::Delete,
        winit::event::VirtualKeyCode::End => KeyCode::End,
        winit::event::VirtualKeyCode::PageDown => KeyCode::PageDown,
        winit::event::VirtualKeyCode::PageUp => KeyCode::PageUp,
        winit::event::VirtualKeyCode::Left => KeyCode::Left,
        winit::event::VirtualKeyCode::Up => KeyCode::Up,
        winit::event::VirtualKeyCode::Right => KeyCode::Right,
        winit::event::VirtualKeyCode::Down => KeyCode::Down,
        winit::event::VirtualKeyCode::Back => KeyCode::Backspace,
        winit::event::VirtualKeyCode::Return => KeyCode::Enter,
        winit::event::VirtualKeyCode::Space => KeyCode::Space,
        winit::event::VirtualKeyCode::Compose => KeyCode::Compose,
        winit::event::VirtualKeyCode::Caret => KeyCode::Caret,
        winit::event::VirtualKeyCode::Numlock => KeyCode::Numlock,
        winit::event::VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
        winit::event::VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
        winit::event::VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
        winit::event::VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
        winit::event::VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
        winit::event::VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
        winit::event::VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
        winit::event::VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
        winit::event::VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
        winit::event::VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
        winit::event::VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
        winit::event::VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
        winit::event::VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
        winit::event::VirtualKeyCode::Plus => KeyCode::Plus,
        winit::event::VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
        winit::event::VirtualKeyCode::Apps => KeyCode::Apps,
        winit::event::VirtualKeyCode::At => KeyCode::At,
        winit::event::VirtualKeyCode::Ax => KeyCode::Ax,
        winit::event::VirtualKeyCode::Backslash => KeyCode::Backslash,
        winit::event::VirtualKeyCode::Calculator => KeyCode::Calculator,
        winit::event::VirtualKeyCode::Capital => KeyCode::Capital,
        winit::event::VirtualKeyCode::Colon => KeyCode::Colon,
        winit::event::VirtualKeyCode::Comma => KeyCode::Comma,
        winit::event::VirtualKeyCode::Convert => KeyCode::Convert,
        winit::event::VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
        winit::event::VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
        winit::event::VirtualKeyCode::Equals => KeyCode::Equals,
        winit::event::VirtualKeyCode::Grave => KeyCode::Grave,
        winit::event::VirtualKeyCode::Kana => KeyCode::Kana,
        winit::event::VirtualKeyCode::Kanji => KeyCode::Kanji,
        winit::event::VirtualKeyCode::LAlt => KeyCode::LAlt,
        winit::event::VirtualKeyCode::LBracket => KeyCode::LBracket,
        winit::event::VirtualKeyCode::LControl => KeyCode::LControl,
        winit::event::VirtualKeyCode::LShift => KeyCode::LShift,
        winit::event::VirtualKeyCode::LWin => KeyCode::LWin,
        winit::event::VirtualKeyCode::Mail => KeyCode::Mail,
        winit::event::VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
        winit::event::VirtualKeyCode::MediaStop => KeyCode::MediaStop,
        winit::event::VirtualKeyCode::Minus => KeyCode::Minus,
        winit::event::VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
        winit::event::VirtualKeyCode::Mute => KeyCode::Mute,
        winit::event::VirtualKeyCode::MyComputer => KeyCode::MyComputer,
        winit::event::VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
        winit::event::VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
        winit::event::VirtualKeyCode::NextTrack => KeyCode::NextTrack,
        winit::event::VirtualKeyCode::NoConvert => KeyCode::NoConvert,
        winit::event::VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
        winit::event::VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
        winit::event::VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
        winit::event::VirtualKeyCode::OEM102 => KeyCode::OEM102,
        winit::event::VirtualKeyCode::Period => KeyCode::Period,
        winit::event::VirtualKeyCode::PlayPause => KeyCode::PlayPause,
        winit::event::VirtualKeyCode::Power => KeyCode::Power,
        winit::event::VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
        winit::event::VirtualKeyCode::RAlt => KeyCode::RAlt,
        winit::event::VirtualKeyCode::RBracket => KeyCode::RBracket,
        winit::event::VirtualKeyCode::RControl => KeyCode::RControl,
        winit::event::VirtualKeyCode::RShift => KeyCode::RShift,
        winit::event::VirtualKeyCode::RWin => KeyCode::RWin,
        winit::event::VirtualKeyCode::Semicolon => KeyCode::Semicolon,
        winit::event::VirtualKeyCode::Slash => KeyCode::Slash,
        winit::event::VirtualKeyCode::Sleep => KeyCode::Sleep,
        winit::event::VirtualKeyCode::Stop => KeyCode::Stop,
        winit::event::VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
        winit::event::VirtualKeyCode::Sysrq => KeyCode::Sysrq,
        winit::event::VirtualKeyCode::Tab => KeyCode::Tab,
        winit::event::VirtualKeyCode::Underline => KeyCode::Underline,
        winit::event::VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
        winit::event::VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
        winit::event::VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
        winit::event::VirtualKeyCode::Wake => KeyCode::Wake,
        winit::event::VirtualKeyCode::WebBack => KeyCode::WebBack,
        winit::event::VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
        winit::event::VirtualKeyCode::WebForward => KeyCode::WebForward,
        winit::event::VirtualKeyCode::WebHome => KeyCode::WebHome,
        winit::event::VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
        winit::event::VirtualKeyCode::WebSearch => KeyCode::WebSearch,
        winit::event::VirtualKeyCode::WebStop => KeyCode::WebStop,
        winit::event::VirtualKeyCode::Yen => KeyCode::Yen,
        winit::event::VirtualKeyCode::Copy => KeyCode::Copy,
        winit::event::VirtualKeyCode::Paste => KeyCode::Paste,
        winit::event::VirtualKeyCode::Cut => KeyCode::Cut,
        winit::event::VirtualKeyCode::Asterisk => KeyCode::Asterisk,
    }
}

// As defined in: http://www.unicode.org/faq/private_use.html
pub(crate) fn is_private_use_character(c: char) -> bool {
    matches!(
        c,
        '\u{E000}'..='\u{F8FF}'
        | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}'
    )
}
