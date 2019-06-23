use std::{
    collections::BTreeMap,
    fmt,
};

pub mod builder;
pub mod event;
mod value;

use crate::Callback;
pub use event::Event;
pub use value::Value;

/// This is the core data structure of the library.
/// Any tree can be represented by `Node`.
/// The `T` is generic instead of just using plain `&'static str`
/// in order for this library to be used not only in html based widget
/// but can also be used to represent native GUI widgets
/// in various platforms.
///
/// Note: Clone is necessary for the aesthetics in the construction of node through series of function
/// calls.
/// Without Clone, the user code would look like these:
/// ```ignore
///     div(&[class("some-class"), &[text("Some text")])
/// ```
/// as compared to
/// ```ignore
///     div([class("some-class"), [text("some text)])
/// ```
/// Cloning is only done once, and happens when constructing the views into a node tree.
/// Cloning also allows flexibility such as adding more children into an existing node/element.
#[derive(Debug, PartialEq, Clone)]
pub enum Node<T, EVENT, MSG>
where
    MSG: Clone + 'static,
    EVENT: 'static,
{
    Element(Element<T, EVENT, MSG>),
    Text(Text),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Element<T, EVENT, MSG>
where
    MSG: Clone + 'static,
    EVENT: 'static,
{
    pub tag: T,
    pub attrs: Vec<Attribute<EVENT, MSG>>,
    pub children: Vec<Node<T, EVENT, MSG>>,
    pub namespace: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute<EVENT, MSG>
where
    MSG: Clone,
{
    pub name: &'static str,
    pub value: AttribValue<EVENT, MSG>,
}

impl<EVENT, MSG> Attribute<EVENT, MSG>
where
    MSG: Clone + 'static,
    EVENT: 'static,
{
    pub fn new(name: &'static str, value: AttribValue<EVENT, MSG>) -> Self {
        Attribute { name, value }
    }

    pub fn map<F, MSG2>(self, func: F) -> Attribute<EVENT, MSG2>
    where
        F: Fn(MSG) -> MSG2 + 'static + Clone,
        MSG2: Clone + 'static,
    {
        Attribute::new(self.name, self.value.map(func.clone()))
    }

    fn is_event(&self) -> bool {
        self.value.is_event()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttribValue<EVENT, MSG>
where
    MSG: Clone,
{
    Value(Value),
    Callback(Callback<EVENT, MSG>),
}

impl<EVENT, MSG> AttribValue<EVENT, MSG>
where
    MSG: Clone + 'static,
    EVENT: 'static,
{
    pub fn map<F, MSG2>(self, func: F) -> AttribValue<EVENT, MSG2>
    where
        F: Fn(MSG) -> MSG2 + 'static + Clone,
        MSG2: Clone,
    {
        match self {
            AttribValue::Value(value) => AttribValue::Value(value),
            AttribValue::Callback(cb) => {
                AttribValue::Callback(cb.map(func.clone()))
            }
        }
    }

    fn is_event(&self) -> bool {
        match self {
            AttribValue::Value(_) => false,
            AttribValue::Callback(_) => true,
        }
    }

    pub fn get_callback(&self) -> Option<&Callback<EVENT, MSG>> {
        match self {
            AttribValue::Value(_) => None,
            AttribValue::Callback(cb) => Some(cb),
        }
    }
}

impl<EVENT, MSG> fmt::Display for AttribValue<EVENT, MSG>
where
    MSG: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AttribValue::Value(value) => write!(f, "{}", value),
            AttribValue::Callback(cb) => write!(f, "{:?}", cb),
        }
    }
}

impl<EVENT, MSG> From<Callback<EVENT, MSG>> for AttribValue<EVENT, MSG>
where
    MSG: Clone,
{
    fn from(cb: Callback<EVENT, MSG>) -> Self {
        AttribValue::Callback(cb)
    }
}

impl<T, EVENT, MSG> Node<T, EVENT, MSG>
where
    EVENT: Clone + 'static,
    MSG: Clone + 'static,
    T: Clone,
{
    /// map the return of the callback from MSG to MSG2
    pub fn map<F, MSG2>(self, func: F) -> Node<T, EVENT, MSG2>
    where
        F: Fn(MSG) -> MSG2 + 'static + Clone,
        MSG2: Clone + 'static,
    {
        match self {
            Node::Element(element) => Node::Element(element.map(func)),
            Node::Text(text) => Node::Text(Text::new(text.text)),
        }
    }

    fn to_pretty_string(&self, indent: i32) -> String
    where
        T: ToString,
    {
        match self {
            Node::Element(element) => element.to_pretty_string(indent),
            Node::Text(text) => format!("{}", text),
        }
    }

    fn is_text_node(&self) -> bool {
        match self {
            Node::Element(_) => false,
            Node::Text(_) => true,
        }
    }

    pub fn as_element(&mut self) -> Option<&mut Element<T, EVENT, MSG>> {
        match *self {
            Node::Element(ref mut element) => Some(element),
            Node::Text(_) => None,
        }
    }

    pub fn as_element_ref(&mut self) -> Option<&Element<T, EVENT, MSG>> {
        match *self {
            Node::Element(ref element) => Some(element),
            Node::Text(_) => None,
        }
    }

    /// Append children to this element
    pub fn children(mut self, children: Vec<Node<T, EVENT, MSG>>) -> Self {
        if let Some(element) = self.as_element() {
            element.add_children(children);
        }
        self
    }

    /// add attributes to the node
    pub fn attributes(
        mut self,
        attributes: Vec<Attribute<EVENT, MSG>>,
    ) -> Self {
        if let Some(elm) = self.as_element() {
            elm.add_attributes(attributes);
        }
        self
    }
}

impl<T, EVENT, MSG> Element<T, EVENT, MSG>
where
    EVENT: Clone + 'static,
    MSG: Clone + 'static,
    T: Clone,
{
    /// map the return of the callback from MSG to MSG2
    pub fn map<F, MSG2>(self, func: F) -> Element<T, EVENT, MSG2>
    where
        F: Fn(MSG) -> MSG2 + 'static + Clone,
        MSG2: Clone + 'static,
    {
        Element {
            tag: self.tag,
            attrs: self
                .attrs
                .into_iter()
                .map(|attr| attr.map(func.clone()))
                .collect(),
            namespace: self.namespace,
            children: self
                .children
                .into_iter()
                .map(|child| child.map(func.clone()))
                .collect(),
        }
    }

    /// check if the children of this node is only 1 and it is a text node
    fn is_children_a_node_text(&self) -> bool {
        self.children.len() == 1 && self.children[0].is_text_node()
    }

    /// make a pretty string representation of this node
    fn to_pretty_string(&self, indent: i32) -> String
    where
        T: Clone + ToString,
    {
        let mut buffer = String::new();
        buffer += &format!("<{}", self.tag.to_string());

        for attr in self.attrs.iter() {
            buffer += &format!(r#" {}="{}""#, attr.name, attr.value);
        }
        buffer += ">";

        // do not indent if it is only text child node
        if self.is_children_a_node_text() {
            buffer += &self.children[0].to_pretty_string(indent);
        } else {
            // otherwise print all child nodes with each line and indented
            for child in self.children.iter() {
                buffer += &format!(
                    "\n{}{}",
                    padd(indent + 1),
                    child.to_pretty_string(indent + 1)
                );
            }
        }
        // do not make a new line it if is only a text child node or it has no child nodes
        if !(self.is_children_a_node_text() || self.children.is_empty()) {
            buffer += &format!("\n{}", padd(indent));
        }
        buffer += &format!("</{}>", self.tag.to_string());
        buffer
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Text {
    pub text: String,
}

impl<T, EVENT, MSG> Element<T, EVENT, MSG>
where
    T: Clone,
    MSG: Clone + 'static,
    EVENT: Clone + 'static,
{

    pub fn with_tag(tag: T) -> Self {
        Element{
            tag,
            attrs: vec![],
            children: vec![],
            namespace: None,
        }
    }
    pub fn get_attr(&self, key: &str) -> Option<&Attribute<EVENT, MSG>> {
        self.attrs.iter().find_map(|ref att| {
            if att.name == key {
                Some(*att)
            } else {
                None
            }
        })
    }

    /// get the attributes that are events
    pub fn events(&self) -> Vec<&Attribute<EVENT, MSG>> {
        self.attrs.iter().filter(|attr| attr.is_event()).collect()
    }

    pub fn get_event(&self, name: &str) -> Option<&Attribute<EVENT, MSG>> {
        self.events()
            .iter()
            .find(|event| event.name == name)
            .map(|event| *event)
    }

    pub fn attributes(&self) -> Vec<&Attribute<EVENT, MSG>> {
        self.attrs.iter().filter(|attr| !attr.is_event()).collect()
    }

    pub fn get_attrib_value(
        &self,
        key: &str,
    ) -> Option<&AttribValue<EVENT, MSG>> {
        self.attributes().iter().find_map(|ref att| {
            if att.name == key {
                Some(&att.value)
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn add_attributes(&mut self, attrs: Vec<Attribute<EVENT, MSG>>) {
        self.attrs.extend(attrs);
    }

    #[inline]
    pub fn add_children(&mut self, children: Vec<Node<T, EVENT, MSG>>) {
        self.children.extend(children);
    }

    #[inline]
    pub fn add_event_listener(
        &mut self,
        event: &'static str,
        cb: Callback<EVENT, MSG>,
    ) {
        let attr_event = Attribute::new(event, cb.into());
        self.attrs.push(attr_event);
    }
}

impl Text {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Text { text: s.into() }
    }
}

// Turn a Text into an HTML string
impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<T, EVENT, MSG> fmt::Display for Node<T, EVENT, MSG>
where
    T: Clone + ToString,
    EVENT: Clone + 'static,
    MSG: Clone + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_pretty_string(0))
    }
}

impl<T, EVENT, MSG> From<Element<T, EVENT, MSG>> for Node<T, EVENT, MSG>
where
    MSG: Clone,
{
    fn from(v: Element<T, EVENT, MSG>) -> Self {
        Node::Element(v)
    }
}

/// make a blank string with indented padd
fn padd(n: i32) -> String {
    let mut buffer = String::new();
    for _ in 0..n {
        buffer += "    ";
    }
    buffer
}
