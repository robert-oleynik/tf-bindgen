use std::rc::Rc;

#[derive(::tf_bindgen::codegen::Construct)]
pub struct Custom {
    #[scope]
    __m_scope: Rc<dyn ::tf_bindgen::Construct>,
    #[id]
    __m_name: String,
}

fn main() {}
