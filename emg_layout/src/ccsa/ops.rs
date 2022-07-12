/*
 * @Author: Rais
 * @Date: 2022-07-12 15:41:55
 * @LastEditTime: 2022-07-12 16:05:41
 * @LastEditors: Rais
 * @Description:
 */

use super::CCSSOpSvv;

use super::CCSSSvvOpSvvExpr;

use super::ScopeViewVariable;

impl std::ops::Add<Self> for ScopeViewVariable {
    type Output = CCSSSvvOpSvvExpr;
    fn add(self, other: Self) -> CCSSSvvOpSvvExpr {
        CCSSSvvOpSvvExpr {
            svv: self,
            op_exprs: vec![CCSSOpSvv::new_add(other)],
        }
    }
}

impl std::ops::Sub<Self> for ScopeViewVariable {
    type Output = CCSSSvvOpSvvExpr;
    fn sub(self, other: Self) -> CCSSSvvOpSvvExpr {
        CCSSSvvOpSvvExpr {
            svv: self,
            op_exprs: vec![CCSSOpSvv::new_sub(other)],
        }
    }
}

impl std::ops::Mul<Self> for ScopeViewVariable {
    type Output = CCSSSvvOpSvvExpr;
    fn mul(self, other: Self) -> CCSSSvvOpSvvExpr {
        CCSSSvvOpSvvExpr {
            svv: self,
            op_exprs: vec![CCSSOpSvv::new_mul(other)],
        }
    }
}

impl std::ops::Add<ScopeViewVariable> for CCSSSvvOpSvvExpr {
    type Output = CCSSSvvOpSvvExpr;
    fn add(self, other: ScopeViewVariable) -> CCSSSvvOpSvvExpr {
        let mut op_exprs = self.op_exprs;
        op_exprs.push(CCSSOpSvv::new_add(other));
        CCSSSvvOpSvvExpr {
            svv: self.svv,
            op_exprs,
        }
    }
}

impl std::ops::Sub<ScopeViewVariable> for CCSSSvvOpSvvExpr {
    type Output = CCSSSvvOpSvvExpr;
    fn sub(self, other: ScopeViewVariable) -> CCSSSvvOpSvvExpr {
        let mut op_exprs = self.op_exprs;
        op_exprs.push(CCSSOpSvv::new_sub(other));
        CCSSSvvOpSvvExpr {
            svv: self.svv,
            op_exprs,
        }
    }
}

impl std::ops::Mul<ScopeViewVariable> for CCSSSvvOpSvvExpr {
    type Output = CCSSSvvOpSvvExpr;
    fn mul(self, other: ScopeViewVariable) -> CCSSSvvOpSvvExpr {
        let mut op_exprs = self.op_exprs;
        op_exprs.push(CCSSOpSvv::new_mul(other));
        CCSSSvvOpSvvExpr {
            svv: self.svv,
            op_exprs,
        }
    }
}
