/*
 * @Author: Rais
 * @Date: 2022-07-12 11:27:18
 * @LastEditTime: 2022-07-12 18:34:46
 * @LastEditors: Rais
 * @Description:
 */
#[cfg(test)]
mod code_test {

    use emg_core::IdStr;
    use gtree_macro::cassowary::*;
    use quote::ToTokens;
    use std::path::Path;
    use syn::parse::Parse;
    use tracing::{debug, info};

    use tracing_subscriber::{prelude::*, registry::Registry};

    fn token_2_code_test(name: &str, input: &str) -> String {
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        let mut s = String::new();

        // ─────────────────────────────────────────────────────────────────

        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {

            info!("=========== parse \n {:?}\n",&input);

            match syn::parse_str::<VFLStatement>(input) {
                Ok( ok) => {
                    info!("=============VFLStatement ok \n{:#?}\n", &ok);
                    let x = format!("{}", ok.to_token_stream());

                    info!("=================== build-code \n {}\n", &x);

                    let disp = CCSSSDisp(ok.ccsss);
                    info!("=================== build---display \n {}\n", &disp);
                    insta::assert_display_snapshot!(name.to_string()+"_build_display", &disp);


                    insta::assert_display_snapshot!(name.to_string()+"_code", x);
                    s = format!("{}",disp);


                }
                Err(error) => panic!("...{:?}", error),
            }
        });
        s
    }

    fn cass_code_test<T: ToTokens + Parse + std::fmt::Debug>(name: &str, input: &str) {
        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        insta::with_settings!({snapshot_path => Path::new("./vfl_code_snap")}, {
            debug!("=========== parse \n {:?}\n", &input);

            match syn::parse_str::<T>(input) {
                Ok(ok) => {
                    println!("============= parsed\n{:#?}\n", &ok);

                    let x = format!("{}", ok.to_token_stream());
                    println!("token_stream result:\n{}", x);
                    insta::assert_display_snapshot!(name.to_string() + "_ccss_code", x);
                }
                Err(error) => panic!("...{:?}", error),
            }
        });
    }

    #[test]
    fn name_chars() {
        println!();
        let input = r#"#button"#;

        cass_code_test::<NameChars>("name_chars", input);
        let code = emg_layout::ccsa::NameChars::Id(IdStr::new("button"));
        println!("{}", code);
        assert_eq!(input, format!("{}", code));
    }
    #[test]
    fn scope_view_variable() {
        println!();
        let input = r#"#button[width]"#;

        cass_code_test::<ScopeViewVariable>("ScopeViewVariable", input);

        let code = emg_layout::ccsa::ScopeViewVariable::new_id_var("button", "width");
        let code_gen = emg_layout::ccsa::ScopeViewVariable::new(
            ::std::option::Option::None,
            ::std::option::Option::Some(emg_layout::ccsa::NameChars::Id(emg_core::IdStr::new(
                "button",
            ))),
            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(emg_core::IdStr::new(
                "width",
            ))),
        );
        println!("{}", code);
        assert_eq!(input, format!("{}", code));
        assert_eq!(code, code_gen);
    }
    #[test]
    fn ccss_svv_op_svv_expr() {
        println!();
        let input = r#"#button[width]+ 10"#;

        let code = emg_layout::ccsa::ScopeViewVariable::new_id_var("button", "width");
        let res = code + emg_layout::ccsa::ScopeViewVariable::new_number(10.0);
        println!("{}", res);
        assert_eq!(input, format!("{}", res));
    }
    #[test]
    fn ccss_svv_op_svv_expr2() {
        println!();
        let input = r#"#button[width]+ #button2[height]"#;

        let code = emg_layout::ccsa::ScopeViewVariable::new_id_var("button", "width");
        let res = code + emg_layout::ccsa::ScopeViewVariable::new_id_var("button2", "height");
        println!("{}", res);
        assert_eq!(input, format!("{}", res));
    }
    #[test]
    fn base1() {
        let input = r#" 
                @h (#b1)(#b2)
            "#;

        let parsed = token_2_code_test("base1", input);

        let res = emg_core::vector![emg_layout::ccsa::CCSS::new(
            emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameChars::Id(
                        emg_core::IdStr::new("b1"),
                    )),
                    ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                        emg_core::IdStr::new("right"),
                    )),
                ),
                vec![],
            ),
            vec![emg_layout::ccsa::CCSSEqExpression::new(
                emg_layout::ccsa::PredEq::Eq,
                emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                    emg_layout::ccsa::ScopeViewVariable::new(
                        ::std::option::Option::None,
                        ::std::option::Option::Some(emg_layout::ccsa::NameChars::Id(
                            emg_core::IdStr::new("b2"),
                        )),
                        ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                            emg_core::IdStr::new("left"),
                        )),
                    ),
                    vec![],
                ),
            )],
            ::std::option::Option::None,
        )];
        let res_disp = emg_layout::ccsa::CCSSSDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }

    #[test]
    fn base2() {
        let input = r#" 
                @v (#b1)(#b2)
            "#;

        let parsed = token_2_code_test("base2", input);

        let res = emg_core::vector![emg_layout::ccsa::CCSS::new(
            emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameChars::Id(
                        emg_core::IdStr::new("b1"),
                    )),
                    ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                        emg_core::IdStr::new("bottom"),
                    )),
                ),
                vec![],
            ),
            vec![emg_layout::ccsa::CCSSEqExpression::new(
                emg_layout::ccsa::PredEq::Eq,
                emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                    emg_layout::ccsa::ScopeViewVariable::new(
                        ::std::option::Option::None,
                        ::std::option::Option::Some(emg_layout::ccsa::NameChars::Id(
                            emg_core::IdStr::new("b2"),
                        )),
                        ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                            emg_core::IdStr::new("top"),
                        )),
                    ),
                    vec![],
                ),
            )],
            ::std::option::Option::None,
        )];
        let res_disp = emg_layout::ccsa::CCSSSDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }
}
