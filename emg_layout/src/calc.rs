use std::rc::Rc;

/*
* @Author: Rais
* @Date: 2021-03-29 17:30:58
 * @LastEditTime: 2023-01-28 19:46:33
 * @LastEditors: Rais
* @Description:
*/
use crate::{
    ccsa::CassowaryMap, EdgeData, GenericSize, GenericSizeAnchor, Layout, LayoutCalculated, Mat4,
    CHILD_PROP_WEIGHT,
};

use cassowary::{
    strength::{REQUIRED, WEAK},
    WeightedRelation,
};
use derive_more::From;
use emg::EdgeIndex;
use emg_common::na::{Translation3, Vector2};
use emg_common::{Precision, TypeName};
use emg_state::{topo, StateAnchor, StateMultiAnchor, StateVar};
use float_cmp::assert_approx_eq;
use seed_styles as styles;
use styles::{s, CssTransform, CssTransformTrait, LogicLength};
use tracing::{debug_span, info, trace, trace_span, warn, warn_span};

use self::cassowary_calc::cassowary_calculation;

mod cassowary_calc;

// ────────────────────────────────────────────────────────────────────────────────

// #[track_caller]
#[topo::nested]
pub fn layout_calculating(
    _id: StateVar<StateAnchor<EdgeIndex>>,
    path_edgedata: &EdgeData, //parent
    current_cassowary_map: &Rc<CassowaryMap>,
    layout: &StateAnchor<Layout>,
) -> LayoutCalculated

