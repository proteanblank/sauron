use crate::dom::{
    Application, Cmd, Component, Container, Effects, MountAction, MountTarget, Program, Task,
};
use crate::vdom::Node;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[cfg(feature = "use-snippets")]
#[wasm_bindgen(module = "/js/define_custom_element.js")]
extern "C" {
    // register using custom element define
    // # Example:
    // ```rust,ignore
    //  sauron::register_custom_element("date-time", "DateTimeWidgetCustomElement");
    // ```
    pub fn register_custom_element(custom_tag: &str, adapter: &str);
}

#[cfg(not(feature = "use-snippets"))]
thread_local!(static REGISTER_CUSTOM_ELEMENT_FUNCTION: js_sys::Function = create_register_custom_element_function());

/// register using custom element define
/// # Example:
/// ```rust,ignore
///  sauron::register_custom_element("date-time", "DateTimeWidgetCustomElement");
/// ```
#[cfg(not(feature = "use-snippets"))]
pub fn register_custom_element(custom_tag: &str, adapter: &str) {
    REGISTER_CUSTOM_ELEMENT_FUNCTION.with(|func| {
        func.call2(
            &JsValue::NULL,
            &JsValue::from_str(custom_tag),
            &JsValue::from_str(adapter),
        )
        .expect("must call");
    })
}

/// TODO: refer to https://github.com/gbj/custom-elements
/// for improvements
/// dynamically create the function which will register the custom tag
#[cfg(not(feature = "use-snippets"))]
fn create_register_custom_element_function() -> js_sys::Function {
    js_sys::Function::new_with_args(
        "custom_tag, adapterClassName",
        r#"
    function define_custom_element(custom_tag, adapterClassName)
    {
         console.log(`custom tag: ${custom_tag}, adapterClassName: ${adapterClassName}`);
         let adapter = window[adapterClassName];
         console.log("adapter: ", adapter);
          if (window.customElements.get(custom_tag) === undefined ){
            window.customElements.define(custom_tag,
                class extends HTMLElement{
                    constructor(){
                        super();
                        this.instance = new adapter(this);
                    }

                    static get observedAttributes(){
                        return adapter.observedAttributes;
                    }

                    connectedCallback(){
                        this.instance.connectedCallback();
                    }
                    disconnectedCallback(){
                        this.instance.disconnectedCallback();
                    }
                    adoptedCallback(){
                        this.instance.adoptedCallback();
                    }
                    attributeChangedCallback(name, oldValue, newValue){
                        this.instance.attributeChangedCallback(name, oldValue, newValue);
                    }

                    appendChild(child){
                        console.log("appending a child:", child);
                        this.instance.appendChild(child);
                    }

                }
            );
        }
    }
    define_custom_element(custom_tag, adapterClassName);
"#,
    )
}

/// a trait for implementing CustomElement in the DOM with custom tag
pub trait CustomElement<MSG> {
    /// returns the attributes that is observed by this component
    /// These are the names of the attributes the component is interested in
    fn observed_attributes() -> Vec<&'static str>;

    /// This will be invoked when a component is used as a custom element
    /// and the attributes of the custom-element has been modified
    ///
    /// if the listed attributes in the observed attributes are modified
    fn attribute_changed(
        program: &Program<Self, MSG>,
        attr_name: &str,
        old_value: Option<String>,
        new_value: Option<String>,
    ) where
        Self: Sized + Application<MSG>;

    /// the component is attached to the dom
    fn connected_callback(&mut self);
    /// the component is removed from the DOM
    fn disconnected_callback(&mut self);

    /// the component is moved or attached to the dom
    fn adopted_callback(&mut self);
}

/// A self contain web component
/// This is needed to move some of the code from the #web_component macro
/// This is also necessary, since #[wasm_bindgen] macro can not process impl types which uses
/// generics, we use generics here to simplify the code and do the type checks for us, rather than
/// in the code derived from the #[web_component] macro
pub struct WebComponent<APP, MSG>
where
    MSG: 'static,
{
    /// the underlying program running this web component
    pub program: Program<APP, MSG>,
}

