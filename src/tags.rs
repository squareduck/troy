//! Helper builders for standart HTML tags.
//! https://developer.mozilla.org/en-US/docs/Web/HTML/Element
//!

use element::VElement;

// Macro to create tags in bulk
macro_rules! tags {
    ($($tag:ident),*) => {
        $(
            pub fn $tag() -> VElement {
                VElement::new(stringify!($tag))
            }
        )*
    };
}

// Macro to create void tags in bulk
macro_rules! void_tags {
    ($($tag:ident),*) => {
        $(
            pub fn $tag() -> VElement {
                VElement::new_void(stringify!($tag))
            }
        )*
    };
}

// Main root
tags![html];

// Document metadata
tags![style, title];
void_tags![link, meta];

// Sectioning root
tags![body];

// Content sectioning
#[cfg_attr(rustfmt, rustfmt_skip)]
tags![
    address, article, aside, footer, header, h1, h2, h3, h4, h5, h6, nav,
    section
];

// Text content
tags![
    blockquote, dd, div, dl, dt, figcaption, figure, li, main, ol, p, pre, ul
];
void_tags![hr];

// Inline text semantics
#[cfg_attr(rustfmt, rustfmt_skip)]
tags![
    a, abbr, b, bdi, bdo, cite, code, data, dfn, em, i, kbd, mark, q, rp, rt,
    rtc, ruby, s, samp, small, span, strong, sub, sup, time, u, var
];
void_tags![br, wbr];

// Image and multimedia
tags![area, audio, map, video];
void_tags![img, track];

// Embedded content
tags![iframe, object, picture];
void_tags![embed, param, source];

// Scripting
tags![canvas, noscript, script];

// Demarkating edits
tags![del, ins];

// Table content
tags![
    caption, col, colgroup, table, tbody, td, tfoot, th, thead, tr
];

// Forms
#[cfg_attr(rustfmt, rustfmt_skip)]
tags![
    button, datalist, fieldlist, form, label, legend, meter, optgroup, option,
    output, progress, select, textarea
];
void_tags![input];

// Interactive elements

tags![details, dialog, menu, menuitem, summary];