{
    let _span_ = trace_span!("->[ layout_calculating ] ").entered();
    let _debug_span_ = debug_span!("->[ layout_calculating ] ").entered();

    let EdgeData {
        calculated: p_calculated,
        children_vars_sa: p_children_vars_sa,
        cassowary_calculated_vars: p_cass_calculated_vars,
        cassowary_map: p_cassowary_map,
        cassowary_calculated_layout: p_cassowary_calculated_layout,
        ..
    } = path_edgedata;

    // ────────────────────────────────────────────────────────────────────────────────

    // let p_calc_size_sa = &p_calculated.real_size;
    // NOTE p_cassowary_calculated_layout   - parent 自己算的,
    // NOTE p_calculated.real_size          - parent 的 parent算的
    // let p_cass_or_calc_size_sa = &(p_cassowary_calculated_layout,&p_calculated.cass_or_calc_size).map(|(w,h),size|{
    //     // let w = w.unwrap_or(size.x);
    //     // let h = h.unwrap_or(size.y);
    //     let w = w.unwrap();
    //     let h = h.unwrap();
    //     Vector2::<f64>::new(w,h)
    // });
    let p_cass_p_size_sa = p_cassowary_calculated_layout.map(|(w, h)| {
        // let w = w.unwrap_or(size.x);
        // let h = h.unwrap_or(size.y);
        let w = w.unwrap();
        let h = h.unwrap();
        Vector2::<Precision>::new(w, h)
    });

    // ─────────────────────────────────────────────────────────────────
    let w = layout.then(|l: &Layout| l.w.watch().into());
    let h = layout.then(|l: &Layout| l.h.watch().into());
    let origin_x = layout.then(|l: &Layout| l.origin_x.watch().into());
    let origin_y = layout.then(|l: &Layout| l.origin_y.watch().into());
    let align_x = layout.then(|l: &Layout| l.align_x.watch().into());
    let align_y = layout.then(|l: &Layout| l.align_y.watch().into());
    let current_cassowary_generals_sa = layout.then(|l| l.cassowary_generals.watch().into());
    // ─────────────────────────────────────────────────────────────────
    let width_var = current_cassowary_map.var("width").unwrap();
    let height_var = current_cassowary_map.var("height").unwrap();
    let top_var = current_cassowary_map.var("top").unwrap();
    let left_var = current_cassowary_map.var("left").unwrap();
    let bottom_var = current_cassowary_map.var("bottom").unwrap();
    let right_var = current_cassowary_map.var("right").unwrap();
    // ────────────────────────────────────────────────────────────────────────────────

    let p_cassowary_map2 = p_cassowary_map.clone();
    let sa_gs_w = w.then(|w| w.get_anchor());
    let sa_gs_h = h.then(|h| h.get_anchor());

    let size_constraints = (&sa_gs_w, &sa_gs_h).map(move |w: &GenericSize, h: &GenericSize| {
        let mut size_constraints = vec![];

        // if let Ok(ww)   = w.try_get_length_value() &&  approx_eq!(f64,ww,0.0,(0.1,2)){
        if !w.is_none() {
            size_constraints.push(
                width_var
                    | WeightedRelation::EQ(CHILD_PROP_WEIGHT)
                    | cassowary_calculation("width", &p_cassowary_map2, w),
            );
        }
        // if let Ok(hh)  = h.try_get_length_value() && approx_eq!(f64,hh,0.0,(0.1,2)){
        if !h.is_none() {
            // size_constraints.push(  height_var  | WeightedRelation::EQ(cassowary::strength::WEAK) | 0.0);

            size_constraints.push(
                height_var
                    | WeightedRelation::EQ(CHILD_PROP_WEIGHT)
                    | cassowary_calculation("height", &p_cassowary_map2, h),
            );
        }

        size_constraints.extend([
            (right_var - left_var) | WeightedRelation::EQ(REQUIRED) | width_var,
            (bottom_var - top_var) | WeightedRelation::EQ(REQUIRED) | height_var,
            bottom_var | WeightedRelation::GE(REQUIRED) | top_var,
            right_var | WeightedRelation::GE(REQUIRED) | left_var,
            width_var | WeightedRelation::GE(REQUIRED) | 0.0,
            height_var | WeightedRelation::GE(REQUIRED) | 0.0,
            top_var | WeightedRelation::GE(WEAK) | 0.0,
            left_var | WeightedRelation::GE(WEAK) | 0.0,
        ]);

        size_constraints
    });

    let current_cassowary_map2 = current_cassowary_map.clone();
    let current_cassowary_inherited_generals_sa = (
        &p_calculated.cassowary_inherited_generals_sa,
        &current_cassowary_generals_sa,
    )
        .map(move |p_cass_inherited_generals, self_generals| {
            let _span = trace_span!("build inherited cassowary_generals_map").entered();
            trace!("parent_cassowary_generals + current_cassowary_generals + current_cassowary_map:----");
            trace!("-- parent_cassowary_generals:{:#?}", &p_cass_inherited_generals);
            trace!("-- current_cassowary_generals:{:#?}", &self_generals);
            trace!("-- current_cassowary_map:{:#?}", &current_cassowary_map2);

            //TODO 当前 + 法 使 Rc<CassowaryGeneralMap> 中包含 Rc<CassowaryGeneralMap>  可能会导致 无法 Drop?
            let end = p_cass_inherited_generals.clone()
                + self_generals.clone()
                + current_cassowary_map2.clone();
            trace!("-- end final map:{:#?}", &end);

            Rc::new(end)
        });

    let no_cass_downgrade_calculated_size = (&p_cass_p_size_sa, &sa_gs_w, &sa_gs_h).map(
        move |&p_calc_size: &Vector2<Precision>, w: &GenericSize, h: &GenericSize| {
            // TODO  如果根 parent 无关 不是百分比  那么 不监听 parent, 如果 w, h 独立 不依赖, 分开计算suggest_calculated_width suggest_calculated_height

            //TODO check editor display error
            let new_size = Vector2::<Precision>::new(
                calculation_w(p_calc_size, w),
                calculation_h(p_calc_size, h),
            );
            trace!("new size: {}", &new_size);
            info!("==== new size: {}", &new_size);
            new_size
        },
    );

    let (no_cass_downgrade_calculated_width, no_cass_downgrade_calculated_height) =
        no_cass_downgrade_calculated_size
            .map(|size| (size.x, size.y))
            .split();

    // ────────────────────────────────────────────────────────────────────────────────

    //NOTE used for suggest in current cassowary , with in [cass_or_calc_size]
    let cass_or_calc_width = (p_cass_calculated_vars, &no_cass_downgrade_calculated_width).map(
        move |p_vars, no_cass_width| {
            let _debug_span_ = warn_span!("->[ get self prop calculated value ] ").entered();
            // warn!("p_vars: {:?},  \n get :{:?}",&p_vars,&width_var);
            // • • • • •
            //TODO only width change then do this
            //NOTE 如果 是 root下面的第一阶层节点,如果没定义cassowary constraint 或者定义少量、不涉及某些 Id element, p_calculated_vars 很可能是无 or 不全面的,
            p_vars
                .get(&width_var)
                .map_or_else(|| *no_cass_width, |(val, _)| **val)
        },
    );

    //NOTE used for suggest in current cassowary , with in [cass_or_calc_size]
    let cass_or_calc_height = (p_cass_calculated_vars, &no_cass_downgrade_calculated_height).map(
        move |p_vars, no_cass_height| {
            //TODO only height change then do this
            p_vars
                .get(&height_var)
                .map_or_else(|| *no_cass_height, |(val, _)| **val)
        },
    );

    let top = (p_children_vars_sa, p_cass_calculated_vars).map(move |p_children_vars, p_vars| {
        //TODO only xx change then do this
        if p_children_vars.contains(&top_var) {
            p_vars.get(&top_var).map(|(val, _)| **val)
        } else {
            None
        }
    });
    let left = (p_children_vars_sa, p_cass_calculated_vars).map(move |p_children_vars, p_vars| {
        //TODO only xx change then do this
        if p_children_vars.contains(&left_var) {
            p_vars.get(&left_var).map(|(val, _)| **val)
        } else {
            None
        }
    });
    let bottom =
        (p_children_vars_sa, p_cass_calculated_vars).map(move |p_children_vars, p_vars| {
            //TODO only xx change then do this
            if p_children_vars.contains(&bottom_var) {
                p_vars.get(&bottom_var).map(|(val, _)| **val)
            } else {
                None
            }
        });

    let right = (p_children_vars_sa, p_cass_calculated_vars).map(move |p_children_vars, p_vars| {
        //TODO only xx change then do this
        if p_children_vars.contains(&right_var) {
            p_vars.get(&right_var).map(|(val, _)| **val)
        } else {
            None
        }
    });

    //TODO 如果 父层 cassowary 不涉及到 此 element , 那么就需要 进行 原定位计算
    let cass_trans: StateAnchor<Translation3<Precision>> = (
        &cass_or_calc_width,
        &cass_or_calc_height,
        &top,
        &left,
        &bottom,
        &right,
    )
        .map(
            move |&w: &Precision,
                  &h: &Precision,
                  &opt_t: &Option<Precision>,
                  &opt_l: &Option<Precision>,
                  &opt_b: &Option<Precision>,
                  &opt_r: &Option<Precision>| {
                let _span = warn_span!("cass_trans calculting map").entered();
                warn!(
                    "[cass_trans] t:{:?} l:{:?} b:{:?} r:{:?}",
                    opt_t, opt_l, opt_b, opt_r
                );
                let check: bool = false;
                match (opt_t, opt_l, opt_b, opt_r) {
                    (None, None, None, None) => Translation3::<Precision>::new(0., 0., 0.0),
                    (None, None, None, Some(r)) => Translation3::<Precision>::new(r - w, 0., 0.0),
                    (None, None, Some(b), None) => Translation3::<Precision>::new(0., b - h, 0.0),
                    (None, None, Some(b), Some(r)) => {
                        Translation3::<Precision>::new(r - w, b - h, 0.0)
                    }
                    (None, Some(l), None, None) => Translation3::<Precision>::new(l, 0., 0.0),
                    (None, Some(l), None, Some(r)) => {
                        if check {
                            assert_approx_eq!(Precision, r - l, w, (0.1, 2));
                        }
                        Translation3::<Precision>::new(l, 0., 0.0)
                    }
                    (None, Some(l), Some(b), None) => Translation3::<Precision>::new(l, b - h, 0.0),
                    (None, Some(l), Some(b), Some(r)) => {
                        if check {
                            assert_approx_eq!(Precision, r - l, w, (0.1, 2));
                        }
                        Translation3::<Precision>::new(l, b - h, 0.0)
                    }
                    (Some(t), None, None, None) => Translation3::<Precision>::new(0., t, 0.0),
                    (Some(t), None, None, Some(r)) => Translation3::<Precision>::new(r - w, t, 0.0),
                    (Some(t), None, Some(b), None) => {
                        if check {
                            assert_approx_eq!(Precision, b - t, h, (0.1, 2));
                        }
                        Translation3::<Precision>::new(0.0, t, 0.0)
                    }
                    (Some(t), None, Some(b), Some(r)) => {
                        if check {
                            assert_approx_eq!(Precision, b - t, h, (0.1, 2));
                        }
                        Translation3::<Precision>::new(r - w, t, 0.0)
                    }
                    (Some(t), Some(l), None, None) => Translation3::<Precision>::new(l, t, 0.0),
                    (Some(t), Some(l), None, Some(r)) => {
                        warn!("t:{} l:{} r:{}", t, l, r);
                        //TODO remove this if release
                        if check {
                            assert_approx_eq!(Precision, r - l, w, (0.1, 2));
                        }
                        Translation3::<Precision>::new(l, t, 0.0)
                    }
                    (Some(t), Some(l), Some(b), None) => {
                        //TODO remove this if release
                        if check {
                            assert_approx_eq!(Precision, b - t, h, (0.1, 2));
                        }
                        Translation3::<Precision>::new(l, t, 0.0)
                    }
                    (Some(t), Some(l), Some(b), Some(_r)) => {
                        //TODO remove this if release

                        // let mut buffer = ryu::Buffer::new();
                        // let b_t = buffer.format_finite(b-t);
                        // let mut buffer2 = ryu::Buffer::new();
                        // let h_ = buffer2.format_finite(*h);
                        warn!("b-t:{:.10} h:{:.10}", b - t, h);

                        // assert_approx_eq!(Precision,b-t,*h,(0.1,2));
                        // assert_approx_eq!(Precision,r-l,*w,(0.1,2));
                        Translation3::<Precision>::new(l, t, 0.0)
                    }
                }
            },
        );

    //TODO use this?
    // let origin_x_var  =current_cassowary_map.var("origin_x").unwrap();
    // let origin_y_var  =current_cassowary_map.var("origin_y").unwrap();
    // let align_x_var  =current_cassowary_map.var("align_x").unwrap();
    // let align_y_var  =current_cassowary_map.var("align_y").unwrap();
    // ────────────────────────────────────────────────────────────────────────────────

    // let calculated_size = p_calculated_vars.then(|p_vars|{
    //     let width = if let Some (width_val) = p_vars.get(width_var){
    //         StateAnchor::constant( **width_val)
    //     }else{
    //         (p_calc_size_sa, &w).then(
    //             move|p_calc_size: &Vector2<f64>, sa_w: &GenericSizeAnchor| {
    //                let p_calc_size = *p_calc_size;

    //                // TODO  如果根 parent 无关 不是百分比  那么 不监听 parent

    //                 sa_w.map(move |w:&GenericSize|->f64{
    //                    calculation_w(&p_calc_size, w)
    //                }).into()

    //            } )
    //     };

    //     let height = if let Some (height_val) = p_vars.get(height_var){
    //         StateAnchor::constant( **height_val)
    //     }else{
    //         (p_calc_size_sa, &h).then(
    //             move|p_calc_size: &Vector2<f64>, sa_h: &GenericSizeAnchor| {
    //                let p_calc_size = *p_calc_size;

    //                // TODO  如果根 parent 无关 不是百分比  那么 不监听 parent

    //                 sa_h.map(move |h:&GenericSize|->f64{
    //                    calculation_h(&p_calc_size, h)
    //                }).into()

    //            } )
    //     };

    //     (&width, &height).map(|w,h|{
    //         Vector2::<f64>::new(*w,*h)
    //     }).into()

    // });

    //NOTE used for suggest in current cassowary , and children [suggest_calculated_size]
    let cass_or_calc_size =
        (&cass_or_calc_width, &cass_or_calc_height).map(|w, h| Vector2::<Precision>::new(*w, *h));

    let calculated_origin = (
        &p_cass_p_size_sa,
        &p_calculated.origin,
        &p_calculated.align,
        &cass_or_calc_size,
        &origin_x,
        &origin_y,
    )
        .then(
            move |&p_calc_size: &Vector2<Precision>,
                  &p_calc_origin: &Translation3<Precision>,
                  &p_calc_align: &Translation3<Precision>,
                  &calc_size: &Vector2<Precision>,
                  origin_x: &GenericSizeAnchor,
                  origin_y: &GenericSizeAnchor| {
                let _enter = trace_span!("-> [ calculated_origin ] recalculation..",).entered();

                (&**origin_x, &**origin_y)
                    .map(move |ox: &GenericSize, oy: &GenericSize| {
                        calculation_origin(
                            p_calc_size,
                            p_calc_origin,
                            p_calc_align,
                            calc_size,
                            ox,
                            oy,
                        )
                    })
                    .into()
            },
        );

    let calculated_align:StateAnchor<Translation3<Precision>> = (&p_cass_p_size_sa,&p_calculated.origin, &p_calculated.align, &align_x, &align_y).then(
                move |&p_calc_size: &Vector2<Precision>,&p_calc_origin:&Translation3<Precision>,&p_calc_align:&Translation3<Precision>, align_x: &GenericSizeAnchor, align_y: &GenericSizeAnchor| {
                    // let p_calc_size= *p_calc_size;




                    let _enter = trace_span!(
                        "-> [ calculated_align ] recalculation..(&p_calculated.size, &layout.align.watch()).map ",
                        ).entered();
                    (&**align_x ,&**align_y).map(move|ax:&GenericSize,ay:&GenericSize|{
                        calculation_align(p_calc_size,p_calc_origin,p_calc_align, ax,ay)

                    }).into()
                },
            );

    let coordinates_trans =
                (&calculated_origin, &calculated_align).map(move |origin:&Translation3<Precision>, align:&Translation3<Precision>| {

                    let _span =trace_span!(
                        "-> [ coordinates_trans ] recalculation..(&calculated_origin, &calculated_align).map ",
                        );

                    let _g = _span.enter();

                    let ff =  origin * align;
                    drop(_g);
                    trace!("coordinates_trans : {:?}",   &ff);

                    ff
                });

    let calculated_translation =
        (&cass_trans, &coordinates_trans).map(|cass, defined| cass * defined);
    // let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());
    let matrix = calculated_translation.map(|translation| translation.to_homogeneous().into());
    //TODO suppot use_size blend_origin(0~1) blend_align(0~1) def:0  blend_origin_x ...
    // let matrix = (&cass_trans,&calculated_origin).map(|cass,origin| (origin*cass).to_homogeneous().into());
    // let matrix = (&cass_trans,&coordinates_trans).map(|cass,defined| (cass).to_homogeneous().into());

    // @styles calculation ─────────────────────────────────────────────────────────────────
    // ────────────────────────────────────────────────────────────────────────────────

    let loc_styles = (&cass_or_calc_width,&cass_or_calc_height, &matrix).map( move |w,h, mat4: &Mat4| {
                            trace!( "------------size: w:{:?}  h:{:?}  , matrix: {}", &w,&h,CssTransform::from(*mat4) );

                        { let _ender = trace_span!(
                                    "-> [ loc_styles ] recalculation..(&calculated_size, &matrix).map ",
                                    ).entered();

                            trace!("loc_styles calculting ===============---------------------================-----------");
                            // log::trace!("-> [ loc_styles ] recalculation..(&calculated_size, &matrix).map ");



                            // TODO use  key 更新 s(),
                            // s().w(px(*w)).h(px(*h)).transform(*mat4)
                            s().transform(*mat4)

                        }


            });

    let world = (&p_calculated.world, &calculated_translation).map(|pw, t| pw * t);

    LayoutCalculated {
        // suggest_size: suggest_calculated_size,
        size_constraints,
        cassowary_inherited_generals_sa: current_cassowary_inherited_generals_sa,
        cass_or_calc_size,
        origin: calculated_origin,
        align: calculated_align,
        translation: calculated_translation,
        coordinates_trans,
        cass_trans,
        matrix,
        // • • • • •
        loc_styles,
        world,
    }
}
fn calculation_w_logiclength(p_calc_size: Vector2<Precision>, l: &LogicLength) -> Precision {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => p_calc_size.x * v * 0.01,
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => {
                calculation_w_logiclength(p_calc_size, a) * b.into_inner()
            }
            emg_common::CalcOp::Add { a, b } => {
                calculation_w_logiclength(p_calc_size, a)
                    + calculation_w_logiclength(p_calc_size, b)
            }
        },
    }
}
fn calculation_h_logiclength(p_calc_size: Vector2<Precision>, l: &LogicLength) -> Precision {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => p_calc_size.y * v * 0.01,
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => {
                calculation_h_logiclength(p_calc_size, a) * b.into_inner()
            }
            emg_common::CalcOp::Add { a, b } => {
                calculation_h_logiclength(p_calc_size, a)
                    + calculation_w_logiclength(p_calc_size, b)
            }
        },
    }
}
fn calculation_w(p_calc_size: Vector2<Precision>, w: &GenericSize) -> Precision {
    trace!("calculation_w");
    match w {
        GenericSize::None => 0.0,

        GenericSize::Length(logic_l) => calculation_w_logiclength(p_calc_size, logic_l),
        // GenericSize::Percentage(pc) => p_calc_size.x * pc.value()*0.01,
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => calculation_w(p_calc_size, a) * b.into_inner(),
            emg_common::CalcOp::Add { a, b } => {
                calculation_w(p_calc_size, a) + calculation_w(p_calc_size, b)
            }
        },
        //TODO 实现 parent 的parent 需要 p_calc_size 保存 parent的 p_calc_size
        GenericSize::Parent(type_name) => match type_name.as_str() {
            "CssWidth" => p_calc_size.x,
            "CssHeight" => p_calc_size.y,
            other => {
                panic!("current not implemented for GenericSize::Parent({other})");
            }
        },
    }
}
fn calculation_h(p_calc_size: Vector2<Precision>, h: &GenericSize) -> Precision {
    trace!("calculation_h");

    match h {
        GenericSize::None => 0.0,
        GenericSize::Length(logic_l) => calculation_h_logiclength(p_calc_size, logic_l),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => calculation_h(p_calc_size, a) * b.into_inner(),
            emg_common::CalcOp::Add { a, b } => {
                calculation_h(p_calc_size, a) + calculation_h(p_calc_size, b)
            }
        },
        GenericSize::Parent(type_name) => match type_name.as_str() {
            "CssWidth" => p_calc_size.x,
            "CssHeight" => p_calc_size.y,
            other => {
                panic!("current not implemented for GenericSize::Parent({other})");
            }
        },
    }
}

