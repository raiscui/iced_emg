
use std::rc::Rc;

/*
* @Author: Rais
* @Date: 2021-03-29 17:30:58
 * @LastEditTime: 2022-07-17 22:11:38
 * @LastEditors: Rais
* @Description:
*/
use crate::{EdgeData, GenericSize, GenericSizeAnchor, Layout, LayoutCalculated, Mat4, ccsa::CassowaryMap};

use float_cmp::{approx_eq, assert_approx_eq};
use cassowary::WeightedRelation;
use emg::EdgeIndex;
use emg_core::TypeName;
use emg_state::{StateAnchor, StateMultiAnchor, StateVar, topo, Anchor};
use nalgebra::{Translation3, Vector2};
use seed_styles as styles;
use styles::{ CssHeightTrait, CssTransform, CssTransformTrait, CssWidthTrait, LogicLength, px, s};
use tracing::{ trace,trace_span, warn, debug, debug_span, warn_span};
use derive_more::From;

use self::cassowary_calc::cassowary_calculation;

mod cassowary_calc;

// ────────────────────────────────────────────────────────────────────────────────
    
// #[track_caller]
#[topo::nested]
pub fn layout_calculating<Ix>(
    _id:StateVar< StateAnchor<EdgeIndex<Ix>>>,
    path_edgedata: &EdgeData,//parent
    current_cassowary_map:&Rc<CassowaryMap>,
    layout: &StateAnchor<Layout>,
) -> LayoutCalculated 
where 
    Ix: 'static + std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default + std::cmp::Ord+ std::fmt::Display 
    
    {
    let _span_ = trace_span!( "->[ layout_calculating ] ").entered();
    let _debug_span_ = debug_span!( "->[ layout_calculating ] ").entered();
    
            let EdgeData{
                        calculated:p_calculated,
                        cassowary_calculated_vars:p_calculated_vars,
                        cassowary_map:p_cassowary_map,
                        // cassowary_calculated_layout,
                .. }=path_edgedata;
            // ─────────────────────────────────────────────────────────────────

            let p_calc_size_sa = &p_calculated.real_size;
            //NOTE loop !!
            // let p_calc_size_sa = &cassowary_calculated_layout.map(|(w,h)|Vector2::<f64>::new(*w,*h));

            // ─────────────────────────────────────────────────────────────────
            let w = layout.then(|l:&Layout|l.w.watch().into());
            let h = layout.then(|l:&Layout|l.h.watch().into());
            let origin_x = layout.then(|l:&Layout|l.origin_x.watch().into());
            let origin_y = layout.then(|l:&Layout|l.origin_y.watch().into());
            let align_x = layout.then(|l:&Layout|l.align_x.watch().into());
            let align_y = layout.then(|l:&Layout|l.align_y.watch().into());
            // ─────────────────────────────────────────────────────────────────
            let width_var  =current_cassowary_map.var("width").unwrap();
            let height_var  =current_cassowary_map.var("height").unwrap();

            let p_cassowary_map2 = p_cassowary_map.clone();
            let sa_w = w.then(|w|w.get_anchor());
            let sa_h = h.then(|h|h.get_anchor());

            let current_cassowary_map2 = current_cassowary_map.clone();
            let size_constraints = 
                (&sa_w,&sa_h).map(move |w:&GenericSize,h:&GenericSize|{
                    let mut size_constraints = vec![];

                    let mut skip = false;

                    if let Ok(ww)  = w.try_get_length_value(){
                        if approx_eq!(f64,ww,0.0,(0.1,2)) {
                            size_constraints.push(  width_var  | WeightedRelation::EQ(cassowary::strength::WEAK) | 0.0);
                            skip = true;
                            
                        }
                    }
                    if let Ok(hh)  = h.try_get_length_value(){
                        if approx_eq!(f64,hh,0.0,(0.1,2)) {
                            size_constraints.push(  height_var  | WeightedRelation::EQ(cassowary::strength::WEAK) | 0.0);
                            skip = true;
                            
                        }
                    }

                    if !skip{
                        size_constraints.extend([
                            //NOTE will use for parent cassowary
                            width_var  | WeightedRelation::EQ(cassowary::strength::MEDIUM * 0.5) | cassowary_calculation("width", &p_cassowary_map2, w),
                            height_var | WeightedRelation::EQ(cassowary::strength::MEDIUM * 0.5) | cassowary_calculation("height",&p_cassowary_map2, h),
                            // • • • • •
                        ]);
                    }

                    
                    size_constraints.extend([
                        
                        current_cassowary_map2.var("right").unwrap() - current_cassowary_map2.var("left").unwrap()| WeightedRelation::EQ(cassowary::strength::REQUIRED) | width_var,
                        current_cassowary_map2.var("bottom").unwrap() - current_cassowary_map2.var("top").unwrap()| WeightedRelation::EQ(cassowary::strength::REQUIRED) | height_var,

                        current_cassowary_map2.var("bottom").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | current_cassowary_map2.var("top").unwrap(),
                        current_cassowary_map2.var("right").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | current_cassowary_map2.var("left").unwrap(),
                        width_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                        height_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                        current_cassowary_map2.var("top").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
                        current_cassowary_map2.var("left").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
                    ]);

                    size_constraints
                });
            

            let calculated_size = (p_calc_size_sa, &w,&h).then(
                move|p_calc_size: &Vector2<f64>, sa_w: &GenericSizeAnchor,sa_h:&GenericSizeAnchor| {
                   let p_calc_size = *p_calc_size;   
                   
                   // TODO  如果根 parent 无关 不是百分比  那么 不监听 parent
                   let _enter = trace_span!( 
                       "-> [ calculated_size ] recalculation..(&p_calculated.size, &layout.size.watch()).map ",
                       ).entered();
                    (&**sa_w,&**sa_h).map(move |w:&GenericSize,h:&GenericSize|->Vector2<f64>{
                       //TODO check editor display error 
                       let new_size = Vector2::<f64>::new(calculation_w(&p_calc_size, w), calculation_h(&p_calc_size, h));
                       trace!("new size: {}",&new_size);
                       new_size
                   }).into()
               },
            );
            let (calculated_width,calculated_height) = calculated_size.map(|size|{
                (size.x,size.y)
            }).split();
            
            // ────────────────────────────────────────────────────────────────────────────────

            let _debug_span_2 =_debug_span_.clone();
            let width = p_calculated_vars.then(move|p_vars|{
                let _debug_span_ = warn_span!( "->[ get self prop calculated value ] ").entered();
                // warn!("p_vars: {:?},  \n get :{:?}",&p_vars,&width_var);
                // • • • • •
                //NOTE 如果 是 root下面的第一阶层节点,如果没定义cassowary constraint 或者定义少量、不涉及某些 Id element, p_calculated_vars 很可能是无 or 不全面的,
                if let Some(w) = p_vars.get(&width_var).map(|(val,_)| **val){
                    Anchor::constant(w)
                }else{
                    calculated_width.get_anchor()
                }
            });
            let height = p_calculated_vars.then(move|p_vars|{

                if let Some(h) = p_vars.get(&height_var).map(|(val,_)| **val){
                    Anchor::constant(h)
                 }else{
                    calculated_height.get_anchor()
                }

            });
            let top_var  =current_cassowary_map.var("top").unwrap();
            let top = p_calculated_vars.map(move|p_vars|{
                p_vars.get(&top_var).map(|(val,_)| **val)
            });
            let left_var  =current_cassowary_map.var("left").unwrap();
            let left = p_calculated_vars.map(move|p_vars|{
                p_vars.get(&left_var).map(|(val,_)| **val)
            });
            let bottom_var  =current_cassowary_map.var("bottom").unwrap();
            let bottom = p_calculated_vars.map(move|p_vars|{
                p_vars.get(&bottom_var).map(|(val,_)| **val)
            });
            
            let right_var  =current_cassowary_map.var("right").unwrap();
            let right = p_calculated_vars.map(move|p_vars|{
                p_vars.get(&right_var).map(|(val,_)| **val)
            });

            //TODO 如果 父层 cassowary 不涉及到 此 element , 那么就需要 进行 原定位计算
            let cass_trans:StateAnchor<Translation3<f64>>  =  (&width,&height,&top,&left,&bottom,&right).map(move|w:&f64,h:&f64,opt_t:&Option<f64>,opt_l:&Option<f64>,opt_b: &Option<f64>,opt_r: &Option<f64>,|{
                let _span = warn_span!("cass_trans calculting map").entered();
                warn!("[cass_trans] t:{:?} l:{:?} b:{:?} r:{:?}",opt_t,opt_l,opt_b,opt_r);
                match (opt_t,opt_l,opt_b,opt_r) {
                    (None, None, None, None) => {
                        Translation3::<f64>::new(0.,0.,0.0)

                    },
                    (None, None, None, Some(r)) => {
                        Translation3::<f64>::new(r-w,0.,0.0)

                    },
                    (None, None, Some(b), None) => {
                        Translation3::<f64>::new(0.,b-h,0.0)

                    },
                    (None, None, Some(b), Some(r)) => {
                        Translation3::<f64>::new(r-w,b-h,0.0)

                    },
                    (None, Some(l), None, None) => {
                        Translation3::<f64>::new(*l,0.,0.0)

                    },
                    (None, Some(l), None, Some(r)) => {
                        assert_approx_eq!(f64,r-l,*w,(0.1,2));
                        Translation3::<f64>::new(*l,0.,0.0)

                    },
                    (None, Some(l), Some(b), None) => {
                        Translation3::<f64>::new(*l,b-h,0.0)
                    },
                    (None, Some(l), Some(b), Some(r)) => {
                        assert_approx_eq!(f64,r-l,*w,(0.1,2));
                        Translation3::<f64>::new(*l,b-h,0.0)
                    },
                    (Some(t), None, None, None) => {
                        Translation3::<f64>::new(0.,*t,0.0)

                    },
                    (Some(t), None, None, Some(r)) => {
                        Translation3::<f64>::new(r-w,*t,0.0)
                    },
                    (Some(t), None, Some(b), None) => {
                        assert_approx_eq!(f64,b-t,*h,(0.1,2));
                        Translation3::<f64>::new(0.0,*t,0.0)
                    },
                    (Some(t), None, Some(b), Some(r)) => {
                        assert_approx_eq!(f64,b-t,*h,(0.1,2));
                        Translation3::<f64>::new(r-w,*t,0.0)
                        

                    },
                    (Some(t), Some(l), None, None) => {
                        Translation3::<f64>::new(*l,*t,0.0)

                    },
                    (Some(t), Some(l), None, Some(r)) => {
                        warn!("t:{} l:{} r:{}",t,l,r);
                           //TODO remove this if release
                           assert_approx_eq!(f64,r-l,*w,(0.1,2));
                           Translation3::<f64>::new(*l,*t,0.0)
   
                    },
                    (Some(t), Some(l), Some(b), None) => {
                           //TODO remove this if release
                           assert_approx_eq!(f64,b-t,*h,(0.1,2));
                           Translation3::<f64>::new(*l,*t,0.0)
                    },
                    (Some(t), Some(l), Some(b), Some(r)) => {
                        //TODO remove this if release

                        // let mut buffer = ryu::Buffer::new();
                        // let b_t = buffer.format_finite(b-t);
                        // let mut buffer2 = ryu::Buffer::new();
                        // let h_ = buffer2.format_finite(*h);
                        warn!("b-t:{:.10} h:{:.10}",b-t,*h);

                        // assert_approx_eq!(f64,b-t,*h,(0.1,2));
                        // assert_approx_eq!(f64,r-l,*w,(0.1,2));
                        Translation3::<f64>::new(*l,*t,0.0)

                    },
                }
                
            });



            
            




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

            let real_size = (&width, &height).map(|w,h|Vector2::<f64>::new(*w,*h));

            let calculated_origin = (p_calc_size_sa,&p_calculated.origin, &p_calculated.align,&real_size, &origin_x,&origin_y).then(
                move |p_calc_size: &Vector2<f64>,p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>,size:&Vector2<f64>, origin_x: &GenericSizeAnchor,origin_y: &GenericSizeAnchor| {

      
                    let calc_size = *size;
                    let p_calc_size = *p_calc_size;   
                    let p_calc_origin = *p_calc_origin;
                    let p_calc_align = *p_calc_align;
                    let _enter = trace_span!( 
                        "-> [ calculated_origin ] recalculation..",
                        ).entered();

                        (&**origin_x, &**origin_y).map(move|ox:&GenericSize,oy:&GenericSize|{
                            calculation_origin(&p_calc_size,&p_calc_origin,&p_calc_align,&calc_size, ox,oy)

                        }).into()

                },
            );

            let calculated_align:StateAnchor<Translation3<f64>> = (p_calc_size_sa,&p_calculated.origin, &p_calculated.align, &align_x, &align_y).then(
                move |p_calc_size: &Vector2<f64>,p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>, align_x: &GenericSizeAnchor, align_y: &GenericSizeAnchor| {
                    // let p_calc_size= *p_calc_size;



                    let p_calc_size = *p_calc_size;   
                    let p_calc_origin = *p_calc_origin;
                    let p_calc_align = *p_calc_align;
                    
                    let _enter = trace_span!( 
                        "-> [ calculated_align ] recalculation..(&p_calculated.size, &layout.align.watch()).map ",
                        ).entered();
                    (&**align_x ,&**align_y).map(move|ax:&GenericSize,ay:&GenericSize|{
                        calculation_align(&p_calc_size,&p_calc_origin,&p_calc_align, ax,ay)

                    }).into()
                },
            );

            let coordinates_trans =
                (&calculated_origin, &calculated_align).map(move |origin:&Translation3<f64>, align:&Translation3<f64>| {
                    
                    let _span =trace_span!( 
                        "-> [ coordinates_trans ] recalculation..(&calculated_origin, &calculated_align).map ",
                        );
                        
                    let _g = _span.enter();

                    let ff =  origin * align;
                    drop(_g);
                    trace!("coordinates_trans : {:?}",   &ff);

                    ff
                });

            // let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());
            let matrix = (&cass_trans,&coordinates_trans).map(|cass,defined| (defined* cass).to_homogeneous().into());
            //TODO suppot use_size blend_origin(0~1) blend_align(0~1) def:0  blend_origin_x ...
            // let matrix = (&cass_trans,&calculated_origin).map(|cass,origin| (origin*cass).to_homogeneous().into());
            // let matrix = (&cass_trans,&coordinates_trans).map(|cass,defined| (cass).to_homogeneous().into());

            // @styles calculation ─────────────────────────────────────────────────────────────────
            // ────────────────────────────────────────────────────────────────────────────────
                
            let loc_styles = (&width,&height, &matrix).map( move |w,h, mat4: &Mat4| {
                            trace!( "------------size: w:{:?}  h:{:?}  , matrix: {}", &w,&h,CssTransform::from(*mat4) );

                        { let _ender = trace_span!( 
                                    "-> [ loc_styles ] recalculation..(&calculated_size, &matrix).map ",
                                    ).entered();

                            trace!("loc_styles calculting ===============---------------------================-----------");
                            // log::trace!("-> [ loc_styles ] recalculation..(&calculated_size, &matrix).map ");



                            // TODO use  key 更新 s(),
                            s().w(px(*w)).h(px(*h)).transform(*mat4)
                            // s().transform(*mat4)
                    
                        }
                
                        
            });

            LayoutCalculated {
                suggest_size: calculated_size,
                size_constraints,
                real_size,
                origin: calculated_origin,
                align: calculated_align,
                coordinates_trans,
                cass_trans,
                matrix,
                // • • • • •
                loc_styles,
            }
    
}
fn calculation_w_logiclength(p_calc_size: &Vector2<f64>, l:&LogicLength)->f64 {

    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px |styles::Unit::Empty => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => p_calc_size.x * v*0.01,
                
            }
        },
        LogicLength::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => calculation_w_logiclength(p_calc_size,a)*b.into_inner(),
                emg_core::CalcOp::Add { a, b } => calculation_w_logiclength(p_calc_size,a)+calculation_w_logiclength(p_calc_size,b),
            }
        },
    }
}
fn calculation_h_logiclength(p_calc_size: &Vector2<f64>, l:&LogicLength)->f64 {

    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px |styles::Unit::Empty => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => p_calc_size.y * v*0.01,
                
            }
        },
        LogicLength::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => calculation_h_logiclength(p_calc_size,a)*b.into_inner(),
                emg_core::CalcOp::Add { a, b } => calculation_h_logiclength(p_calc_size,a)+calculation_w_logiclength(p_calc_size,b),
            }
        },
    }
}
fn calculation_w(p_calc_size: &Vector2<f64>, w: &GenericSize) -> f64 {
    trace!("calculation_w");
    match w {
        GenericSize::Length(logic_l) => {
            
            calculation_w_logiclength(p_calc_size,logic_l)
        },
        // GenericSize::Percentage(pc) => p_calc_size.x * pc.value()*0.01,
        
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        },
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                     calculation_w(p_calc_size, a)*b.into_inner()
                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_w(p_calc_size, a)+ calculation_w(p_calc_size,b)

                },
            }
        },
        //TODO 实现 parent 的parent 需要 p_calc_size 保存 parent的 p_calc_size 
        GenericSize::Parent(type_name) => {
            match type_name.as_str(){
                "CssWidth"=>{
                    p_calc_size.x
                }
                "CssHeight"=>{
                    p_calc_size.y
                }
                other=>{
                    panic!("current not implemented for GenericSize::Parent({})",other);
                }
            }
        }
    }


}
fn calculation_h(p_calc_size: &Vector2<f64>, h: &GenericSize) -> f64 {
    trace!("calculation_h");
    
    match h {

        GenericSize::Length(logic_l) => {
            calculation_h_logiclength(p_calc_size,logic_l)

        },
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        },
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                    calculation_h(p_calc_size, a)*b.into_inner()
                },
                emg_core::CalcOp::Add { a, b } => {
                      calculation_h(p_calc_size, a)+ calculation_h(p_calc_size,b)

                },
            }
        },
        GenericSize::Parent(type_name) => {
            match type_name.as_str(){
                "CssWidth"=>{
                    p_calc_size.x
                }
                "CssHeight"=>{
                    p_calc_size.y
                }
                other=>{
                    panic!("current not implemented for GenericSize::Parent({})",other);
                }
            }
        }
    }

}


