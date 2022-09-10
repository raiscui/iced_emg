/*
 * @Author: Rais
 * @Date: 2022-08-29 23:19:00
 * @LastEditTime: 2022-09-10 14:38:30
 * @LastEditors: Rais
 * @Description:
 */

use std::rc::Rc;

use crate::EmgEdgeItem;
use emg_common::TypeCheck;
use emg_refresh::{RefreshFor, RefreshWhoNoWarper};
use emg_state::CloneStateVar;
#[allow(clippy::wildcard_imports)]
use seed_styles::*;

// impl<Ix, RenderCtx> RefreshFor<EmgEdgeItem<Ix, RenderCtx>> for CssFill
// where
//     Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
//     EmgEdgeItem<Ix, RenderCtx>: RefreshWhoNoWarper,
//     RenderCtx: 'static,
// {
//     fn refresh_for(&self, who: &mut EmgEdgeItem<Ix, RenderCtx>) {
//         let type_name = Self::TYPE_NAME;
//         who.styles.update(|s| {
//             s.insert(type_name, Rc::new(self.clone()));
//         });
//     }
// }

macro_rules! impl_css_native_refresh {
    ($css:ident) => {
        impl<Ix, RenderCtx> RefreshFor<EmgEdgeItem<Ix, RenderCtx>> for $css
        where
            Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
            EmgEdgeItem<Ix, RenderCtx>: RefreshWhoNoWarper,
            RenderCtx: 'static,
        {
            fn refresh_for(&self, who: &mut EmgEdgeItem<Ix, RenderCtx>) {
                let type_name = Self::TYPE_NAME;
                who.styles.update(|s| {
                    s.insert(type_name, Rc::new(self.clone()));
                });
            }
        }
    };
}
macro_rules! impl_css_native_refresh_list {
    ($($css:ident),*) => {
        $(
            impl<Ix, RenderCtx> RefreshFor<EmgEdgeItem<Ix, RenderCtx>> for $css
            where
                Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
                EmgEdgeItem<Ix, RenderCtx>: RefreshWhoNoWarper,
                RenderCtx: 'static,
            {
                fn refresh_for(&self, who: &mut EmgEdgeItem<Ix, RenderCtx>) {
                    let type_name = Self::TYPE_NAME;
                    who.styles.update(|s| {
                        s.insert(type_name, Rc::new(self.clone()));
                    });
                }
            }
        )*
    };
}

