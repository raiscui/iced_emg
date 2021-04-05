/*
* @Author: Rais
* @Date: 2021-03-29 17:30:58
 * @LastEditTime: 2021-04-03 16:53:36
 * @LastEditors: Rais
* @Description:
*/
use crate::{
     EdgeDataWithParent, EdgeItem, GenericLoc, GenericSize, GenericWH, Layout,
    LayoutCalculated, Mat4, Size2, Trans3,
};

use emg_state::{ StateMultiAnchor, StateVar};
use seed_styles as styles;
use styles::{px, s, CssHeightTrait, CssTransform, CssTransformTrait, CssWidthTrait, };
use tracing::{Level,  span, trace,trace_span};

// ────────────────────────────────────────────────────────────────────────────────
pub fn layout_calculating(
    id: StateVar<String>,
    parent_edge_item_sv: &StateVar<Option<EdgeItem>>,
    layout: &Layout,
) -> LayoutCalculated {

   
        

        
    let _span_ = span!(Level::TRACE, "layout_calculating",%id).entered();
    let p_calc_size_sa = parent_edge_item_sv.watch().then(move |opt_p_sa: &Option<EdgeItem>| {
        let _span_ = span!(Level::TRACE, "p_calculated recalculation",%id).entered();
        trace!("parent_sv change");
        if let Some(pei) = opt_p_sa {
            let EdgeDataWithParent {
                parent: _,
                current: ped,
            } = pei.as_edge_data_with_parent().expect(
                "EdgeItem must be `EdgeItem::EdgeDataWithParent`, other type can't make child edge",
            );

            trace!("ped get");

            // TODO how to use Pre-acquired G_STATE_STORE Optimize performance

            ped.calculated.size.map(|x| *x).into()
        } else {
            todo!("build child edge with None parent,is not impl now.  impl hide or display none")
        }
    });
    // let p_calc_size_sa:StateAnchor<Size2> = p_calculated.then(|p_calc:&LayoutCalculated|p_calc.size.clone().into());
    
    let calculated_size = (&p_calc_size_sa, &layout.size.watch()).map(
         move   |p_calc_size: &Size2, wh: &GenericWH| {
                
                // TODO  如果根 parent 无关 不是百分比  那么 不监听 parent
            let _enter = span!(Level::TRACE, 
                "-> [ calculated_size ] recalculation..(&p_calculated.size, &layout.size.watch()).map ",
                %id).entered();

            calculation_size(p_calc_size, wh)
        },
    );
    let calculated_origin = (&calculated_size, &layout.origin.watch()).map(
        move       |calc_size: &Size2, origin: &GenericLoc| {
            let _enter = span!(Level::TRACE, 
                "-> [ calculated_origin ] recalculation..(&calculated_size, &layout.origin.watch()).map ",
                %id).entered();

            calculation_origin(calc_size, origin)
        },
    );
    let calculated_align = (&p_calc_size_sa, &layout.align.watch()).map(
          move  |p_calc_size: &Size2, align: &GenericLoc| {
            let _enter = span!(Level::TRACE, 
                "-> [ calculated_align ] recalculation..(&p_calculated.size, &layout.align.watch()).map ",
                %id).entered();

            calculation_align(p_calc_size, align)
        },
    );

    let coordinates_trans =
        (&calculated_origin, &calculated_align).map(move |origin, align| {
            
            let _span =span!(Level::TRACE, 
                "-> [ coordinates_trans ] recalculation..(&calculated_origin, &calculated_align).map ",
                %id);
                
            let _g = _span.enter();

            let ff =  align * origin;
            drop(_g);
            trace!("coordinates_trans : {:?}",   &ff);

            ff
        });

    let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());

    // @styles calculation ─────────────────────────────────────────────────────────────────
    // ────────────────────────────────────────────────────────────────────────────────
        

    let loc_styles = (&calculated_size, &matrix).map( move |calc_size: &Size2, mat4: &Mat4| {
                    log::trace!( "------------size: {:?}  , matrix: {}", &calc_size, CssTransform::from(*mat4) );

                   { let _ender = span!(Level::TRACE, 
                            "-> [ loc_styles ] recalculation..(&calculated_size, &matrix).map ",
                            ).entered();

                    trace!("loc_styles calculting ===============---------------------================-----------");
                    // log::trace!("-> [ loc_styles ] recalculation..(&calculated_size, &matrix).map ");



                    // TODO use  key 更新 s(),
                    s().w(px(calc_size.x)).h(px(calc_size.y)).transform(*mat4)
               
}
        
                
            
        });


   

    LayoutCalculated {
        size: calculated_size,
        origin: calculated_origin,
        align: calculated_align,
        coordinates_trans,
        matrix,
        // • • • • •
        loc_styles,
    }
}