fn calculation_align(p_calc_size: &Vector2<f64>,p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>, align_x: &GenericSize,align_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_align");

    calculation_align_x(p_calc_size, p_calc_origin,p_calc_align,align_x) * calculation_align_y(p_calc_size,p_calc_origin,p_calc_align, align_y)
}
fn calculation_align_x_logiclength(p_calc_size: &Vector2<f64>, l:&LogicLength)->Translation3<f64> {

    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px |styles::Unit::Empty => Translation3::<f64>::new(v, 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<f64>::new(p_calc_size.x * v*0.01, 0., 0.)
                }
                
            }
        },
        LogicLength::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => calculation_align_x_logiclength(p_calc_size,a).vector.scale(b.into_inner()).into(),
                emg_core::CalcOp::Add { a, b } => calculation_align_x_logiclength(p_calc_size,a)*
                calculation_align_x_logiclength(p_calc_size,b),
            }
        },
    }
}

fn calculation_align_y_logiclength(p_calc_size: &Vector2<f64>, l:&LogicLength)->Translation3<f64> {

    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px |styles::Unit::Empty => Translation3::<f64>::new(0., v, 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<f64>::new(0., p_calc_size.y * v*0.01, 0.)
                }
                
            }
        },
        LogicLength::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } =>  calculation_align_y_logiclength(p_calc_size,a).vector.scale(b.into_inner()).into(),
                emg_core::CalcOp::Add { a, b } => calculation_align_y_logiclength(p_calc_size,a)*calculation_align_y_logiclength(p_calc_size,b),
            }
        },
    }
}

