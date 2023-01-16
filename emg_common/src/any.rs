/*
 * @Author: Rais
 * @Date: 2022-08-10 15:50:41
 * @LastEditTime: 2022-08-10 16:48:54
 * @LastEditors: Rais
 * @Description:
 */
use better_any::TidAble;

pub trait MessageTid<'a>: TidAble<'a> {}
