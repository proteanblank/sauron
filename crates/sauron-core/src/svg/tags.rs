//! Provides macro for creating functions for tags
//!

macro_rules! declare_svg_tags{

    ( $(
         $(#[$attr:meta])*
         $name:ident;
       )*
     ) => {
        $(
            doc_comment!{
                concat!("Creates an svg [",stringify!($name),"](/https://developer.mozilla.org/en-US/docs/Web/SVG/Element/",stringify!($name),") element"),

                $(#[$attr])*
                #[inline]
                #[allow(non_snake_case)]
                pub fn $name<MSG>(attrs: Vec<$crate::Attribute<MSG>>, children: Vec<$crate::Node<MSG>>) -> $crate::Node<MSG>
                    {
                        $crate::svg::svg_element(stringify!($name), attrs, children)
                }
            }
         )*
    };

    ( $(
         $(#[$attr:meta])*
         $name:ident => $tagname:tt;
       )*
     ) => {
        $(
            doc_comment!{
                concat!("Creates an svg [",$tagname,"](/https://developer.mozilla.org/en-US/docs/Web/SVG/Element/",$tagname,") element"),

                $(#[$attr])*
                #[inline]
                #[allow(non_snake_case)]
                pub fn $name<MSG>(attrs: Vec<$crate::Attribute<MSG>>, children: Vec<$crate::Node<MSG>>) -> $crate::Node<MSG>
                    {
                        $crate::svg::svg_element($tagname, attrs, children)
                 }
            }
         )*
    }

}

/// declare common svg tags that is not in conflict with the html tags
/// at the same time this also fills the SVG_TAGS const with all the svg tags
macro_rules! declare_common_svg_tags_and_macro {
    ($($(#[$attr:meta])* $name:ident;)*) => {

        pub(crate) mod commons {
            declare_svg_tags! { $($name;)* }
        }


        #[cfg(feature = "with-parser")]
        /// These are the commonly used svg tags such as rect, circle, path, arc, ..etc.
        pub const SVG_TAGS: [&'static str; 65] = [ $(stringify!($name),)* ];

    };
}

/// declare svg tags, at the same time this also
/// fills up the SVG_TAGS_SPECIAL const with the svg tags that are not
/// regular identifiers
macro_rules! declare_svg_tags_special{
    ( $(
         $(#[$attr:meta])*
         $name:ident => $attribute:tt;
       )*
     ) => {
        declare_svg_tags!{ $($name=>$attribute;)*}

        #[cfg(feature = "with-parser")]
        /// These are svg tags which the tags are non proper rust identifier, so they are handled
        /// differently
        pub const SVG_TAGS_SPECIAL:[(&'static str,&'static str); 2] = [$((stringify!($name),$attribute),)*];
    }
}

macro_rules! declare_svg_tags_non_common{

    ( $(
         $(#[$attr:meta])*
         $name:ident;
       )*
     ) => {
        declare_svg_tags!{ $($name;)*}

        #[cfg(feature = "with-parser")]
        /// These are collection of svg tags that are non commonly used put together in this
        /// collection so as not to create import conflicts with the common tags
        ///
        /// Warning: These are not included in the NAMESPACED TAGS in sauron-parse
        ///     Any attempt to use them inside of a node! macro inside svg may not work correcly
        pub const SVG_TAGS_NON_COMMON:[&'static str;6] = [$(stringify!($name),)*];
    }
}

declare_common_svg_tags_and_macro! {
    animate;
    animateMotion;
    animateTransform;
    circle;
    clipPath;
    defs;
    desc;
    discard;
    ellipse;
    feBlend;
    feColorMatrix;
    feComponentTransfer;
    feComposite;
    feConvolveMatrix;
    feDiffuseLighting;
    feDisplacementMap;
    feDistantLight;
    feDropShadow;
    feFlood;
    feFuncA;
    feFuncB;
    feFuncG;
    feFuncR;
    feGaussianBlur;
    feImage;
    feMerge;
    feMergeNode;
    feMorphology;
    feOffset;
    fePointLight;
    feSpecularLighting;
    feSpotLight;
    feTile;
    feTurbulence;
    filter;
    foreignObject;
    g;
    hatch;
    hatchpath;
    image;
    linearGradient;
    marker;
    mask;
    mesh;
    meshgradient;
    meshpatch;
    meshrow;
    metadata;
    mpath;
    path;
    pattern;
    polygon;
    polyline;
    radialGradient;
    rect;
    set;
    solidcolor;
    stop;
    svg;
    switch;
    symbol;
    textPath;
    tspan;
    unknown;
    view;
}
declare_svg_tags_special! {
    color_profile => "color-profile";
    r#use => "use";
}

// These are non-common tags
// which the users need to explicitly import using
// svg::tags::style, svg::tags::text, svg::tags::title, etc
//
declare_svg_tags_non_common! {
    line; // since this conflicts with std::line! macro, std::line                > svg::tags::line
    script; // this conflicts with html::script        , html::tags::script       > svg::tags::script
    style; // conflics with html::attributes::style    , html::attributes::style  > svg::tags::style
    text; // conflicts with html::text                 , html::text               > svg::tags::text
    a;   // conflicts with html::a                     , html::tags::a            > svg::tags::a
    title;  // conflicts with html::attributes::title  , html::attributes::title  > svg::tags::title
}