fn calculation_align_x(p_calc_size: &Vector2<f64>, p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>,align_x: &GenericSize) -> Translation3<f64> {
    trace!("calculation_align");

     match align_x {
        GenericSize::Length(logic_l) => {
            calculation_align_x_logiclength(p_calc_size,logic_l)
        },
        // GenericSize::Percentage(pc) => Translation3::<f64>::new(p_calc_size.x * pc.value()*0.01, 0., 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        },
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {

                    // let scaling = nalgebra::Similarity3::from_scaling(b.into_inner());
                  calculation_align_x(p_calc_size, p_calc_origin,p_calc_align,a).vector.scale(b.into_inner()) .into()
                   
                    
                },
                emg_core::CalcOp::Add { a, b } => {
                      calculation_align_x(p_calc_size, p_calc_origin,p_calc_align, a)* calculation_align_x(p_calc_size, p_calc_origin,p_calc_align,b)
                }
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val = get_parent_calculated(type_name,p_calc_size, p_calc_origin,p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => {
                    Translation3::<f64>::new( v,0.,0.)
                },
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                ParentCalculated::T3(t) => *t,
            }
        }
    }
}
fn calculation_align_y(p_calc_size: &Vector2<f64>, p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>,align_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_align");

    
    match align_y {
        GenericSize::Length(logic_l) => {
            calculation_align_y_logiclength(p_calc_size, logic_l)
        },
        // GenericSize::Percentage(pc) => Translation3::<f64>::new(0., p_calc_size.y * pc.value()*0.01, 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        },
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                  
                  calculation_align_y(p_calc_size, p_calc_origin,p_calc_align,a).vector.scale(b.into_inner()).into()
                   

                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_align_y(p_calc_size, p_calc_origin,p_calc_align, a) * calculation_align_y(p_calc_size, p_calc_origin,p_calc_align,b)
                }
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val = get_parent_calculated(type_name,p_calc_size, p_calc_origin,p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => {
                    Translation3::<f64>::new( 0.,v,0.)
                },
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                ParentCalculated::T3(t) => *t,
            }
        }
    }
}

