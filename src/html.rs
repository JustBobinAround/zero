use macros::html;
use std::collections::HashMap;

pub type HTML<'a> = Vec<Tag<'a>>;
#[derive(Debug)]
pub struct Tag<'a> {
    pub ty: TagType,
    pub attrs: HashMap<Text<'a>, Text<'a>>,
    pub content: Markup<'a>,
}

//TODO: sanatize strings
impl<'a> std::fmt::Display for Tag<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = &self.ty.as_str();
        write!(f, "<{} ", tag)?;
        for (k, v) in self.attrs.iter() {
            write!(f, "{}=\"{}\" ", k, v)?;
        }
        write!(f, ">")?;
        self.content.fmt(f)?;
        write!(f, "</{}>", tag)
    }
}

#[derive(Debug)]
pub enum Markup<'a> {
    Text(Text<'a>),
    Html(HTML<'a>),
    None,
}
impl<'a> std::fmt::Display for Markup<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => t.fmt(f),
            Self::Html(h) => {
                for tag in h {
                    tag.fmt(f)?;
                }
                Ok(())
            }
            Self::None => Ok(()),
        }
    }
}
impl<'a> From<&'a str> for Markup<'a> {
    fn from(value: &'a str) -> Self {
        Markup::Text(value.into())
    }
}
impl<'a> From<Text<'a>> for Markup<'a> {
    fn from(value: Text<'a>) -> Self {
        Markup::Text(value)
    }
}
impl<'a> From<HTML<'a>> for Markup<'a> {
    fn from(value: HTML<'a>) -> Self {
        Markup::Html(value)
    }
}
impl From<()> for Markup<'_> {
    fn from(_value: ()) -> Self {
        Markup::None
    }
}

use std::borrow::Cow;
use std::hash::Hasher;

#[derive(Debug)]
pub struct Text<'a>(Cow<'a, str>);
impl<'a> std::fmt::Display for Text<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Text<'a> {
    pub fn borrowed(s: &'a str) -> Self {
        Text(Cow::Borrowed(s))
    }

    pub fn owned(s: String) -> Self {
        Text(Cow::Owned(s))
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(value: &'a str) -> Self {
        Text::borrowed(value)
    }
}

impl From<String> for Text<'_> {
    fn from(value: String) -> Self {
        Text::owned(value)
    }
}

impl<'a> PartialEq for Text<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> Eq for Text<'a> {}
impl<'a> std::hash::Hash for Text<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state)
    }
}

#[derive(Debug)]
pub enum TagType {
    A,
    ABBR,
    ACRONYM,
    ADDRESS,
    APPLET,
    AREA,
    ARTICLE,
    ASIDE,
    AUDIO,
    B,
    BASE,
    BASEFONT,
    BDI,
    BDO,
    BIG,
    BLOCKQUOTE,
    BODY,
    BR,
    BUTTON,
    CANVAS,
    CAPTION,
    CENTER,
    CITE,
    CODE,
    COL,
    COLGROUP,
    DATA,
    DATALIST,
    DD,
    DEL,
    DETAILS,
    DFN,
    DIALOG,
    DIR,
    DIV,
    DL,
    DT,
    EM,
    EMBED,
    FIELDSET,
    FIGCAPTION,
    FIGURE,
    FONT,
    FOOTER,
    FORM,
    FRAME,
    FRAMESET,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    HEAD,
    HEADER,
    HGROUP,
    HR,
    HTML,
    I,
    IFRAME,
    IMG,
    INPUT,
    INS,
    KBD,
    LABEL,
    LEGEND,
    LI,
    LINK,
    MAIN,
    MAP,
    MARK,
    MENU,
    META,
    METER,
    NAV,
    NOFRAMES,
    NOSCRIPT,
    OBJECT,
    OL,
    OPTGROUP,
    OPTION,
    OUTPUT,
    P,
    PARAM,
    PICTURE,
    PRE,
    PROGRESS,
    Q,
    RP,
    RT,
    RUBY,
    S,
    SAMP,
    SCRIPT,
    SEARCH,
    SECTION,
    SELECT,
    SMALL,
    SOURCE,
    SPAN,
    STRIKE,
    STRONG,
    STYLE,
    SUB,
    SUMMARY,
    SUP,
    SVG,
    TABLE,
    TBODY,
    TD,
    TEMPLATE,
    TEXTAREA,
    TFOOT,
    TH,
    THEAD,
    TIME,
    TITLE,
    TR,
    TRACK,
    TT,
    U,
    UL,
    VAR,
    VIDEO,
    WBR,
}