fn calculation_align(
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
    align_x: &GenericSize,
    align_y: &GenericSize,
) -> Translation3<Precision> {
    trace!("calculation_align");

    calculation_align_x(p_calc_size, p_calc_origin, p_calc_align, align_x)
        * calculation_align_y(p_calc_size, p_calc_origin, p_calc_align, align_y)
}
fn calculation_align_x_logiclength(
    p_calc_size: Vector2<Precision>,
    l: &LogicLength,
) -> Translation3<Precision> {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => Translation3::<Precision>::new(v, 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<Precision>::new(p_calc_size.x * v * 0.01, 0., 0.)
                }
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => calculation_align_x_logiclength(p_calc_size, a)
                .vector
                .scale(b.into_inner())
                .into(),
            emg_common::CalcOp::Add { a, b } => {
                calculation_align_x_logiclength(p_calc_size, a)
                    * calculation_align_x_logiclength(p_calc_size, b)
            }
        },
    }
}

fn calculation_align_y_logiclength(
    p_calc_size: Vector2<Precision>,
    l: &LogicLength,
) -> Translation3<Precision> {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => Translation3::<Precision>::new(0., v, 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<Precision>::new(0., p_calc_size.y * v * 0.01, 0.)
                }
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => calculation_align_y_logiclength(p_calc_size, a)
                .vector
                .scale(b.into_inner())
                .into(),
            emg_common::CalcOp::Add { a, b } => {
                calculation_align_y_logiclength(p_calc_size, a)
                    * calculation_align_y_logiclength(p_calc_size, b)
            }
        },
    }
}