pub fn calculation_origin(p_calc_size: &Vector2<f64>, p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>,calc_size: &Vector2<f64>, origin_x: &GenericSize,origin_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_origin");
    calculation_origin_x(p_calc_size, p_calc_origin,p_calc_align,calc_size, origin_x) * calculation_origin_y(p_calc_size, p_calc_origin,p_calc_align,calc_size, origin_y)
}

fn calculation_origin_x_logiclength(calc_size: &Vector2<f64>, l:&LogicLength)-> Translation3<f64> {

    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px |styles::Unit::Empty => Translation3::<f64>::new(-v, 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<f64>::new(-(calc_size.x * v*0.01), 0., 0.)
                }
                
            }
        },
        LogicLength::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } =>  calculation_origin_x_logiclength(calc_size,a).vector.scale(b.into_inner()).into(),
                emg_core::CalcOp::Add { a, b } => calculation_origin_x_logiclength(calc_size,a)*calculation_origin_x_logiclength(calc_size,b),
            }
        },
    }
}

fn calculation_origin_y_logiclength(calc_size: &Vector2<f64>, l:&LogicLength)->Translation3<f64> {

    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px |styles::Unit::Empty => Translation3::<f64>::new(0.,-v,  0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => {
                    Translation3::<f64>::new(0.,-(calc_size.y * v*0.01),  0.)
                }
                
            }
        },
        LogicLength::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => calculation_origin_y_logiclength(calc_size,a).vector.scale(b.into_inner()).into() ,
                emg_core::CalcOp::Add { a, b } => calculation_origin_y_logiclength(calc_size,a)*calculation_origin_y_logiclength(calc_size,b),
            }
        },
    }
}

