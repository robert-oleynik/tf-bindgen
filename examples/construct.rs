use std::rc::Rc;

#[derive(::tf_bindgen::codegen::Construct)]
pub struct Custom {
    #[construct(scope)]
    __m_scope: Rc<dyn ::tf_bindgen::Scope>,
    #[construct(id)]
    __m_name: String,
}

fn main() {}
