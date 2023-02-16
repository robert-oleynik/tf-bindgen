use std::rc::Rc;

use crate::app::App;
use crate::construct::Construct;

/// Top-level construct. Used as an abstract layer between resources/constructs and apps.
pub struct Stack {
    app: Rc<App>,
    name: String,
}

impl Stack {
    /// Create a new infrastructure stack.
    ///
    /// # Parameters
    ///
    /// - `app` App to deploy stack in.
    /// - `name` Name of the stack.
    pub fn new(app: Rc<App>, name: String) -> Rc<Self> {
        Rc::new(Self { app, name })
    }
}

impl Construct for Stack {
    fn app(&self) -> Rc<App> {
        self.app.clone()
    }

    fn stack(&self) -> &str {
        &self.name
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &str {
        &self.name
    }
}
