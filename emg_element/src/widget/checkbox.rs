/*
 * @Author: Rais
 * @Date: 2021-09-01 09:58:44
 * @LastEditTime: 2023-03-02 19:42:20
 * @LastEditors: Rais
 * @Description:
 */
// ! Show toggle controls using checkboxes.

use gtree::gtree;

use crate::{
    g_element::DynGElement, g_tree_builder::GTreeInit, GElement, GTreeBuilderElement, InitdTree,
    IntoOptionMs,
};

use emg_common::{
    any::MessageTid,
    better_any::{Tid, TidAble, TidExt},
    mouse::CLICK,
    IdStr, LogicLength, TypeCheckObjectSafe, TypeCheckObjectSafeTid, TypeName,
};
use emg_layout::EmgEdgeItem;
use emg_shaping::{Shaping, ShapingAny, ShapingUse, ShapingUseAny};
use emg_state::{topo, use_state, StateAnchor, StateMultiAnchor, StateVar};
use tracing::{debug, debug_span, info, warn, Span};

use std::{panic::Location, rc::Rc};

#[allow(missing_debug_implementations)]
#[derive(Tid)]
pub struct Checkbox<Message> {
    is_checked: StateVar<bool>,
    //FIXME use cow for Rc 防止 克隆对象和 原始对象使用同一个 callback
    on_toggle: Option<Rc<dyn Fn(bool) -> Option<Message>>>,
    label: IdStr,
    id: Option<IdStr>,
    width: LogicLength,
    // #[allow(dead_code)]
    // style: Box<dyn StyleSheet>,
}

impl<Message> Eq for Checkbox<Message> {}

impl<Message> Clone for Checkbox<Message> {
    fn clone(&self) -> Self {
        Self {
            is_checked: self.is_checked,
            on_toggle: self.on_toggle.clone(),
            label: self.label.clone(),
            id: self.id.clone(),
            width: self.width.clone(),
        }
    }
}

impl<Message> std::fmt::Debug for Checkbox<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("id", &self.id)
            .field("width", &self.width)
            .finish()
    }
}
impl<Message> PartialEq for Checkbox<Message>
// where
// Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.is_checked == other.is_checked
            && if let (Some(s), Some(t)) = (&self.on_toggle, &other.on_toggle) {
                std::ptr::eq(
                    (std::ptr::addr_of!(**s)).cast::<u8>(),
                    (std::ptr::addr_of!(**t)).cast::<u8>(),
                )
            } else {
                false
            }
            && self.label == other.label
            && self.id == other.id
            && self.width == other.width
    }
}

impl<Message> Checkbox<Message>
// where
//     dyn std::ops::Fn(bool) -> Message + 'static: std::cmp::PartialEq,
{
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    #[topo::nested]
    pub fn new<T>(
        is_checked: bool,
        label: impl Into<IdStr>,
        f: impl Fn(bool) -> T + 'static,
    ) -> Self
    where
        T: IntoOptionMs<Message>,
    {
        Self {
            is_checked: use_state(|| is_checked),
            on_toggle: Some(Rc::new(move |is_checked| f(is_checked).into_option())),
            label: label.into(),
            id: None,
            width: LogicLength::default(),
            // style: std::boxed::Box::default(),
        }
    }

    /// Sets the width of the [`Checkbox`].
    #[must_use]
    pub fn width(mut self, width: LogicLength) -> Self {
        self.width = width;
        self
    }

    // /// Sets the style of the [`Checkbox`].
    // #[must_use]
    // pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
    //     self.style = style.into();
    //     self
    // }

    /// Sets the id of the [`Checkbox`].
    #[must_use]
    pub fn id(mut self, id: impl Into<IdStr>) -> Self {
        self.id = Some(id.into());
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: IdStr) -> Self {
        self.label = label;
        self
    }
}

