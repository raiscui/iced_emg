/*
 * @Author: Rais
 * @Date: 2022-08-29 23:19:00
 * @LastEditTime: 2023-03-17 10:10:29
 * @LastEditors: Rais
 * @Description:
 */

use crate::EmgEdgeItem;
use emg_common::TypeCheck;
use emg_native::WidgetState;
use emg_shaping::{EqShapingWithDebug, Shaping, ShapingWhoNoWarper};
use emg_state::StateTypeCheck;
use emg_state::{CloneStateVar, StateAnchor, StateVar};
#[allow(clippy::wildcard_imports)]
use seed_styles::*;
use std::rc::Rc;

// impl Shaping<EmgEdgeItem> for CssBackgroundAttachment
// where
//     EmgEdgeItem: ShapingWhoNoWarper,
// {
//     fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
//         let type_name = Self::TYPE_NAME;
//         who.styles.update(|s| {
//             s.insert(type_name, StateAnchor::constant(Rc::new(self.clone())));
//         });
//         true
//     }
// }

// impl Shaping<EmgEdgeItem> for StateVar<CssBackgroundAttachment>
// where
//     EmgEdgeItem: ShapingWhoNoWarper,
// {
//     fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
//         let type_name = Self::INSIDE_TYPE_NAME;
//         who.styles.update(|s| {
//             let value = self
//                 .watch()
//                 .map(|x| Rc::new(x.clone()) as Rc<dyn EqShapingWithDebug<WidgetState>>);
//             s.insert(type_name, value);
//         });
//         true
//     }
// }

// impl Shaping<EmgEdgeItem> for StateAnchor<CssBackgroundAttachment>
// where
//     EmgEdgeItem: ShapingWhoNoWarper,
// {
//     fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
//         let type_name = Self::INSIDE_TYPE_NAME;
//         who.styles.update(|s| {
//             let value = self.map(|x| Rc::new(x.clone()) as Rc<dyn EqShapingWithDebug<WidgetState>>);
//             s.insert(type_name, value);
//         });
//         true
//     }
// }

macro_rules! impl_css_native_shaping {
    ($css:ident) => {
        impl Shaping<EmgEdgeItem> for $css
        where
            EmgEdgeItem: ShapingWhoNoWarper,
        {
            fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
                let type_name = Self::TYPE_NAME;
                who.styles.update(|s| {
                    s.insert(type_name, StateAnchor::constant(Rc::new(self.clone())));
                });
                true
            }
        }

        impl Shaping<EmgEdgeItem> for StateVar<$css>
        where
            EmgEdgeItem: ShapingWhoNoWarper,
        {
            fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
                let type_name = Self::INSIDE_TYPE_NAME;
                who.styles.update(|s| {
                    let value = self
                        .watch()
                        .map(|x| Rc::new(x.clone()) as Rc<dyn EqShapingWithDebug<WidgetState>>);
                    s.insert(type_name, value);
                });
                true
            }
        }

        impl Shaping<EmgEdgeItem> for StateAnchor<$css>
        where
            EmgEdgeItem: ShapingWhoNoWarper,
        {
            fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
                let type_name = Self::INSIDE_TYPE_NAME;
                who.styles.update(|s| {
                    let value =
                        self.map(|x| Rc::new(x.clone()) as Rc<dyn EqShapingWithDebug<WidgetState>>);
                    s.insert(type_name, value);
                });
                true
            }
        }
    };
}
macro_rules! impl_css_native_shaping_list {
    ($($css:ident),*) => {
        $(

            impl_css_native_shaping!($css);

        )*
    };
}

