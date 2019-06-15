use proc_macro2::{Ident, Span};
use quote::quote;

use rust_jsx::{SnaxAttribute, SnaxItem, SnaxSelfClosingTag, SnaxTag};

/// Like quote!, but returns a single TokenTree instead
macro_rules! quote_one {
    ($($value: tt)*) => {
        quote!($($value)*).into_iter().next().unwrap()
    };
}

#[test]
fn just_string() {
    let input = quote!("hello");
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::Content(quote_one!("hello"));
    assert_eq!(output, expected);
}

#[test]
fn just_number() {
    let input = quote!(5);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::Content(quote_one!(5));
    assert_eq!(output, expected);
}

#[test]
fn empty_div() {
    let input = quote!(<div></div>);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn self_closing_div() {
    let input = quote!(<div />);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_div_comment() {
    let input = quote!(<div>/* Hello, world! */</div>);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_with_literal_attributes() {
    let input = quote!(<div foo="bar" baz="qux"></div>);
    let output = rust_jsx::parse(input).unwrap();
    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: vec![
            SnaxAttribute::Simple {
                name: Ident::new("foo", Span::call_site()),
                value: quote_one!("bar"),
            },
            SnaxAttribute::Simple {
                name: Ident::new("baz", Span::call_site()),
                value: quote_one!("qux"),
            },
        ],
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn empty_with_block_attribute() {
    let input = quote!(<label sum={ 5 + 5 }></label>);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("label", Span::call_site()),
        attributes: vec![SnaxAttribute::Simple {
            name: Ident::new("sum", Span::call_site()),
            value: quote_one!({ 5 + 5 }),
        }],
        children: Default::default(),
    });

    assert_eq!(output, expected);
}

#[test]
fn self_closing_with_literal_attributes() {
    let input = quote!(<div foo="bar" baz="qux" />);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
        name: Ident::new("div", Span::call_site()),
        attributes: vec![
            SnaxAttribute::Simple {
                name: Ident::new("foo", Span::call_site()),
                value: quote_one!("bar"),
            },
            SnaxAttribute::Simple {
                name: Ident::new("baz", Span::call_site()),
                value: quote_one!("qux"),
            },
        ],
    });

    assert_eq!(output, expected);
}

#[test]
fn self_closing_with_block_attribute() {
    let input = quote!(<label sum={ 5 + 5 } />);
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::SelfClosingTag(SnaxSelfClosingTag {
        name: Ident::new("label", Span::call_site()),
        attributes: vec![SnaxAttribute::Simple {
            name: Ident::new("sum", Span::call_site()),
            value: quote_one!({ 5 + 5 }),
        }],
    });

    assert_eq!(output, expected);
}

#[test]
fn nested_tags() {
    let input = quote!(
        <div>
            <span></span>
        </div>
    );
    let output = rust_jsx::parse(input).unwrap();

    let expected = SnaxItem::Tag(SnaxTag {
        name: Ident::new("div", Span::call_site()),
        attributes: Default::default(),
        children: vec![SnaxItem::Tag(SnaxTag {
            name: Ident::new("span", Span::call_site()),
            attributes: Default::default(),
            children: Default::default(),
        })],
    });

    assert_eq!(output, expected);
}