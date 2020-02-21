//! This module parses literal html returns sauron dom tree

use crate::{
    html::{
        attributes::{
            HTML_ATTRS,
            HTML_ATTRS_SPECIAL,
        },
        tags::{
            HTML_TAGS,
            HTML_TAGS_NON_COMMON,
            HTML_TAGS_WITH_MACRO_NON_COMMON,
        },
        text,
    },
    sauron_vdom::builder::element,
    svg::{
        attributes::{
            SVG_ATTRS,
            SVG_ATTRS_SPECIAL,
            SVG_ATTRS_XLINK,
        },
        tags::{
            SVG_TAGS,
            SVG_TAGS_NON_COMMON,
            SVG_TAGS_SPECIAL,
        },
    },
    Event,
};
use html5ever::{
    local_name,
    namespace_url,
    ns,
    parse_document,
    parse_fragment,
    tendril::TendrilSink,
    QualName,
};
use markup5ever_rcdom::{
    Handle,
    NodeData,
    RcDom,
};
use std::{
    io,
    io::Cursor,
};
use thiserror::Error;
use to_syntax::ToSyntax;

pub(crate) type Node = sauron_vdom::Node<String, String, Event, ()>;
pub(crate) type Attribute = sauron_vdom::Attribute<String, Event, ()>;
pub(crate) type Element = sauron_vdom::Element<String, String, Event, ()>;

mod to_syntax;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Generic Error {0}")]
    Generic(String),
    #[error("{0}")]
    IoError(#[from] io::Error),
}

fn parse_doc<'a>(node: &Handle) -> Option<Node> {
    match node.data {
        NodeData::Document => process_node(node),
        _ => None,
    }
}

fn match_tag(tag: &str) -> Option<String> {
    HTML_TAGS
        .iter()
        .chain(HTML_TAGS_NON_COMMON.iter())
        .chain(HTML_TAGS_WITH_MACRO_NON_COMMON.iter())
        .chain(SVG_TAGS.iter())
        .chain(SVG_TAGS_NON_COMMON.iter())
        .find(|item| item.eq_ignore_ascii_case(&tag))
        .map(|item| item.to_string())
        .or_else(|| {
            SVG_TAGS_SPECIAL
                .iter()
                .find(|(_func, item)| item.eq_ignore_ascii_case(&tag))
                .map(|(func, _item)| func.to_string())
        })
}

fn match_attribute(key: &str) -> Option<String> {
    HTML_ATTRS
        .iter()
        .chain(SVG_ATTRS.iter())
        .find(|att| att.eq_ignore_ascii_case(key))
        .map(|att| att.to_string())
        .or_else(|| {
            HTML_ATTRS_SPECIAL
                .iter()
                .chain(SVG_ATTRS_SPECIAL.iter())
                .chain(SVG_ATTRS_XLINK.iter())
                .find(|(_func, att)| att.eq_ignore_ascii_case(key))
                .map(|(func, _att)| func.to_string())
        })
}

fn extract_attributes(attrs: &Vec<html5ever::Attribute>) -> Vec<Attribute> {
    attrs
        .iter()
        .filter_map(|att| {
            let key = att.name.local.to_string();
            let value = att.value.to_string();
            if let Some(attr) = match_attribute(&key) {
                Some(crate::html::attributes::attr(attr.to_string(), value))
            } else {
                None
            }
        })
        .collect()
}

fn process_children(node: &Handle) -> Vec<Node> {
    node.children
        .borrow()
        .iter()
        .filter_map(|child_node| process_node(child_node))
        .collect()
}

fn process_node(node: &Handle) -> Option<Node> {
    match &node.data {
        NodeData::Text { ref contents } => {
            let text_content = contents.borrow().to_string();
            if text_content.trim().is_empty() {
                None
            } else {
                Some(text(text_content))
            }
        }

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let tag = name.local.to_string();
            if let Some(html_tag) = match_tag(&tag) {
                let children_nodes = process_children(node);
                let attributes = extract_attributes(&attrs.borrow());
                Some(element(html_tag, attributes, children_nodes))
            } else {
                println!("tag not found: {}", tag);
                None
            }
        }
        NodeData::Document => {
            let mut children_nodes = process_children(node);
            let children_len = children_nodes.len();
            if children_len == 1 {
                Some(children_nodes.remove(0))
            } else if children_len == 2 {
                Some(children_nodes.remove(1))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_html(html: &str) -> Result<Option<Node>, ParseError> {
    let html = html.to_string().into_bytes();
    let mut cursor = Cursor::new(html);
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut cursor)?;
    let node = parse_doc(&dom.document);

    if !dom.errors.is_empty() {
        let errors: Vec<String> =
            dom.errors.iter().map(|i| i.to_string()).collect();
        Err(ParseError::Generic(errors.join(", ")))
    } else {
        Ok(node)
    }
}

fn parse_html_fragment(html: &str) -> Result<Option<Node>, ParseError> {
    let html = html.to_string().into_bytes();
    let mut cursor = Cursor::new(html);
    let dom = parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(html), local_name!("div")),
        vec![],
    )
    .from_utf8()
    .read_from(&mut cursor)?;

    let node = parse_doc(&dom.document);
    if !dom.errors.is_empty() {
        let errors: Vec<String> =
            dom.errors.iter().map(|i| i.to_string()).collect();
        Err(ParseError::Generic(errors.join(", ")))
    } else {
        Ok(node)
    }
}