impl_css_native_shaping!(CssBackgroundAttachment);
impl_css_native_shaping!(CssColumnSpan);
impl_css_native_shaping!(CssColumnsFill);
impl_css_native_shaping!(CssColumnRule);
impl_css_native_shaping!(CssColumnCount);
impl_css_native_shaping!(CssColumnWidth);
impl_css_native_shaping!(CssRaw);
impl_css_native_shaping!(CssBackgroundImage);
impl_css_native_shaping!(CssBackgroundPosition);
impl_css_native_shaping!(CssBackgroundRepeat);
impl_css_native_shaping!(CssBorderCollapse);
// impl_css_native_refresh!(CssBorderSpacing);
impl_css_native_shaping!(CssCaptionSide);
impl_css_native_shaping!(CssClear);
impl_css_native_shaping!(CssClip);
impl_css_native_shaping!(CssContent);
impl_css_native_shaping!(CssCounterIncrement);
impl_css_native_shaping!(CssCursor);
impl_css_native_shaping!(CssDirection);
impl_css_native_shaping!(CssEmptyCells);
impl_css_native_shaping!(CssFloat);
impl_css_native_shaping!(CssFontStyle);
impl_css_native_shaping!(CssFontVariant);
impl_css_native_shaping!(CssFontWeight);
impl_css_native_shaping!(CssListStyleImage);
impl_css_native_shaping!(CssListStylePosition);
impl_css_native_shaping!(CssListStyleType);
impl_css_native_shaping!(CssListStyle);
impl_css_native_shaping!(CssOrphans);
impl_css_native_shaping!(CssOverflow);
impl_css_native_shaping!(CssOverflowX);
impl_css_native_shaping!(CssOverflowY);
impl_css_native_shaping!(CssPageBreak);
impl_css_native_shaping!(CssPosition);
impl_css_native_shaping!(CssQuotes);
impl_css_native_shaping!(CssTableLayout);
impl_css_native_shaping!(CssTextAlign);
impl_css_native_shaping!(CssTextDecoration);
impl_css_native_shaping!(CssTextDecorationColor);
impl_css_native_shaping!(CssTextIndent);
impl_css_native_shaping!(CssTextTransform);
impl_css_native_shaping!(CssUnicodeBidi);
impl_css_native_shaping!(CssVerticalAlign);
impl_css_native_shaping!(CssVisibility);
impl_css_native_shaping!(CssWhiteSpace);
impl_css_native_shaping!(CssWidows);
impl_css_native_shaping!(CssWordSpacing);
impl_css_native_shaping!(CssZIndex);
impl_css_native_shaping!(CssMargin);
impl_css_native_shaping!(CssMarginTop);
impl_css_native_shaping!(CssMarginBottom);
impl_css_native_shaping!(CssMarginLeft);
impl_css_native_shaping!(CssMarginRight);
impl_css_native_shaping!(CssSpace);
impl_css_native_shaping!(CssTop);
impl_css_native_shaping!(CssBottom);
impl_css_native_shaping!(CssLeft);
impl_css_native_shaping!(CssRight);
impl_css_native_shaping!(CssGridGap);
impl_css_native_shaping!(CssGridColumnGap);
impl_css_native_shaping!(CssGridRowGap);
impl_css_native_shaping!(CssGap);
impl_css_native_shaping!(CssColumnGap);
impl_css_native_shaping!(CssRowGap);
impl_css_native_shaping!(CssPadding);
impl_css_native_shaping!(CssPaddingTop);
impl_css_native_shaping!(CssPaddingRight);
impl_css_native_shaping!(CssPaddingLeft);
impl_css_native_shaping!(CssPaddingBottom);
impl_css_native_shaping!(CssBorderStyle);
impl_css_native_shaping!(CssBorderLeftStyle);
impl_css_native_shaping!(CssBorderRightStyle);
impl_css_native_shaping!(CssBorderTopStyle);
impl_css_native_shaping!(CssBorderBottomStyle);
impl_css_native_shaping!(CssOutlineStyle);
impl_css_native_shaping!(CssOutlineLeftStyle);
impl_css_native_shaping!(CssOutlineRightStyle);
impl_css_native_shaping!(CssOutlineTopStyle);
impl_css_native_shaping!(CssOutlineBottomStyle);
impl_css_native_shaping!(CssBorderWidth);
impl_css_native_shaping!(CssBorderLeftWidth);
impl_css_native_shaping!(CssBorderRightWidth);
impl_css_native_shaping!(CssBorderTopWidth);
impl_css_native_shaping!(CssBorderBottomWidth);
impl_css_native_shaping!(CssOutlineWidth);
impl_css_native_shaping!(CssOutlineLeftWidth);
impl_css_native_shaping!(CssOutlineRightWidth);
impl_css_native_shaping!(CssOutlineTopWidth);
impl_css_native_shaping!(CssOutlineBottomWidth);
impl_css_native_shaping!(CssSize);
impl_css_native_shaping!(CssFlexBasis);
// impl_css_native_refresh!(CssWidth);
// impl_css_native_refresh!(CssHeight);
impl_css_native_shaping!(CssMinWidth);
impl_css_native_shaping!(CssMaxWidth);
impl_css_native_shaping!(CssMinHeight);
impl_css_native_shaping!(CssMaxHeight);
impl_css_native_shaping!(CssBorder);
impl_css_native_shaping!(CssBorderLeft);
impl_css_native_shaping!(CssBorderRight);
impl_css_native_shaping!(CssBorderTop);
impl_css_native_shaping!(CssBorderBottom);
impl_css_native_shaping!(CssOutline);
impl_css_native_shaping!(CssOutlineLeft);
impl_css_native_shaping!(CssOutlineRight);
impl_css_native_shaping!(CssOutlineTop);
impl_css_native_shaping!(CssOutlineBottom);
impl_css_native_shaping!(CssTransition);
impl_css_native_shaping!(CssLetterSpacing);
impl_css_native_shaping!(CssLineHeight);
impl_css_native_shaping!(CssBorderRadius);
impl_css_native_shaping!(CssBorderTopRightRadius);
impl_css_native_shaping!(CssBorderTopLeftRadius);
impl_css_native_shaping!(CssBorderBRRadius);
impl_css_native_shaping!(CssBorderBLRadius);
impl_css_native_shaping!(CssFont);
impl_css_native_shaping!(CssFontSize);
impl_css_native_shaping!(CssColor);
impl_css_native_shaping!(CssStroke);
impl_css_native_shaping!(CssBackgroundColor);
impl_css_native_shaping!(CssFill);
impl_css_native_shaping!(CssBorderColor);
impl_css_native_shaping!(CssBorderLeftColor);
impl_css_native_shaping!(CssBorderRightColor);
impl_css_native_shaping!(CssBorderTopColor);
impl_css_native_shaping!(CssBorderBottomColor);
impl_css_native_shaping!(CssOutlineColor);
impl_css_native_shaping!(CssOutlineLeftColor);
impl_css_native_shaping!(CssOutlineRightColor);
impl_css_native_shaping!(CssOutlineTopColor);
impl_css_native_shaping!(CssOutlineBottomColor);
impl_css_native_shaping!(CssShadow);
impl_css_native_shaping!(CssTextShadow);
impl_css_native_shaping!(CssBoxShadow);
impl_css_native_shaping!(CssGridTemplateColumns);
impl_css_native_shaping!(CssGridTemplateRows);
impl_css_native_shaping!(CssGridArea);
impl_css_native_shaping!(CssGridAutoColumns);
impl_css_native_shaping!(CssGridAutoRows);
impl_css_native_shaping!(CssGridAutoFlow);
impl_css_native_shaping!(CssFlex);
impl_css_native_shaping!(CssFlexDirection);
impl_css_native_shaping!(CssFlexWrap);
impl_css_native_shaping!(CssAlignItems);
impl_css_native_shaping!(CssAlignSelf);
impl_css_native_shaping!(CssJustifyItems);
impl_css_native_shaping!(CssJustifySelf);
impl_css_native_shaping!(CssJustifyContent);
impl_css_native_shaping!(CssAlignContent);
impl_css_native_shaping!(CssBoxSizing);
impl_css_native_shaping!(CssBackfaceVisibility);
impl_css_native_shaping!(CssWebkitFontSmoothing);
impl_css_native_shaping!(CssDisplay);