pub fn calculation_origin_x(p_calc_size: &Vector2<f64>, p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>,calc_size: &Vector2<f64>, origin_x: &GenericSize) -> Translation3<f64> {
    trace!("calculation_origin");

     match origin_x {
        GenericSize::Length(logic_l) => {
            calculation_origin_x_logiclength(calc_size, logic_l)
        },
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        },
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                     calculation_origin_x(p_calc_size, p_calc_origin,p_calc_align,calc_size,a).vector.scale(b.into_inner()) .into()
                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_origin_x(p_calc_size, p_calc_origin,p_calc_align,calc_size, a) * calculation_origin_x(p_calc_size, p_calc_origin,p_calc_align,calc_size,b)
                }
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val = get_parent_calculated(type_name,p_calc_size, p_calc_origin,p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => {
                    Translation3::<f64>::new( v,0.,0.)
                },
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                ParentCalculated::T3(t) => *t,
            }
        }
    }
}
pub fn calculation_origin_y(p_calc_size: &Vector2<f64>, p_calc_origin:&Translation3<f64>,p_calc_align:&Translation3<f64>,calc_size: &Vector2<f64>, origin_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_origin");

    
     match origin_y {
        GenericSize::Length(logic_l) => {
            calculation_origin_y_logiclength(calc_size, logic_l)

        },
        // GenericSize::Percentage(pc) => Translation3::<f64>::new(0., -(calc_size.y * pc.value()*0.01), 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        },
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                    calculation_origin_y(p_calc_size, p_calc_origin,p_calc_align,calc_size,a) .vector.scale(b.into_inner()).into()
                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_origin_y(p_calc_size, p_calc_origin,p_calc_align,calc_size, a) * calculation_origin_y(p_calc_size, p_calc_origin,p_calc_align,calc_size,b)
                }
            }
        },
        GenericSize::Parent(type_name) => {
            let parent_val = get_parent_calculated(type_name,p_calc_size, p_calc_origin,p_calc_align);
            match parent_val {
                ParentCalculated::Number(v) => {
                    Translation3::<f64>::new( 0.,v,0.)
                },
                ParentCalculated::V2(_) => unimplemented!("unsupported type"),
                ParentCalculated::T3(t) => *t,
            }
        }
    }

}


#[derive( Clone, Debug,From)]
enum ParentCalculated<'a>{
    Number(f64),
    V2(&'a Vector2<f64>),
    T3(&'a Translation3<f64>)
}
fn get_parent_calculated<'a>(type_name:&TypeName, p_calc_size: &Vector2<f64>,p_calc_origin:&'a Translation3<f64>,p_calc_align:&'a Translation3<f64>)->ParentCalculated<'a>{

  match type_name.as_str(){
                "CssWidth"=>{
                    p_calc_size.x.into()
                }
                "CssHeight"=>{
                    p_calc_size.y.into()

                }
                "OriginX"=>{
                    
                    p_calc_origin.vector.x.into()

                }
                "OriginY"=>{
                    
                    p_calc_origin.vector.y.into()

                }
                "Origin"=>{
                    
                    p_calc_origin.into()

                }
                "AlignX"=>{
                    
                    p_calc_align.vector.x.into()

                }
                "AlignY"=>{
                    
                    p_calc_align.vector.y.into()


                }
                "Align"=>{
                    
                    p_calc_align.into()


                }
                other=>{
                    panic!("current not implemented for GenericSize::Parent({})",other);
                }
            }
}