impl<Message> GTreeInit<Message> for Checkbox<Message>
where
    Message: Clone + PartialEq + for<'a> emg_common::any::MessageTid<'a>,
{
    #[topo::nested]
    #[allow(clippy::useless_conversion)]
    fn tree_init(
        mut self,
        id: &IdStr, //outSide generated id
        _es: &[Rc<dyn Shaping<EmgEdgeItem>>],
        _children: &[GTreeBuilderElement<Message>],
    ) -> InitdTree<Message> {
        use crate::gtree_macro_prelude::*;
        let is_checked = self.is_checked;
        let on_toggle = self.on_toggle.take().unwrap();
        gtree! {
            //TODO add str default id
            @SkipInit self =>[
                @=id.clone() + "|CLICK" On:CLICK  move||{

                    // is_checked.update_and_return(|v| {

                    //   let new_v =   !*v;
                    //   *v =new_v;
                    //   (on_toggle)(new_v)
                    // })
                    let _span = debug_span!("checkbox", at = "click").entered();
                    is_checked.set_with_once(|v| !*v);
                      (on_toggle)(is_checked.get())

                },
            ]
        }
        .into()
    }
}

#[cfg(all(feature = "gpu"))]
use crate::renderer::*;
#[cfg(all(feature = "gpu"))]
impl<Message> crate::Widget for Checkbox<Message>
where
    Message: 'static,
{
    type SceneCtxType = crate::SceneFrag;
    fn paint_sa(
        &self,
        painter: &StateAnchor<crate::PaintCtx>,
    ) -> StateAnchor<Rc<Self::SceneCtxType>> {
        let span = illicit::expect::<Span>();

        (painter, &self.is_checked.watch()).map(move |incoming_painter, &is_checked| {
            let dpr = incoming_painter.dpr();
            debug_span!("window_size", at = "checkbox paint_sa", dpr).in_scope(|| {});

            let mut sc = Self::SceneCtxType::new(incoming_painter.get_translation());
            let mut builder = sc.gen_builder();

            let rect = incoming_painter.size().to_rect();
            //fill

            if let Some(fill) = incoming_painter.get_fill_color() {
                info!(parent: &*span,"fill color: {:?}", &fill);
                builder.fill(Fill::NonZero, Affine::IDENTITY, fill, None, &rect);
            }
            // check_zone
            let check_zone_w_h = 14. * dpr;
            let origin = ((rect.height() - check_zone_w_h) * 0.5).trunc();

            let box_rect = Rect {
                x0: 0.,
                y0: 0.,
                x1: check_zone_w_h,
                y1: check_zone_w_h, //
            }
            .with_origin((origin, origin));
            // .to_rounded_rect(2. * dpr);

            if is_checked {
                // emg_layout::styles::seed_colors::Red;
                builder.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::BLUE,
                    None,
                    &box_rect,
                );
            }

            builder.stroke(
                // &Stroke::new(1. * dpr as f32),
                &Stroke::new(1.),
                Affine::IDENTITY,
                Color::BLACK,
                None,
                &box_rect,
            );

            // border
            if let Some(bw) = incoming_painter.get_border_width() {
                if let Some(bc) = incoming_painter.get_border_color() {
                    info!(parent: &*span,"border width: {:?} color: {:?}", &bw, &bc);

                    builder.stroke(&Stroke::new(bw), Affine::IDENTITY, bc, None, &rect);
                } else {
                    // has border width but no border color
                    builder.stroke(
                        &Stroke::new(bw),
                        Affine::IDENTITY,
                        Color::BLACK,
                        None,
                        &rect.inset(-(bw as f64) / 2. - 0.), //TODO 检查,这是临时设置
                    );
                }
            }

            // ─────────────────────────────────────────────

            builder.finish();
            Rc::new(sc)
        })
    }
}

impl<Message> Shaping<Self> for Checkbox<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a>,
{
    fn shaping(&self, who: &mut Self) -> bool {
        debug!(
            "Generic: use Checkbox refresh for checkbox self:{} shaping-> who:{}",
            &self.label, &who.label
        );
        if *who != self.clone() {
            *who = self.clone();
            return true;
        }
        false
    }
}
// impl<'a, Message, T: Shaping<Checkbox<Message>>> AsRefreshFor<Checkbox<Message>> for T {
//     fn as_refresh_for(&self) -> &dyn Shaping<Checkbox<Message>> {
//         self
//     }
// }