impl_css_native_refresh!(CssBackgroundAttachment);
impl_css_native_refresh!(CssColumnSpan);
impl_css_native_refresh!(CssColumnsFill);
impl_css_native_refresh!(CssColumnRule);
impl_css_native_refresh!(CssColumnCount);
impl_css_native_refresh!(CssColumnWidth);
impl_css_native_refresh!(CssRaw);
impl_css_native_refresh!(CssBackgroundImage);
impl_css_native_refresh!(CssBackgroundPosition);
impl_css_native_refresh!(CssBackgroundRepeat);
impl_css_native_refresh!(CssBorderCollapse);
// impl_css_native_refresh!(CssBorderSpacing);
impl_css_native_refresh!(CssCaptionSide);
impl_css_native_refresh!(CssClear);
impl_css_native_refresh!(CssClip);
impl_css_native_refresh!(CssContent);
impl_css_native_refresh!(CssCounterIncrement);
impl_css_native_refresh!(CssCursor);
impl_css_native_refresh!(CssDirection);
impl_css_native_refresh!(CssEmptyCells);
impl_css_native_refresh!(CssFloat);
impl_css_native_refresh!(CssFontStyle);
impl_css_native_refresh!(CssFontVariant);
impl_css_native_refresh!(CssFontWeight);
impl_css_native_refresh!(CssListStyleImage);
impl_css_native_refresh!(CssListStylePosition);
impl_css_native_refresh!(CssListStyleType);
impl_css_native_refresh!(CssListStyle);
impl_css_native_refresh!(CssOrphans);
impl_css_native_refresh!(CssOverflow);
impl_css_native_refresh!(CssOverflowX);
impl_css_native_refresh!(CssOverflowY);
impl_css_native_refresh!(CssPageBreak);
impl_css_native_refresh!(CssPosition);
impl_css_native_refresh!(CssQuotes);
impl_css_native_refresh!(CssTableLayout);
impl_css_native_refresh!(CssTextAlign);
impl_css_native_refresh!(CssTextDecoration);
impl_css_native_refresh!(CssTextDecorationColor);
impl_css_native_refresh!(CssTextIndent);
impl_css_native_refresh!(CssTextTransform);
impl_css_native_refresh!(CssUnicodeBidi);
impl_css_native_refresh!(CssVerticalAlign);
impl_css_native_refresh!(CssVisibility);
impl_css_native_refresh!(CssWhiteSpace);
impl_css_native_refresh!(CssWidows);
impl_css_native_refresh!(CssWordSpacing);
impl_css_native_refresh!(CssZIndex);
impl_css_native_refresh!(CssMargin);
impl_css_native_refresh!(CssMarginTop);
impl_css_native_refresh!(CssMarginBottom);
impl_css_native_refresh!(CssMarginLeft);
impl_css_native_refresh!(CssMarginRight);
impl_css_native_refresh!(CssSpace);
impl_css_native_refresh!(CssTop);
impl_css_native_refresh!(CssBottom);
impl_css_native_refresh!(CssLeft);
impl_css_native_refresh!(CssRight);
impl_css_native_refresh!(CssGridGap);
impl_css_native_refresh!(CssGridColumnGap);
impl_css_native_refresh!(CssGridRowGap);
impl_css_native_refresh!(CssGap);
impl_css_native_refresh!(CssColumnGap);
impl_css_native_refresh!(CssRowGap);
impl_css_native_refresh!(CssPadding);
impl_css_native_refresh!(CssPaddingTop);
impl_css_native_refresh!(CssPaddingRight);
impl_css_native_refresh!(CssPaddingLeft);
impl_css_native_refresh!(CssPaddingBottom);
impl_css_native_refresh!(CssBorderStyle);
impl_css_native_refresh!(CssBorderLeftStyle);
impl_css_native_refresh!(CssBorderRightStyle);
impl_css_native_refresh!(CssBorderTopStyle);
impl_css_native_refresh!(CssBorderBottomStyle);
impl_css_native_refresh!(CssOutlineStyle);
impl_css_native_refresh!(CssOutlineLeftStyle);
impl_css_native_refresh!(CssOutlineRightStyle);
impl_css_native_refresh!(CssOutlineTopStyle);
impl_css_native_refresh!(CssOutlineBottomStyle);
impl_css_native_refresh!(CssBorderWidth);
impl_css_native_refresh!(CssBorderLeftWidth);
impl_css_native_refresh!(CssBorderRightWidth);
impl_css_native_refresh!(CssBorderTopWidth);
impl_css_native_refresh!(CssBorderBottomWidth);
impl_css_native_refresh!(CssOutlineWidth);
impl_css_native_refresh!(CssOutlineLeftWidth);
impl_css_native_refresh!(CssOutlineRightWidth);
impl_css_native_refresh!(CssOutlineTopWidth);
impl_css_native_refresh!(CssOutlineBottomWidth);
impl_css_native_refresh!(CssSize);
impl_css_native_refresh!(CssFlexBasis);
// impl_css_native_refresh!(CssWidth);
// impl_css_native_refresh!(CssHeight);
impl_css_native_refresh!(CssMinWidth);
impl_css_native_refresh!(CssMaxWidth);
impl_css_native_refresh!(CssMinHeight);
impl_css_native_refresh!(CssMaxHeight);
impl_css_native_refresh!(CssBorder);
impl_css_native_refresh!(CssBorderLeft);
impl_css_native_refresh!(CssBorderRight);
impl_css_native_refresh!(CssBorderTop);
impl_css_native_refresh!(CssBorderBottom);
impl_css_native_refresh!(CssOutline);
impl_css_native_refresh!(CssOutlineLeft);
impl_css_native_refresh!(CssOutlineRight);
impl_css_native_refresh!(CssOutlineTop);
impl_css_native_refresh!(CssOutlineBottom);
impl_css_native_refresh!(CssTransition);
impl_css_native_refresh!(CssLetterSpacing);
impl_css_native_refresh!(CssLineHeight);
impl_css_native_refresh!(CssBorderRadius);
impl_css_native_refresh!(CssBorderTopRightRadius);
impl_css_native_refresh!(CssBorderTopLeftRadius);
impl_css_native_refresh!(CssBorderBRRadius);
impl_css_native_refresh!(CssBorderBLRadius);
impl_css_native_refresh!(CssFont);
impl_css_native_refresh!(CssFontSize);
impl_css_native_refresh!(CssColor);
impl_css_native_refresh!(CssStroke);
impl_css_native_refresh!(CssBackgroundColor);
impl_css_native_refresh!(CssFill);
impl_css_native_refresh!(CssBorderColor);
impl_css_native_refresh!(CssBorderLeftColor);
impl_css_native_refresh!(CssBorderRightColor);
impl_css_native_refresh!(CssBorderTopColor);
impl_css_native_refresh!(CssBorderBottomColor);
impl_css_native_refresh!(CssOutlineColor);
impl_css_native_refresh!(CssOutlineLeftColor);
impl_css_native_refresh!(CssOutlineRightColor);
impl_css_native_refresh!(CssOutlineTopColor);
impl_css_native_refresh!(CssOutlineBottomColor);
impl_css_native_refresh!(CssShadow);
impl_css_native_refresh!(CssTextShadow);
impl_css_native_refresh!(CssBoxShadow);
impl_css_native_refresh!(CssGridTemplateColumns);
impl_css_native_refresh!(CssGridTemplateRows);
impl_css_native_refresh!(CssGridArea);
impl_css_native_refresh!(CssGridAutoColumns);
impl_css_native_refresh!(CssGridAutoRows);
impl_css_native_refresh!(CssGridAutoFlow);
impl_css_native_refresh!(CssFlex);
impl_css_native_refresh!(CssFlexDirection);
impl_css_native_refresh!(CssFlexWrap);
impl_css_native_refresh!(CssAlignItems);
impl_css_native_refresh!(CssAlignSelf);
impl_css_native_refresh!(CssJustifyItems);
impl_css_native_refresh!(CssJustifySelf);
impl_css_native_refresh!(CssJustifyContent);
impl_css_native_refresh!(CssAlignContent);
impl_css_native_refresh!(CssBoxSizing);
impl_css_native_refresh!(CssBackfaceVisibility);
impl_css_native_refresh!(CssWebkitFontSmoothing);
impl_css_native_refresh!(CssDisplay);

impl_css_native_refresh_list!(
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