impl_css_native_shaping_list!(
    CssAnimationDelay,
    CssAnimationDirection,
    CssAnimationDuration,
    CssAnimationFillMode,
    // CssAnimationIterationCount,//too long
    CssAnimationName,
    CssAnimationPlayState,
    // CssAnimationTimingFunction,//too long
    CssAnimation,
    CssBackground,
    CssBackgroundBlendMode,
    CssBackgroundClip,
    CssBackgroundOrigin,
    CssBackgroundSize,
    CssBorderImage,
    CssBorderImageOutset,
    CssBorderImageRepeat,
    CssBorderImageSlice,
    CssBorderImageSource,
    CssBorderImageWidth,
    CssBoxDecorationBreak,
    CssBreakAfter,
    CssBreakBefore,
    CssBreakInside,
    CssCaretColor,
    CssClipPath,
    CssColumnRuleColor,
    CssColumnRuleStyle,
    CssColumnRuleWidth,
    CssColumns,
    CssCounterReset,
    CssFilter,
    CssFlexFlow,
    CssFlexGrow,
    CssFlexShrink,
    CssFontFamily,
    CssFontFeatureSettings,
    CssFontKerning,
    CssFontLanguageOverride,
    CssFontSizeAdjust,
    CssFontStretch,
    CssFontSynthesis,
    CssFontVariantAlternates,
    CssFontVariantCaps,
    CssFontVariantEastAsian,
    CssFontVariantLigatures,
    CssFontVariantNumeric,
    CssFontVariantPosition,
    CssGrid,
    CssGridColumn,
    CssGridColumnEnd,
    CssGridColumnStart,
    CssGridRow,
    CssGridRowEnd,
    CssGridRowStart,
    CssGridTemplate,
    CssGridTemplateAreas,
    CssHyphens,
    CssImageRendering,
    CssIsolation,
    CssLineBreak,
    CssMask,
    CssMaskType,
    CssMixBlendMode,
    CssObjectFit,
    CssObjectPosition,
    CssOpacity,
    CssOrder,
    CssOverflowWrap,
    CssPageBreakAfter,
    CssPageBreakBefore,
    CssPageBreakInside,
    CssPerspective,
    CssPerspectiveOrigin,
    CssPlaceContent,
    CssPointerEvents,
    CssResize,
    CssScrollBehavior,
    CssShapeImageThreshold,
    CssShapeMargin,
    CssTabSize,
    CssTextAlignLast,
    CssTextCombineUpright,
    CssTextDecorationLine,
    CssTextDecorationStyle,
    CssTextEmphasis,
    CssTextEmphasisColor,
    CssTextEmphasisPosition,
    CssTextEmphasisStyle,
    CssTextJustify,
    CssTextOrientation,
    CssTextOverflow,
    CssTextUnderlinePosition,
    CssTouchAction,
    CssTransform,
    CssTransformOrigin,
    CssTransformStyle,
    CssTransitionDelay,
    CssTransitionDuration,
    CssTransitionProperty,
    // CssTransitionTimingFunction,//too long
    CssUserSelect,
    CssWillChange,
    CssWordBreak,
    CssWordWrap,
    CssWritingMode
);