impl TagType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            TagType::A => "a",
            TagType::ABBR => "abbr",
            TagType::ACRONYM => "acronym",
            TagType::ADDRESS => "address",
            TagType::APPLET => "applet",
            TagType::AREA => "area",
            TagType::ARTICLE => "article",
            TagType::ASIDE => "aside",
            TagType::AUDIO => "audio",
            TagType::B => "b",
            TagType::BASE => "base",
            TagType::BASEFONT => "basefont",
            TagType::BDI => "bdi",
            TagType::BDO => "bdo",
            TagType::BIG => "big",
            TagType::BLOCKQUOTE => "blockquote",
            TagType::BODY => "body",
            TagType::BR => "br",
            TagType::BUTTON => "button",
            TagType::CANVAS => "canvas",
            TagType::CAPTION => "caption",
            TagType::CENTER => "center",
            TagType::CITE => "cite",
            TagType::CODE => "code",
            TagType::COL => "col",
            TagType::COLGROUP => "colgroup",
            TagType::DATA => "data",
            TagType::DATALIST => "datalist",
            TagType::DD => "dd",
            TagType::DEL => "del",
            TagType::DETAILS => "details",
            TagType::DFN => "dfn",
            TagType::DIALOG => "dialog",
            TagType::DIR => "dir",
            TagType::DIV => "div",
            TagType::DL => "dl",
            TagType::DT => "dt",
            TagType::EM => "em",
            TagType::EMBED => "embed",
            TagType::FIELDSET => "fieldset",
            TagType::FIGCAPTION => "figcaption",
            TagType::FIGURE => "figure",
            TagType::FONT => "font",
            TagType::FOOTER => "footer",
            TagType::FORM => "form",
            TagType::FRAME => "frame",
            TagType::FRAMESET => "frameset",
            TagType::H1 => "h1",
            TagType::H2 => "h2",
            TagType::H3 => "h3",
            TagType::H4 => "h4",
            TagType::H5 => "h5",
            TagType::H6 => "h6",
            TagType::HEAD => "head",
            TagType::HEADER => "header",
            TagType::HGROUP => "hgroup",
            TagType::HR => "hr",
            TagType::HTML => "html",
            TagType::I => "i",
            TagType::IFRAME => "iframe",
            TagType::IMG => "img",
            TagType::INPUT => "input",
            TagType::INS => "ins",
            TagType::KBD => "kbd",
            TagType::LABEL => "label",
            TagType::LEGEND => "legend",
            TagType::LI => "li",
            TagType::LINK => "link",
            TagType::MAIN => "main",
            TagType::MAP => "map",
            TagType::MARK => "mark",
            TagType::MENU => "menu",
            TagType::META => "meta",
            TagType::METER => "meter",
            TagType::NAV => "nav",
            TagType::NOFRAMES => "noframes",
            TagType::NOSCRIPT => "noscript",
            TagType::OBJECT => "object",
            TagType::OL => "ol",
            TagType::OPTGROUP => "optgroup",
            TagType::OPTION => "option",
            TagType::OUTPUT => "output",
            TagType::P => "p",
            TagType::PARAM => "param",
            TagType::PICTURE => "picture",
            TagType::PRE => "pre",
            TagType::PROGRESS => "progress",
            TagType::Q => "q",
            TagType::RP => "rp",
            TagType::RT => "rt",
            TagType::RUBY => "ruby",
            TagType::S => "s",
            TagType::SAMP => "samp",
            TagType::SCRIPT => "script",
            TagType::SEARCH => "search",
            TagType::SECTION => "section",
            TagType::SELECT => "select",
            TagType::SMALL => "small",
            TagType::SOURCE => "source",
            TagType::SPAN => "span",
            TagType::STRIKE => "strike",
            TagType::STRONG => "strong",
            TagType::STYLE => "style",
            TagType::SUB => "sub",
            TagType::SUMMARY => "summary",
            TagType::SUP => "sup",
            TagType::SVG => "svg",
            TagType::TABLE => "table",
            TagType::TBODY => "tbody",
            TagType::TD => "td",
            TagType::TEMPLATE => "template",
            TagType::TEXTAREA => "textarea",
            TagType::TFOOT => "tfoot",
            TagType::TH => "th",
            TagType::THEAD => "thead",
            TagType::TIME => "time",
            TagType::TITLE => "title",
            TagType::TR => "tr",
            TagType::TRACK => "track",
            TagType::TT => "tt",
            TagType::U => "u",
            TagType::UL => "ul",
            TagType::VAR => "var",
            TagType::VIDEO => "video",
            TagType::WBR => "wbr",
        }
    }
}

impl From<TagType> for &'static str {
    fn from(value: TagType) -> Self {
        value.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro() {
        Tag {
            ty: TagType::P,
            attrs: HashMap::new(),
            content: Markup::None,
        };
    }
}
