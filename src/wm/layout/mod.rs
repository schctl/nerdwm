pub trait Layout {
    fn configure(&self, clients: &Vec<xcb::Window>);
}

/// A layout that does nothing.
pub struct BlankLayout {}

impl Layout for BlankLayout {
    fn configure(&self, _: &Vec<xcb::Window>) {}
}