fn calculation_align_x(
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
    align_x: &GenericSize,
) -> Translation3<Precision> {
    trace!("calculation_align");

    match align_x {
        GenericSize::None => Translation3::<Precision>::default(),
        GenericSize::Length(logic_l) => calculation_align_x_logiclength(p_calc_size, logic_l),
        // GenericSize::Percentage(pc) => Translation3::<Precision>::new(p_calc_size.x * pc.value()*0.01, 0., 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_common::CalcOp::Mul { a, b } => {
                    // let scaling = nalgebra::Similarity3::from_scaling(b.into_inner());
                    calculation_align_x(p_calc_size, p_calc_origin, p_calc_align, a)
                        .vector
                        .scale(b.into_inner())
                        .into()
                }
                emg_common::CalcOp::Add { a, b } => {
                    calculation_align_x(p_calc_size, p_calc_origin, p_calc_align, a)
                        * calculation_align_x(p_calc_size, p_calc_origin, p_calc_align, b)
                }
            }
        }
        GenericSize::Parent(type_name) => {
            let parent_val =
                get_parent_calculated(type_name, p_calc_size, p_calc_origin, p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => Translation3::<Precision>::new(v, 0., 0.),
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                ParentCalculated::T3(t) => t,
            }
        }
    }
}
fn calculation_align_y(
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
    align_y: &GenericSize,
) -> Translation3<Precision> {
    trace!("calculation_align");

    match align_y {
        GenericSize::None => Translation3::<Precision>::default(),
        GenericSize::Length(logic_l) => calculation_align_y_logiclength(p_calc_size, logic_l),
        // GenericSize::Percentage(pc) => Translation3::<Precision>::new(0., p_calc_size.y * pc.value()*0.01, 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => {
                calculation_align_y(p_calc_size, p_calc_origin, p_calc_align, a)
                    .vector
                    .scale(b.into_inner())
                    .into()
            }
            emg_common::CalcOp::Add { a, b } => {
                calculation_align_y(p_calc_size, p_calc_origin, p_calc_align, a)
                    * calculation_align_y(p_calc_size, p_calc_origin, p_calc_align, b)
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val =
                get_parent_calculated(type_name, p_calc_size, p_calc_origin, p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => Translation3::<Precision>::new(0., v, 0.),
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                ParentCalculated::T3(t) => t,
            }
        }
    }
}

