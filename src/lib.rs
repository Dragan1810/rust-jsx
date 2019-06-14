mod tokenizer;

use crate::tokenizer::{parse_html_token, HtmlOpenToken, HtmlToken, TokenizeError};
use proc_macro2::{Ident, TokenStream, TokenTree};
/// An attribute that's present on either a [`SnaxTag`] or a
/// [`SnaxSelfClosingTag`].
///
/// [`SnaxTag`]: struct.SnaxTag.html
/// [`SnaxSelfClosingTag`]: struct.SnaxSelfClosingTag.html
#[derive(Debug)]
pub enum SnaxAttribute {
    /// ```html
    /// <div foo="bar" />
    ///      ^^^^^^^^^
    ///      SnaxAttribute::Simple {
    ///          name: Ident(foo),
    ///          value: TokenTree("bar"),
    ///      }
    /// ```
    Simple { name: Ident, value: TokenTree },
}

impl PartialEq for SnaxAttribute {
    fn eq(&self, other: &Self) -> bool {
        use SnaxAttribute::*;

        match (self, other) {
            (
                Simple { name, value },
                Simple {
                    name: other_name,
                    value: other_value,
                },
            ) => name == other_name && value.to_string() == other_value.to_string(),
        }
    }
}

/// One complete block in the syntax.
///
/// For more information, look at the documentation for the struct that each
/// variant wraps.
#[derive(Debug)]
pub enum SnaxItem {
    /// A standard tag, which can have attributes and children.
    Tag(SnaxTag),

    /// An empty tag, which can only have attributes.
    SelfClosingTag(SnaxSelfClosingTag),

    /// A block of content, which can contain any Rust expression.
    Content(TokenTree),
}

impl PartialEq for SnaxItem {
    fn eq(&self, other: &Self) -> bool {
        use SnaxItem::*;

        match (self, other) {
            (Tag(this), Tag(other)) => this == other,
            (SelfClosingTag(this), SelfClosingTag(other)) => this == other,
            (Content(this), Content(other)) => this.to_string() == other.to_string(),
            _ => false,
        }
    }
}

/// A standard tag, which can have attributes and children.
///
/// ```html
/// <div hello="world">"Hey!"</div>
/// ```
#[derive(Debug, PartialEq)]
pub struct SnaxTag {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
    pub children: Vec<SnaxItem>,
}

/// A self-closing tag, which doesn't have children:
///
/// ```html
/// <meta name="foo" value="bar" />
/// ```
///
/// Note that snax_syntax does not support automatically closing unclosed
/// tags like HTML does, such as `<br>`. These tags need to be written as
/// `<br />` in order to simplify parsing.
#[derive(Debug, PartialEq)]
pub struct SnaxSelfClosingTag {
    pub name: Ident,
    pub attributes: Vec<SnaxAttribute>,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEnd,
    UnexpectedItem(HtmlToken),
    UnexpectedToken(TokenTree),
}

impl From<TokenizeError> for ParseError {
    fn from(error: TokenizeError) -> ParseError {
        match error {
            TokenizeError::UnexpectedEnd => ParseError::UnexpectedEnd,
            TokenizeError::UnexpectedToken(token) => ParseError::UnexpectedToken(token),
        }
    }
}

macro_rules! expect_end {
    ($iterator: expr) => {
        match $iterator.next() {
            None => {}
            Some(unexpected) => return Err(ParseError::UnexpectedToken(unexpected)),
        }
    };
}

#[derive(Debug)]
enum OpenToken {
    Tag(HtmlOpenToken),
}

/// Attempts to parse a `proc_macro2::TokenStream` into a `SnaxItem`.
pub fn parse(input_stream: TokenStream) -> Result<SnaxItem, ParseError> {
    let mut input = input_stream.into_iter();
    let mut tag_stack: Vec<(OpenToken, Vec<SnaxItem>)> = Vec::new();

    loop {
        match parse_html_token(&mut input)? {
            HtmlToken::OpenTag(opening_tag) => {
                tag_stack.push((OpenToken::Tag(opening_tag), Vec::new()));
            }
            HtmlToken::CloseTag(closing_tag) => {
                let (open_token, children) = tag_stack.pop().ok_or_else(|| {
                    ParseError::UnexpectedItem(HtmlToken::CloseTag(closing_tag.clone()))
                })?;

                let opening_tag = match open_token {
                    OpenToken::Tag(tag) => tag,
                };

                assert_eq!(opening_tag.name, closing_tag.name);

                let tag = SnaxTag {
                    name: opening_tag.name,
                    attributes: opening_tag.attributes,
                    children,
                };

                match tag_stack.last_mut() {
                    None => {
                        expect_end!(input);
                        return Ok(SnaxItem::Tag(tag));
                    }
                    Some((_, parent_children)) => {
                        parent_children.push(SnaxItem::Tag(tag));
                    }
                }
            }

            HtmlToken::SelfClosingTag(self_closing_tag) => {
                let tag = SnaxSelfClosingTag {
                    name: self_closing_tag.name,
                    attributes: self_closing_tag.attributes,
                };

                match tag_stack.last_mut() {
                    None => {
                        expect_end!(input);
                        return Ok(SnaxItem::SelfClosingTag(tag));
                    }
                    Some((_, parent_children)) => {
                        parent_children.push(SnaxItem::SelfClosingTag(tag));
                    }
                }
            }
            HtmlToken::Textish(textish) => match tag_stack.last_mut() {
                None => {
                    expect_end!(input);
                    return Ok(SnaxItem::Content(textish.content));
                }
                Some((_, parent_children)) => {
                    parent_children.push(SnaxItem::Content(textish.content));
                }
            },
        }
    }
}