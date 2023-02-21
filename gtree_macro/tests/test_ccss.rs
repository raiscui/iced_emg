/*
 * @Author: Rais
 * @Date: 2022-07-12 11:27:18
 * @LastEditTime: 2023-02-21 21:44:32
 * @LastEditors: Rais
 * @Description:
 */
#[cfg(test)]
mod code_test {

    use emg_common::{IdStr, VecDisp, VectorDisp};
    use emg_layout::ccsa_macro_prelude;
    use gtree_macro::cassowary::*;
    use quote::ToTokens;
    use std::path::Path;
    use syn::parse::Parse;
    use tracing::{debug, info};

    use tracing_subscriber::{prelude::*, registry::Registry};

    #[track_caller]
    fn token_2_code_test(name: &str, input: &str) -> String {
        // ────────────────────────────────────────────────────────────────────────────────

        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        let mut macro_disp_string = String::new();

        // ─────────────────────────────────────────────────────────────────

        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {

                    info!("=========== parse \n {:?}\n",&input);

                    match syn::parse_str::<VFLStatement>(input) {
                        Ok( ok) => {
                            info!("=============VFLStatement ok \n{:#?}\n", &ok);


                            let macro_disp = VecDisp(ok.ccsss.clone());
                            info!("=================== display--macro-build \n {}\n", &macro_disp);

                            let rust_code = format!("{}", ok.to_token_stream());
                            info!("=================== rust_code \n {}\n", &rust_code);
        // ─────────────────────────────────────────────────────────────────────────────


                            insta::assert_display_snapshot!(name.to_string()+"_build_display", &macro_disp);
                            insta::assert_display_snapshot!(name.to_string()+"_code", rust_code);
                            macro_disp_string = format!("{}",macro_disp);


                        }
                        Err(error) => panic!("...{:?}", error),
                    }
                });
        macro_disp_string
    }

    #[track_caller]
    fn cass_code_test<T: ToTokens + Parse + std::fmt::Debug>(name: &str, input: &str) -> String {
        let subscriber = Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        // .with(subscriber1);
        tracing::subscriber::set_global_default(subscriber).ok();

        insta::with_settings!({snapshot_path => Path::new("./vfl_code_snap")}, {
            debug!("=========== parse \n {:?}\n", &input);

            match syn::parse_str::<T>(input) {
                Ok(ok) => {
                    println!("============= parsed\n{:#?}\n", &ok);

                    let rust_code = format!("{}", ok.to_token_stream());
                    println!("token_stream result:\n{}", rust_code);
                    insta::assert_display_snapshot!(name.to_string() + "_ccss_code", rust_code);
                    rust_code
                }
                Err(error) => panic!("...{:?}", error),
            }
        })
    }

    #[test]
    fn name_chars() {
        println!();
        let input = r#"#button"#;

        let macro_2_code_string = cass_code_test::<NameCharsOrNumber>("name_chars", input);
        let code = emg_layout::ccsa::NameCharsOrNumber::Id(IdStr::new("button"));
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
            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                emg_common::IdStr::new("button"),
            )),
            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(emg_common::IdStr::new(
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
        let input = r#"#button[width] + 10"#;
        //TODO CCSS parse

        let code = emg_layout::ccsa::ScopeViewVariable::new_id_var("button", "width");
        let res = code + emg_layout::ccsa::ScopeViewVariable::new_number(10.0);
        println!("{}", res);
        assert_eq!(input, format!("{}", res));
    }
    #[test]
    fn ccss_svv_op_svv_expr2() {
        println!();
        let input = r#"#button[width] + #button2[height]"#;
        //TODO CCSS parse

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

        let name = &"base1";
        let parsed_macro_disp = token_2_code_test(name, input);

        let (res, selector) = (
            emg_common::im::vector![emg_layout::ccsa::CCSS::new(
                emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                    emg_layout::ccsa::ScopeViewVariable::new(
                        ::std::option::Option::None,
                        ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                            emg_common::IdStr::new("b1")
                        )),
                        ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                            emg_common::IdStr::new("right")
                        ))
                    ),
                    vec![]
                ),
                vec![emg_layout::ccsa::CCSSEqExpression::new(
                    emg_layout::ccsa::PredEq::Eq,
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b2")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("left")
                            ))
                        ),
                        vec![]
                    )
                )],
                ::std::option::Option::None
            )],
            emg_common::im::vector![
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b1")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b2")
                    )),
                    ::std::option::Option::None
                )
            ],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
            insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });
        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed_macro_disp, format!("{}", res_disp));
    }

    #[test]
    fn base2() {
        let input = r#"
                @v (#b1)(#b2)
            "#;

        let name = &"base2";
        let parsed = token_2_code_test(name, input);

        let (res, selector) = (
            emg_common::im::vector![emg_layout::ccsa::CCSS::new(
                emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                    emg_layout::ccsa::ScopeViewVariable::new(
                        ::std::option::Option::None,
                        ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                            emg_common::IdStr::new("b1")
                        )),
                        ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                            emg_common::IdStr::new("bottom")
                        ))
                    ),
                    vec![]
                ),
                vec![emg_layout::ccsa::CCSSEqExpression::new(
                    emg_layout::ccsa::PredEq::Eq,
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b2")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("top")
                            ))
                        ),
                        vec![]
                    )
                )],
                ::std::option::Option::None
            )],
            emg_common::im::vector![
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b1")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b2")
                    )),
                    ::std::option::Option::None
                )
            ],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
        insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });

        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }
    #[test]
    fn base3() {
        let input = r#"
        @v (#b1)-(#b2)  -  (#b3)- (#b4) -(#b5) !weak
            "#;

        let name = &"base3";
        let parsed = token_2_code_test(name, input);

        let (res, selector) = (
            emg_common::im::vector![
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b1")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("bottom")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::None,
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("vgap")
                                ))
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b2")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("top")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::Some(emg_layout::ccsa::StrengthAndWeight::Weak(
                        ::std::option::Option::None
                    ))
                ),
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b2")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("bottom")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::None,
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("vgap")
                                ))
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b3")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("top")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::Some(emg_layout::ccsa::StrengthAndWeight::Weak(
                        ::std::option::Option::None
                    ))
                ),
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b3")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("bottom")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::None,
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("vgap")
                                ))
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b4")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("top")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::Some(emg_layout::ccsa::StrengthAndWeight::Weak(
                        ::std::option::Option::None
                    ))
                ),
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b4")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("bottom")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::None,
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("vgap")
                                ))
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b5")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("top")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::Some(emg_layout::ccsa::StrengthAndWeight::Weak(
                        ::std::option::Option::None
                    ))
                )
            ],
            emg_common::im::vector![
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b1")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b2")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b3")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b4")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b5")
                    )),
                    ::std::option::Option::None
                )
            ],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
        insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });

        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }

    #[test]
    fn base4() {
        let input = r#"
        @v |(#sub)| in("parent")
            "#;

        let name = &"base4";
        let parsed = token_2_code_test(name, input);

        let (res, selector) = (
            emg_common::im::vector![
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(
                                emg_layout::ccsa::NameCharsOrNumber::Virtual(
                                    emg_common::IdStr::new("parent")
                                )
                            ),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("top")
                            ))
                        ),
                        vec![]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("sub")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("top")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("sub")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("bottom")
                            ))
                        ),
                        vec![]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Virtual(
                                        emg_common::IdStr::new("parent")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("bottom")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                )
            ],
            emg_common::im::vector![emg_layout::ccsa::ScopeViewVariable::new(
                ::std::option::Option::None,
                ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                    emg_common::IdStr::new("sub")
                )),
                ::std::option::Option::None
            )],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
        insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });

        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }

    #[test]
    fn base5() {
        let input = r#"
        @h (#b1)-100-(#b2)-8-(#b3)
            "#;

        let name = &"base5";
        let parsed = token_2_code_test(name, input);

        let (res, selector) = (
            emg_common::im::vector![
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b1")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("right")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Number(
                                        emg_common::NotNan::new(100 as f64).unwrap()
                                    )
                                ),
                                ::std::option::Option::None
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b2")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("left")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b2")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("right")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Number(
                                        emg_common::NotNan::new(8 as f64).unwrap()
                                    )
                                ),
                                ::std::option::Option::None
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b3")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("left")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                )
            ],
            emg_common::im::vector![
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b1")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b2")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b3")
                    )),
                    ::std::option::Option::None
                )
            ],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
        insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });

        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }

    #[test]
    fn base6() {
        let input = r#"
        @h (#b1)-[my_gap]-(#b2)-[my_other_gap]-(#b3)
            "#;

        let name = &"base6";
        let parsed = token_2_code_test(name, input);

        let (res, selector) = (
            emg_common::im::vector![
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b1")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("right")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::None,
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("my_gap")
                                ))
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b2")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("left")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::CCSS::new(
                    emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                        emg_layout::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                                emg_common::IdStr::new("b2")
                            )),
                            ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                emg_common::IdStr::new("right")
                            ))
                        ),
                        vec![emg_layout::ccsa::CCSSOpSvv::new(
                            emg_layout::ccsa::PredOp::Add,
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::None,
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("my_other_gap")
                                ))
                            )
                        )]
                    ),
                    vec![emg_layout::ccsa::CCSSEqExpression::new(
                        emg_layout::ccsa::PredEq::Eq,
                        emg_layout::ccsa::CCSSSvvOpSvvExpr::new(
                            emg_layout::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    emg_layout::ccsa::NameCharsOrNumber::Id(
                                        emg_common::IdStr::new("b3")
                                    )
                                ),
                                ::std::option::Option::Some(emg_layout::ccsa::PredVariable(
                                    emg_common::IdStr::new("left")
                                ))
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                )
            ],
            emg_common::im::vector![
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b1")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b2")
                    )),
                    ::std::option::Option::None
                ),
                emg_layout::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(emg_layout::ccsa::NameCharsOrNumber::Id(
                        emg_common::IdStr::new("b3")
                    )),
                    ::std::option::Option::None
                )
            ],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
        insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });

        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }
    #[test]
    fn parent() {
        let input = r#"
        @v |(#sub)|
            "#;

        let name = &"parent";
        let parsed = token_2_code_test(name, input);

        let (res, selector) = (
            ccsa_macro_prelude::common::im::vector![
                ccsa_macro_prelude::ccsa::CCSS::new(
                    ccsa_macro_prelude::ccsa::CCSSSvvOpSvvExpr::new(
                        ccsa_macro_prelude::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::Some(ccsa_macro_prelude::ccsa::Scope::Local),
                            ::std::option::Option::None,
                            ::std::option::Option::Some(ccsa_macro_prelude::ccsa::PredVariable(
                                ccsa_macro_prelude::common::IdStr::new("top")
                            ))
                        ),
                        vec![]
                    ),
                    vec![ccsa_macro_prelude::ccsa::CCSSEqExpression::new(
                        ccsa_macro_prelude::ccsa::PredEq::Eq,
                        ccsa_macro_prelude::ccsa::CCSSSvvOpSvvExpr::new(
                            ccsa_macro_prelude::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    ccsa_macro_prelude::ccsa::NameCharsOrNumber::Id(
                                        ccsa_macro_prelude::common::IdStr::new("sub")
                                    )
                                ),
                                ::std::option::Option::Some(
                                    ccsa_macro_prelude::ccsa::PredVariable(
                                        ccsa_macro_prelude::common::IdStr::new("top")
                                    )
                                )
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                ),
                ccsa_macro_prelude::ccsa::CCSS::new(
                    ccsa_macro_prelude::ccsa::CCSSSvvOpSvvExpr::new(
                        ccsa_macro_prelude::ccsa::ScopeViewVariable::new(
                            ::std::option::Option::None,
                            ::std::option::Option::Some(
                                ccsa_macro_prelude::ccsa::NameCharsOrNumber::Id(
                                    ccsa_macro_prelude::common::IdStr::new("sub")
                                )
                            ),
                            ::std::option::Option::Some(ccsa_macro_prelude::ccsa::PredVariable(
                                ccsa_macro_prelude::common::IdStr::new("bottom")
                            ))
                        ),
                        vec![]
                    ),
                    vec![ccsa_macro_prelude::ccsa::CCSSEqExpression::new(
                        ccsa_macro_prelude::ccsa::PredEq::Eq,
                        ccsa_macro_prelude::ccsa::CCSSSvvOpSvvExpr::new(
                            ccsa_macro_prelude::ccsa::ScopeViewVariable::new(
                                ::std::option::Option::Some(ccsa_macro_prelude::ccsa::Scope::Local),
                                ::std::option::Option::None,
                                ::std::option::Option::Some(
                                    ccsa_macro_prelude::ccsa::PredVariable(
                                        ccsa_macro_prelude::common::IdStr::new("bottom")
                                    )
                                )
                            ),
                            vec![]
                        )
                    )],
                    ::std::option::Option::None
                )
            ],
            ccsa_macro_prelude::common::im::vector![
                ccsa_macro_prelude::ccsa::ScopeViewVariable::new(
                    ::std::option::Option::None,
                    ::std::option::Option::Some(ccsa_macro_prelude::ccsa::NameCharsOrNumber::Id(
                        ccsa_macro_prelude::common::IdStr::new("sub")
                    )),
                    ::std::option::Option::None
                )
            ],
        );

        info!("selector: {}", VectorDisp(selector.clone()));
        insta::with_settings!({snapshot_path => Path::new("./vfl_to_code_snap")}, {
        insta::assert_display_snapshot!(name.to_string()+"_selector_display", &VectorDisp(selector));
        });

        let res_disp = VectorDisp(res);
        info!("res===\n{}", &res_disp);
        assert_eq!(parsed, format!("{}", res_disp));
    }
}