// #[topo::nested]
// pub fn calculated_layout_to_styles(
//     layout_calculated: &StateAnchor<LayoutCalculated>,
// ) -> (
//     StateVar<Style>,
//     StateAnchor<Style>,
//     StateAnchor<Style>,
//     StateAnchor<String>,
// ) {
//     let other_styles = use_state(s());
//     let loc_styles = layout_calculated.then(|calc_layout: &LayoutCalculated| {
//         (&calc_layout.size, &calc_layout.matrix)
//             .map(|size: &Size2, mat4: &Mat4| {
//                 trace!(
//                     "loc_styles change ---------------------------------------------------"
//                 );
//                 trace!("{}", mat4);
//                 trace!("{}", CssTransform::from(*mat4));
//                 trace!(
//                     "---------------------------------------------------------------------"
//                 );
//                 // TODO use  key 更新 s(),
//                 s().w(px(size.x)).h(px(size.y)).transform(*mat4)
//             })
//             .into()
//     });
//     let styles = (&other_styles.watch(), &loc_styles).map(|other: &Style, loc: &Style| {
//         //    s().custom_style_ref(loc_s.clone())
//         //         .custom_style_ref(o_s.clone());
//         // other 覆盖叠加到 loc
//         loc.clone().custom_style(other.clone())
//     });
//     let style_string = styles.map(seed_styles::Style::render);
//     (other_styles, loc_styles, styles, style_string)
// }

fn calculation_size(p_calc_size: &Size2, wh: &GenericWH) -> Size2 {
    trace!("calculation_size");
    let calc_w = match wh.w {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
            }
        }
        GenericSize::Percentage(pc) => p_calc_size.x * pc.0*0.01,
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
    };
    let calc_h = match wh.h {
        GenericSize::Percentage(pc) => p_calc_size.x * pc.0*0.01,

        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
            }
        }
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
    };

    Size2::new(calc_w, calc_h)
}

fn calculation_align(p_calc_size: &Size2, align: &GenericLoc) -> Trans3 {
    trace!("calculation_align");

    let trans_x = match align.x {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px => Trans3::new(v, 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
            }
        }
        GenericSize::Percentage(pc) => Trans3::new(p_calc_size.x * pc.0*0.01, 0., 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
    };
    let trans_y = match align.y {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px => Trans3::new(0., v, 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
            }
        }
        GenericSize::Percentage(pc) => Trans3::new(0., p_calc_size.y * pc.0*0.01, 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
    };
    trans_x * trans_y
}

pub fn calculation_origin(calc_size: &Size2, origin: &GenericLoc) -> Trans3 {
    trace!("calculation_origin");

    let trans_x = match origin.x {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px => Trans3::new(-v, 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
            }
        }
        GenericSize::Percentage(pc) => Trans3::new(-(calc_size.x * pc.0*0.01), 0., 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
    };
    let trans_y = match origin.y {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px => Trans3::new(0., -v, 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
            }
        }
        GenericSize::Percentage(pc) => Trans3::new(0., -(calc_size.y * pc.0*0.01), 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
    };
    trans_x * trans_y
}