pub fn convert_html_to_syntax(html: &str, use_macro: bool) -> String {
    log::info!("input: {}", html);
    match parse_html_fragment(html) {
        Ok(root_node) => {
            if let Some(root_node) = root_node {
                if let Some(root_element) = root_node.take_element() {
                    root_element.to_syntax(use_macro, 0)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        Err(e) => {
            log::error!("error: {}", e);
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        to_syntax::ToSyntax,
        *,
    };

    #[test]
    fn simpe_test() {
        let input = r#"
        <div>content1</div>
        <div>content2</div>
            "#;

        let expected = r#"html!([],[
    div!([],[text("content1")]),
    div!([],[text("content2")]),
])"#;
        let res = parse_html_fragment(input);
        let syntax = res.unwrap().unwrap().to_syntax(true, 0);
        println!("syntax: {}", syntax);
        assert_eq!(expected, syntax);
    }

    #[test]
    fn simple_html_parse() {
        let input = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Interactive sauron app</title>
    <style type="text/css">
        body {
            font-family: "Fira Sans", "Courier New";
        }
    </style>
</head>
<body style='margin: 0; padding: 0; width: 100%; height: 100%;'>
  <div id="web-app" style='width: 100%; height: 100%;'>
      #HTML_INSERTED_HERE_BY_SERVER#
  </div>
  <!-- This is a comment -->
</body>
</html>
"#;
        let expected = r#"html!([lang("en"),],[
    head!([],[
        meta!([charset("UTF-8"),],[]),
        meta!([content("width=device-width, initial-scale=1"),name("viewport"),],[]),
        title!([],[text("Interactive sauron app")]),
        style!([r#type("text/css"),],[text("
        body {
            font-family: "Fira Sans", "Courier New";
        }
    ")]),
    ]),
    body!([style("margin: 0; padding: 0; width: 100%; height: 100%;"),],[
        div!([id("web-app"),style("width: 100%; height: 100%;"),],[text("
      #HTML_INSERTED_HERE_BY_SERVER#
  ")]),
    ]),
])"#;
        let res = parse_html(input);
        let syntax = res.unwrap().unwrap().to_syntax(true, 0);
        println!("syntax: {}", syntax);
        assert_eq!(expected, syntax);
    }

    #[test]
    fn simple_svg_parse() {
        let input = r#"
<svg height="400" viewBox="0 0 600 400" width="600" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <defs>
        <filter id="shadow">
            <feDropShadow dx="2" dy="1" stdDeviation="0.2"></feDropShadow>
        </filter>
    </defs>
    <image height="400" xlink:href="data:image/jpeg;base64,/9j/4AAQSkZJRgABA" width="600" x="0" y="0"></image>
    <text fill="red" font-family="monospace" font-size="40" stroke="white" stroke-width="1" style="filter:url(#shadow);" x="65" y="55">John Smith</text>
    <text fill="white" font-family="monospace" font-size="20" style="filter:url(#shadow);" x="100" y="100">10101011</text>
    <text fill="red" font-family="monospace" font-size="50" style="filter:url(#shadow);" width="500" x="20" y="200">Happy birthday</text>
</svg>
"#;
        let expected = r#"html!([],[
    svg!([height(400),viewBox("0 0 600 400"),width(600),xmlns("http://www.w3.org/2000/svg"),],[
        defs!([],[
            filter!([id("shadow"),],[
                feDropShadow!([dx(2),dy(1),stdDeviation(0.2),],[]),
            ]),
        ]),
        image!([height(400),href("data:image/jpeg;base64,/9j/4AAQSkZJRgABA"),width(600),x(0),y(0),],[]),
        text!([fill("red"),font_family("monospace"),font_size(40),stroke("white"),stroke_width(1),style("filter:url(#shadow);"),x(65),y(55),],[text("John Smith")]),
        text!([fill("white"),font_family("monospace"),font_size(20),style("filter:url(#shadow);"),x(100),y(100),],[text("10101011")]),
        text!([fill("red"),font_family("monospace"),font_size(50),style("filter:url(#shadow);"),width(500),x(20),y(200),],[text("Happy birthday")]),
    ]),
])"#;
        let res = parse_html_fragment(input);
        let syntax = res.unwrap().unwrap().to_syntax(true, 0);
        println!("syntax: {}", syntax);
        assert_eq!(expected, syntax);
    }
}