// #[derive(Tid)]
// struct MM<T>(T);
// NOTE 当前 走 先 被下游更新,  如果 自身无法找到下游对自己的更新规则(下级未知), 则询问下游的向上更新方法
//NOTE 性能上, 上游先判断 一直call 同一个函数 应该会快一点 ,但是 一般情况 新建元素都是属于下级.更多的是下级未知.
//NOTE 如果 先下级向上更新,会downcast很多次上级, 想取消 下级的默认向上更新,可以 warp 此下级,比较好实现, 关键是下级在构建过程中会不可知,很难调取到自身的 shaping
//NOTE     而且下游 很难知道上游的具体类型
//NOTE 如果 先自己被下级更新, 想取消 自己针对此下级的更新规则,可以 warp 此下级,让自己不认识.
// @ 下游 GElement 更新  Checkbox ------------------------------------
impl<Message> Shaping<Checkbox<Message>> for GElement<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    #[allow(clippy::match_same_arms)]
    fn shaping(&self, who_checkbox: &mut Checkbox<Message>) -> bool {
        match self {
            Self::Layer_(_l) => {
                todo!("使用 layer里一堆东西 更新 CheckBox");
            }
            Self::Builder_(builder) => {
                let _span =
                    debug_span!("GElement-shaping", "<Builder_> shaping-> <Checkbox>").entered();

                let _span =
                    debug_span!("better_any_shaping", "Builder_ shaping-> Checkbox").entered();

                builder.widget().shaping(who_checkbox)
            }
            //TODO enable this
            // Self::Text_(t) => {
            //     who_checkbox.label = t.get_content(); //TODO text.get_content directly return IdStr
            // }
            // Self::Button_(_) => {
            //     unimplemented!();
            // }
            Self::Event_(_) => {
                todo!();
            }
            Self::Generic_(g_self) => {
                let _span =
                    debug_span!("GElement-shaping", "<Generic_> shaping-> <Checkbox>").entered();

                let _span1 =
                    debug_span!("better_any_shaping", "Generic shaping-> Checkbox").entered();
                warn!("use Generic({}) shaping Checkbox", g_self.type_name());
                debug!(
                    "Generic is Checkbox? {}",
                    g_self.is::<Box<Checkbox<Message>>>() //is false
                );
                //TODO 使用值反射 不知道下级实际类型也能更新自己的值

                // if let Some(s) = (&**g_self).downcast_ref::<Checkbox<Message>>() {
                //     debug!("成功 downcast to Self");
                //     s.shaping(who_checkbox);
                // } else {
                //     debug!("失败 downcast to Self");
                // }
                // if let Some(_) = (&**g_self).as_any().downcast_ref::<Checkbox<Message>>() {
                //     debug!("成功 downcast to Self");
                // } else {
                //     debug!("失败 downcast to Self");
                // }

                // let is_changed = who_checkbox.shaping_use_any(x as &dyn TypeCheckObjectSafeTid);
                let is_changed = who_checkbox.shaping_use_any(g_self);

                if !is_changed {
                    debug!("===================== 向上更新");
                    g_self.shaping_any(who_checkbox)
                } else {
                    false
                }

                //TODO shaping 反馈 bool, 失败后, 启用 下级的 向上更新
                // todo!("此上为实验性代码");

                //TODO 反射?
                // todo!("reflection? ",);
            }
            Self::NodeRef_(_) => panic!("GElement::NodeIndex_() should handle before."),
            Self::EmptyNeverUse => panic!("EmptyNeverUse never here"),
            Self::SaNode_(_) => todo!(),

            Self::EvolutionaryFactor(_) => todo!(),

            GElement::Shaper_(_) => todo!(),
        }
    }
}

//TODO use macro
impl<Message> TypeCheckObjectSafe for Checkbox<Message> {
    fn type_name(&self) -> TypeName {
        TypeName::new(IdStr::new_inline("Checkbox"))
    }
}

impl<Message> DynGElement<Message> for Checkbox<Message> where
    Message: Clone + 'static + for<'a> MessageTid<'a> + std::cmp::PartialEq
{
}