pub fn calculation_origin(
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
    calc_size: Vector2<Precision>,
    origin_x: &GenericSize,
    origin_y: &GenericSize,
) -> Translation3<Precision> {
    trace!("calculation_origin");
    calculation_origin_x(
        p_calc_size,
        p_calc_origin,
        p_calc_align,
        calc_size,
        origin_x,
    ) * calculation_origin_y(
        p_calc_size,
        p_calc_origin,
        p_calc_align,
        calc_size,
        origin_y,
    )
}

fn calculation_origin_x_logiclength(
    calc_size: Vector2<Precision>,
    l: &LogicLength,
) -> Translation3<Precision> {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => {
                    Translation3::<Precision>::new(-v, 0., 0.)
                }
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<Precision>::new(-(calc_size.x * v * 0.01), 0., 0.)
                }
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => calculation_origin_x_logiclength(calc_size, a)
                .vector
                .scale(b.into_inner())
                .into(),
            emg_common::CalcOp::Add { a, b } => {
                calculation_origin_x_logiclength(calc_size, a)
                    * calculation_origin_x_logiclength(calc_size, b)
            }
        },
    }
}

fn calculation_origin_y_logiclength(
    calc_size: Vector2<Precision>,
    l: &LogicLength,
) -> Translation3<Precision> {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => {
                    Translation3::<Precision>::new(0., -v, 0.)
                }
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<Precision>::new(0., -(calc_size.y * v * 0.01), 0.)
                }
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => calculation_origin_y_logiclength(calc_size, a)
                .vector
                .scale(b.into_inner())
                .into(),
            emg_common::CalcOp::Add { a, b } => {
                calculation_origin_y_logiclength(calc_size, a)
                    * calculation_origin_y_logiclength(calc_size, b)
            }
        },
    }
}

