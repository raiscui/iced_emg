
/*
* @Author: Rais
* @Date: 2021-03-29 17:30:58
 * @LastEditTime: 2021-08-20 12:46:39
 * @LastEditors: Rais
* @Description:
*/
use crate::{EdgeData, GenericSize, GenericSizeAnchor, Layout, LayoutCalculated, Mat4};

use emg::EdgeIndex;
use emg_state::{StateAnchor, StateMultiAnchor, StateVar, topo};
use nalgebra::{Translation3, Vector2};
use seed_styles as styles;
use styles::{px, s, CssHeightTrait, CssTransform, CssTransformTrait, CssWidthTrait, };
use tracing::{ trace,trace_span};

// ────────────────────────────────────────────────────────────────────────────────
    
// #[track_caller]
#[topo::nested]
pub fn layout_calculating<Ix>(
    id:StateVar< StateAnchor<EdgeIndex<Ix>>>,
    path_edgedata: &EdgeData,//parent
    layout: StateAnchor<Layout>,
) -> LayoutCalculated 
where 
    Ix: 'static + std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default + std::cmp::Ord+ std::fmt::Display 
    
    {
    let _span_ = trace_span!( "->[ layout_calculating ] ").entered();
    
            let EdgeData{
                calculated: p_calculated,
                
                ..
            }=path_edgedata;
            // ─────────────────────────────────────────────────────────────────

            let p_calc_size_sa = &p_calculated.size;
            // ─────────────────────────────────────────────────────────────────
            let w = layout.then(|l:&Layout|l.w.watch().into());
            let h = layout.then(|l:&Layout|l.h.watch().into());
            let origin_x = layout.then(|l:&Layout|l.origin_x.watch().into());
            let origin_y = layout.then(|l:&Layout|l.origin_y.watch().into());
            let align_x = layout.then(|l:&Layout|l.align_x.watch().into());
            let align_y = layout.then(|l:&Layout|l.align_y.watch().into());

            let calculated_size = (p_calc_size_sa, &w,&h).then(
                 move|p_calc_size: &Vector2<f64>, sa_w: &GenericSizeAnchor,sa_h:&GenericSizeAnchor| {
                    // let sa_w = sa_w1.clone().into_inner();        
                    // let sa_h = sa_h1.clone().into_inner();    
                    let p_calc_size = *p_calc_size;   
                    
                    // TODO  如果根 parent 无关 不是百分比  那么 不监听 parent
                    let _enter = trace_span!( 
                        "-> [ calculated_size ] recalculation..(&p_calculated.size, &layout.size.watch()).map ",
                        ).entered();
                     (&**sa_w,&**sa_h).map(move |w:&GenericSize,h:&GenericSize|->Vector2<f64>{
                        //TODO check editor display error 
                        let new_size = Vector2::<f64>::from_vec(vec![calculation_w(&p_calc_size, w), calculation_h(&p_calc_size, h)]);
                        // let new_size = Vector2::<f64>::from_vec(vec![calculation_w(p_calc_size, w), calculation_h(p_calc_size, h)]);
                        // Vector2::<f64>::new(w.get_length_value(), h.get_length_value())
                        trace!("new size: {}",&new_size);
                        new_size

                    }).into()

                        
                    

                    
                },
            );

            let calculated_origin = (&calculated_size, &origin_x,&origin_y).then(
                move |calc_size: &Vector2<f64>, origin_x: &GenericSizeAnchor,origin_y: &GenericSizeAnchor| {

      
                    let calc_size = *calc_size;
                    let _enter = trace_span!( 
                        "-> [ calculated_origin ] recalculation..(&calculated_size, &layout.origin.watch()).map ",
                        ).entered();

                        (&**origin_x, &**origin_y).map(move|ox:&GenericSize,oy:&GenericSize|{
                            calculation_origin(&calc_size, ox,oy)

                        }).into()

                },
            );

            let calculated_align:StateAnchor<Translation3<f64>> = (p_calc_size_sa, &align_x, &align_y).then(
                move |p_calc_size: &Vector2<f64>, align_x: &GenericSizeAnchor, align_y: &GenericSizeAnchor| {
                    let p_calc_size= *p_calc_size;
                    let _enter = trace_span!( 
                        "-> [ calculated_align ] recalculation..(&p_calculated.size, &layout.align.watch()).map ",
                        ).entered();
                    (&**align_x ,&**align_y).map(move|ax:&GenericSize,ay:&GenericSize|{
                        calculation_align(&p_calc_size, ax,ay)

                    }).into()
                },
            );

            let coordinates_trans =
                (&calculated_origin, &calculated_align).map(move |origin:&Translation3<f64>, align:&Translation3<f64>| {
                    
                    let _span =trace_span!( 
                        "-> [ coordinates_trans ] recalculation..(&calculated_origin, &calculated_align).map ",
                        );
                        
                    let _g = _span.enter();

                    let ff =  align * origin;
                    drop(_g);
                    trace!("coordinates_trans : {:?}",   &ff);

                    ff
                });

            let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());

            // @styles calculation ─────────────────────────────────────────────────────────────────
            // ────────────────────────────────────────────────────────────────────────────────
                

            let loc_styles = (&calculated_size, &matrix).map( move |calc_size: &Vector2<f64>, mat4: &Mat4| {
                            trace!( "------------size: {:?}  , matrix: {}", &calc_size, CssTransform::from(*mat4) );

                        { let _ender = trace_span!( 
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

fn calculation_w(p_calc_size: &Vector2<f64>, w: &GenericSize) -> f64 {
    trace!("calculation_w");
    match w {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value();
            match ex_l.unit {
                styles::Unit::Px |styles::Unit::Empty => v,
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => unreachable!(),
                
            }
        }
        GenericSize::Percentage(pc) => p_calc_size.x * pc.value()*0.01,
        
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
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
    }


}
fn calculation_h(p_calc_size: &Vector2<f64>, h: &GenericSize) -> f64 {
    trace!("calculation_h");
    
    match h {
        GenericSize::Percentage(pc) => p_calc_size.y * pc.value()*0.01,

        GenericSize::Length(ex_l) => {
            match ex_l.unit {
                styles::Unit::Px |  styles::Unit::Empty => ex_l.value(),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => unreachable!(),
              
            }
        }
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
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
    }

}


fn calculation_align(p_calc_size: &Vector2<f64>, align_x: &GenericSize,align_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_align");

    calculation_align_x(p_calc_size, align_x) * calculation_align_y(p_calc_size, align_y)
}

fn calculation_align_x(p_calc_size: &Vector2<f64>, align_x: &GenericSize) -> Translation3<f64> {
    trace!("calculation_align");

     match align_x {
        GenericSize::Length(ex_l) => {
            match ex_l.unit {
                styles::Unit::Px|  styles::Unit::Empty  => Translation3::<f64>::new(ex_l.value(), 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => unreachable!(),
              
            }
        }
        GenericSize::Percentage(pc) => Translation3::<f64>::new(p_calc_size.x * pc.value()*0.01, 0., 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {

                    // let scaling = nalgebra::Similarity3::from_scaling(b.into_inner());
                    let mut x = calculation_align_x(p_calc_size,a) ;
                    x.vector.scale_mut(b.into_inner());
                    x
                    
                },
                emg_core::CalcOp::Add { a, b } => {
                      calculation_align_x(p_calc_size, a)* calculation_align_x(p_calc_size,b)
                }
            }
        },
    }
}
fn calculation_align_y(p_calc_size: &Vector2<f64>, align_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_align");

    
    match align_y {
        GenericSize::Length(ex_l) => {
            let v = ex_l.value.into_inner();
            match ex_l.unit {
                styles::Unit::Px |styles::Unit::Empty => Translation3::<f64>::new(0., v, 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => unreachable!(),
            }
        }
        GenericSize::Percentage(pc) => Translation3::<f64>::new(0., p_calc_size.y * pc.value()*0.01, 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                  
                    let mut v = calculation_align_y(p_calc_size,a) ;
                    v.vector.scale_mut(b.into_inner());
                    v

                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_align_y(p_calc_size, a) * calculation_align_y(p_calc_size,b)
                }
            }
        },
    }
}

pub fn calculation_origin(calc_size: &Vector2<f64>, origin_x: &GenericSize,origin_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_origin");
    calculation_origin_x(calc_size, origin_x) * calculation_origin_y(calc_size, origin_y)
}
pub fn calculation_origin_x(calc_size: &Vector2<f64>, origin_x: &GenericSize) -> Translation3<f64> {
    trace!("calculation_origin");

     match origin_x {
        GenericSize::Length(ex_l) => {
            match ex_l.unit {
                styles::Unit::Px | styles::Unit::Empty  => Translation3::<f64>::new(-ex_l.value(), 0., 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => unreachable!(),
            }
        }
        GenericSize::Percentage(pc) => Translation3::<f64>::new(-(calc_size.x * pc.value()*0.01), 0., 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                 
                    let mut v = calculation_origin_x(calc_size,a) ;
                    v.vector.scale_mut(b.into_inner());
                    v
                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_origin_x(calc_size, a) * calculation_origin_x(calc_size,b)
                }
            }
        },
    }
}
pub fn calculation_origin_y(calc_size: &Vector2<f64>, origin_y: &GenericSize) -> Translation3<f64> {
    trace!("calculation_origin");

    
     match origin_y {
        GenericSize::Length(ex_l) => {
            match ex_l.unit {
                
                styles::Unit::Px|styles::Unit::Empty => Translation3::<f64>::new(0., -ex_l.value(), 0.),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => unreachable!(),
            }
        }
        GenericSize::Percentage(pc) => Translation3::<f64>::new(0., -(calc_size.y * pc.value()*0.01), 0.),
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => {
            match calc_op.as_ref() {
                emg_core::CalcOp::Mul { a, b } => {
                    let mut v = calculation_origin_y(calc_size,a) ;
                    v.vector.scale_mut(b.into_inner());
                    v
                },
                emg_core::CalcOp::Add { a, b } => {
                    calculation_origin_y(calc_size, a) * calculation_origin_y(calc_size,b)
                }
            }
        },
    }
}