impl<Message> From<Checkbox<Message>> for GElement<Message>
where
    Message: Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq + 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Self {
        Self::Generic_(Box::new(checkbox))
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// impl<Message> ShapingUse<i32> for Checkbox<Message> {
//     fn shaping_use(&mut self, use_something: &i32) {
//         self.label = format!("checkbox i32: {}", use_something).into()
//     }
// }

impl<Message> Shaping<Checkbox<Message>> for i32
where
    Message: 'static + Clone + for<'a> MessageTid<'a>,
{
    fn shaping(&self, who: &mut Checkbox<Message>) -> bool {
        warn!(
            "[checkbox] use i32 refresh for checkbox self:{:?}-who:{}",
            &who, &self
        );

        let into = format!("checkbox i32: {}", self).into();
        if who.label != into {
            who.label = into;
            return true;
        }
        false
    }
}
// impl<Message, Use: Sized + Clone + std::fmt::Debug + 'static> TryRefreshFor<Checkbox<Message>>
//     for Rc<Use>
// {
//     fn try_refresh_for(&self, who: &mut Checkbox<Message>) {
//         warn!(
//             "[try_refresh_for] self:{} try downcast to Rc<dyn Shaping<{}>>",
//             std::any::type_name::<Self>(),
//             std::any::type_name::<Checkbox<Message>>()
//         );
//         let u = self.clone();
//         let any: &dyn Any = &u;
//         if let Some(u_s_e) = any.downcast_ref::<Rc<dyn Shaping<Checkbox<Message>>>>() {
//             who.shape_of_use(&**u_s_e);
//         } else {
//             warn!("try_refresh failed: use {:?} for who:{:?}", &self, &who);
//         }
//     }
// }

//@ 向上更新, ---- 处于下游的 self 向上更新 who - 具体上层类型 ------------------------------------
impl<Message> ShapingAny for Checkbox<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a>,
{
    #[track_caller]
    fn shaping_any(&self, any: &mut dyn TypeCheckObjectSafeTid) -> bool {
        let _span = debug_span!(
            "better_any_shaping",
            at = "ShapingAny",
            note = "向上更新",
            "Checkbox shaping ====> any"
        )
        .entered();

        if let Some(who) = any.downcast_mut::<Checkbox<Message>>() {
            debug!("who 成功 downcast to Self: Checkbox<Message>");
            return self.shaping(who);
        }

        warn!(
            "any - type_name: {}, not match any role,\nLocation:{}",
            any.type_name(),
            Location::caller(),
        );
        false
    }
}

//@ 被下更新  下游 Box<dyn DynGElement<Message>> 更新 self
impl<Message> ShapingUseAny for Checkbox<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a>,
{
    #[track_caller]
    fn shaping_use_any(&mut self, any: &dyn Tid) -> bool {
        let _span = debug_span!(
            "better_any_shaping",
            at = "ShapingUseAny",
            "Checkbox <==== shaping_use_any"
        )
        .entered();

        if let Some(x) = any.downcast_ref::<Box<dyn DynGElement<Message>>>() {
            debug!("成功 downcast to any Box<dyn DynGElement<Message>>");

            // self.shaping_use(x);
            if let Some(x2) = (**x).downcast_ref::<Self>() {
                debug!("1 ** 成功 downcast to Self");
                return self.shaping_use(x2);
            }
        }

        warn!(
            "Self:{} , [try_shaping_use]  try downcast,\nLocation:{}",
            std::any::type_name::<Self>(),
            Location::caller()
        );

        false

        // test code ─────────────────────────────────────────────────────────────

        // if let Some(x2) = x.downcast_ref::<Box<dyn Shaping<Self>>>() {
        //     debug!("0 ** 成功 downcast to  Box<dyn Shaping<Self>>");
        //     // self.shaping_use(x2);
        // }
        // if let Some(x2) = (x.clone()).downcast_box::<Box<dyn Shaping<Self>>>().ok() {
        //     debug!("0 ** 成功 downcast to  Box<dyn Shaping<Self>>");
        //     // self.shaping_use(x2);
        // }

        // if let Some(x) = any.downcast_ref::<Box<dyn DynGElement<Message>>>() {
        //     debug!("成功 downcast to Box<dyn DynGElement<Message>>");
        //     self.shaping_use(x);
        // }
        // if let Some(x) = any.downcast_ref::<Box<dyn Shaping<Self>>>() {
        //     debug!("成功 downcast to Box<dyn Shaping<Self>>");
        //     self.shaping_use(x);
        // }

        // if let Some(x) = any.downcast_ref::<Self>() {
        //     debug!("成功 downcast to Self");
        //     self.shaping_use(x);
        // }
        // if let Some(x) = any.downcast_ref::<Box<Self>>() {
        //     debug!("成功 downcast to box Self");
        //     self.shaping_use(x);
        // }

        // // ─────────────────────────────────────────────────────────────
        // if let Some(x) = any.downcast_any_ref::<Box<dyn DynGElement<Message>>>() {
        //     debug!("成功 downcast to any Box<dyn DynGElement<Message>>");

        //     // self.shaping_use(x);
        //     if let Some(x2) = (&**x).downcast_ref::<Self>() {
        //         debug!("1 ** 成功 downcast to any Self");
        //         self.shaping_use(x2);
        //     }

        //     if let Some(x2) = x.downcast_ref::<Self>() {
        //         debug!("1成功 downcast to any Self");
        //         self.shaping_use(x2);
        //     }
        //     if let Some(x2) = x.downcast_ref::<Box<Self>>() {
        //         debug!("1成功 downcast to any box Self");
        //         self.shaping_use(x2);
        //     }
        // }

        // if let Some(x) = any.downcast_any_ref::<Box<dyn Shaping<Self>>>() {
        //     debug!("成功 downcast to any Box<dyn Shaping<Self>>");
        //     self.shaping_use(x);
        // }

        // if let Some(x) = any.downcast_any_ref::<Self>() {
        //     debug!("成功 downcast to any Self");
        //     self.shaping_use(x);
        // }
        // if let Some(x) = any.downcast_any_ref::<Box<Self>>() {
        //     debug!("成功 downcast to any box Self");
        //     self.shaping_use(x);
        // }
    }
}

#[cfg(test)]
mod test {

    trait Mode {
        type Output;
        fn doit(&self) -> Self::Output;
    }

    impl Mode for i32 {
        type Output = i32;
        fn doit(&self) -> Self::Output {
            1
        }
    }

    impl Mode for String {
        type Output = String;
        fn doit(&self) -> Self::Output {
            "xx".to_string()
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    use bevy_reflect::{reflect_trait, Reflect};

    #[derive(Reflect)]
    // #[reflect(Mode1)]
    struct Wi32(i32);

    // ─────────────────────────────────────────────────────────────────────
    // #[reflect_trait]
    // trait Mode1<X: Reflect> {
    //     fn convert(&self) -> &dyn Shaping<X>;
    // }
    // ─────────────────────────────────────────────────────────────────────────────

    // impl<X: 'static> Mode1<X> for Wi32 {
    //     fn convert(&self) -> &dyn Shaping<X> {
    //         self as &dyn Shaping<X>
    //     }
    // }
    // impl<X> Mode1<X> for String {
    //     fn convert(&self) -> &dyn Shaping<X> {
    //         self as &dyn Shaping<X>
    //     }
    // }
    // #[reflect_trait]
    trait Mode2 {
        type O;
        // fn convert(&self) -> &dyn Shaping<Self>;
        fn convert(&self) -> Self::O;
    }

    impl Mode2 for i32 {
        type O = i32;
        fn convert(&self) -> i32 {
            *self
        }
    }

    #[test]
    fn test() {}
}

#[cfg(test)]
mod tests {
    use emg_msg_macro::emg_msg;

    use super::*;
    use emg_common::{any, better_any};

    #[emg_msg]
    #[derive(Clone, PartialEq)]
    enum Message {
        A,
    }
    #[test]
    fn test_eq() {
        let ff = |_| Message::A;
        let checkbox = Checkbox::new(false, "checkbox", ff);
        let checkbox2 = Checkbox::new(false, "checkbox", ff);
        let checkbox3 = checkbox.clone();
        assert_ne!(checkbox, checkbox2);
        assert_eq!(checkbox, checkbox3);
    }

    #[test]
    fn test_gtree_macro() {
        use crate::gtree_macro_prelude::*;
        let checkbox = Checkbox::new(false, "checkbox", |_| Message::A);
        let gtree = gtree! {
            @="default_id" checkbox=>[]
        };
        println!("{:#?}", gtree);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("test_gtree_macro", gtree);
    }
}