pub fn calculation_origin_x(
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
    calc_size: Vector2<Precision>,
    origin_x: &GenericSize,
) -> Translation3<Precision> {
    trace!("calculation_origin");

    match origin_x {
        GenericSize::None => Translation3::<Precision>::default(),
        GenericSize::Length(logic_l) => calculation_origin_x_logiclength(calc_size, logic_l),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => {
                calculation_origin_x(p_calc_size, p_calc_origin, p_calc_align, calc_size, a)
                    .vector
                    .scale(b.into_inner())
                    .into()
            }
            emg_common::CalcOp::Add { a, b } => {
                calculation_origin_x(p_calc_size, p_calc_origin, p_calc_align, calc_size, a)
                    * calculation_origin_x(p_calc_size, p_calc_origin, p_calc_align, calc_size, b)
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val =
                get_parent_calculated(type_name, p_calc_size, p_calc_origin, p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => Translation3::<Precision>::new(v, 0., 0.),
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                //TODO check is only use t.x
                ParentCalculated::T3(t) => t,
            }
        }
    }
}
pub fn calculation_origin_y(
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
    calc_size: Vector2<Precision>,
    origin_y: &GenericSize,
) -> Translation3<Precision> {
    trace!("calculation_origin");

    match origin_y {
        GenericSize::None => Translation3::<Precision>::default(),

        GenericSize::Length(logic_l) => calculation_origin_y_logiclength(calc_size, logic_l),
        // GenericSize::Percentage(pc) => Translation3::<f64>::new(0., -(calc_size.y * pc.value()*0.01), 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => match calc_op.as_ref() {
            emg_common::CalcOp::Mul { a, b } => {
                calculation_origin_y(p_calc_size, p_calc_origin, p_calc_align, calc_size, a)
                    .vector
                    .scale(b.into_inner())
                    .into()
            }
            emg_common::CalcOp::Add { a, b } => {
                calculation_origin_y(p_calc_size, p_calc_origin, p_calc_align, calc_size, a)
                    * calculation_origin_y(p_calc_size, p_calc_origin, p_calc_align, calc_size, b)
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val =
                get_parent_calculated(type_name, p_calc_size, p_calc_origin, p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => Translation3::<Precision>::new(0., v, 0.),
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                //TODO check is only use t.y
                ParentCalculated::T3(t) => t,
            }
        }
    }
}

#[derive(Clone, Debug, From)]
enum ParentCalculated {
    Number(Precision),
    V2(Vector2<Precision>),
    T3(Translation3<Precision>),
}
fn get_parent_calculated(
    type_name: &TypeName,
    p_calc_size: Vector2<Precision>,
    p_calc_origin: Translation3<Precision>,
    p_calc_align: Translation3<Precision>,
) -> ParentCalculated {
    match type_name.as_str() {
        "CssWidth" => p_calc_size.x.into(),
        "CssHeight" => p_calc_size.y.into(),
        "OriginX" => p_calc_origin.vector.x.into(),
        "OriginY" => p_calc_origin.vector.y.into(),
        "Origin" => p_calc_origin.into(),
        "AlignX" => p_calc_align.vector.x.into(),
        "AlignY" => p_calc_align.vector.y.into(),
        "Align" => p_calc_align.into(),
        other => {
            panic!("current not implemented for GenericSize::Parent({other})");
        }
    }
}
