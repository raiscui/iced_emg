/*
 * @Author: Rais
 * @Date: 2022-08-10 15:50:41
 * @LastEditTime: 2023-02-02 11:55:24
 * @LastEditors: Rais
 * @Description:
 */
use better_any::TidAble;

pub trait MessageTid<'a>: TidAble<'a> {}