/// Auto implementation of Application trait for Component that
/// has no external MSG
/// but only if that Component is intended to be a CustomElement
impl<COMP, MSG> Application<MSG> for COMP
where
    COMP: Component<MSG, ()> + 'static,
    COMP: CustomElement<MSG>,
    MSG: 'static,
{
    fn init(&mut self) -> Vec<Cmd<Self, MSG>> {
        <Self as Component<MSG, ()>>::init(self)
            .into_iter()
            .map(Cmd::from)
            .collect()
    }

    fn update(&mut self, msg: MSG) -> Cmd<Self, MSG> {
        let effects = <Self as Component<MSG, ()>>::update(self, msg);
        Cmd::from(effects)
    }

    fn view(&self) -> Node<MSG> {
        <Self as Component<MSG, ()>>::view(self)
    }

    fn stylesheet() -> Vec<String> {
        <Self as Component<MSG, ()>>::stylesheet()
    }

    fn style(&self) -> Vec<String> {
        <Self as Component<MSG, ()>>::style(self)
    }
}

/// Auto implementation of Component trait for Container,
/// which in turn creates an Auto implementation trait for of Application for Container
/// but only if that Container is intended to be a CustomElement
impl<CONT, MSG> Component<MSG, ()> for CONT
where
    CONT: Container<MSG, ()>,
    CONT: CustomElement<MSG>,
    MSG: 'static,
{
    fn init(&mut self) -> Vec<Task<MSG>> {
        <Self as Container<MSG, ()>>::init(self)
    }

    fn update(&mut self, msg: MSG) -> Effects<MSG, ()> {
        <Self as Container<MSG, ()>>::update(self, msg)
    }

    fn view(&self) -> Node<MSG> {
        // converting the component to container loses ability
        // for the container to contain children components
        <Self as Container<MSG, ()>>::view(self, [])
    }

    fn stylesheet() -> Vec<String> {
        <Self as Container<MSG, ()>>::stylesheet()
    }

    fn style(&self) -> Vec<String> {
        <Self as Container<MSG, ()>>::style(self)
    }
}

impl<APP, MSG> WebComponent<APP, MSG>
where
    APP: Application<MSG> + Default + 'static,
    APP: CustomElement<MSG>,
    MSG: 'static,
{
    /// create a new web component, with the node as the target element to be mounted into
    pub fn new(node: JsValue) -> Self {
        let mount_node: &web_sys::Node = node.unchecked_ref();
        Self {
            program: Program::new(
                APP::default(),
                mount_node,
                MountAction::Append,
                MountTarget::ShadowRoot,
            ),
        }
    }

    /// When the attribute of the component is changed, this method will be called
    pub fn attribute_changed(&self, attr_name: &str, old_value: JsValue, new_value: JsValue) {
        let old_value = old_value.as_string();
        let new_value = new_value.as_string();
        APP::attribute_changed(&self.program, attr_name, old_value, new_value);
    }

    /// called when the web component is mounted
    pub fn connected_callback(&mut self) {
        self.program.mount();
        let static_style = <APP as Application<MSG>>::stylesheet().join("");
        self.program.inject_style_to_mount(&static_style);
        let dynamic_style = <APP as Application<MSG>>::style(&self.program.app.borrow()).join("");
        self.program.inject_style_to_mount(&dynamic_style);
        self.program.app.borrow_mut().connected_callback();
        self.program.update_dom().expect("must update dom");
    }

    /// called when the web component is removed
    pub fn disconnected_callback(&mut self) {
        self.program.app.borrow_mut().disconnected_callback();
    }

    /// called when web componented is moved into other parts of the document
    pub fn adopted_callback(&mut self) {
        self.program.app.borrow_mut().adopted_callback();
    }
}
