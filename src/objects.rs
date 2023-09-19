use std::{any::Any, fmt, mem};
use crate::gc::{GcTrace, Gc};


impl GcTrace for String {
    fn format(&self, f: &mut fmt::Formatter, _gc: &Gc) -> fmt::Result {
        write!(f, "{}", self)
    }
    fn size(&self) -> usize {
        mem::size_of::<String>() + self.as_bytes().len()
    }
    fn trace(&self, _gc: &mut Gc) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
