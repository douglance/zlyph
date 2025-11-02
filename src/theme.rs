use gpui::{Hsla, rgb, hsla};

#[derive(Clone)]
pub struct AtomOneDark {
    pub background: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub selection: Hsla,
    pub cursor: Hsla,
    pub keyword: Hsla,
    pub string: Hsla,
    pub function: Hsla,
    pub number: Hsla,
    pub comment: Hsla,
    pub type_name: Hsla,
    pub success: Hsla,
    pub error: Hsla,
    pub warning: Hsla,
    pub info: Hsla,
}

impl Default for AtomOneDark {
    fn default() -> Self {
        Self {
            background: hsla(0.59, 0.13, 0.20, 0.75),
            border: hsla(0.59, 0.13, 0.31, 0.2),
            text: rgb(0xabb2bf).into(),
            text_muted: hsla(0.59, 0.11, 0.55, 0.6),
            selection: hsla(0.59, 0.13, 0.31, 0.7),
            cursor: rgb(0x528bff).into(),
            keyword: rgb(0xc678dd).into(),
            string: rgb(0x98c379).into(),
            function: rgb(0x61afef).into(),
            number: rgb(0xd19a66).into(),
            comment: hsla(0.59, 0.10, 0.41, 0.7),
            type_name: rgb(0xe5c07b).into(),
            success: rgb(0x98c379).into(),
            error: rgb(0xe06c75).into(),
            warning: rgb(0xe5c07b).into(),
            info: rgb(0x61afef).into(),
        }
    }
}
