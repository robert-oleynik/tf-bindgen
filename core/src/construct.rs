use std::rc::Rc;

use crate::app::App;

pub trait Construct {
    /// Returns the app this construct is assigned to.
    fn app(&self) -> Rc<App>;

    /// Returns the name of the root stack.
    fn stack(&self) -> &str;

    /// Returns the construct's name.
    fn name(&self) -> &str;

    /// Returns the construct's path.
    fn path(&self) -> String;
}
