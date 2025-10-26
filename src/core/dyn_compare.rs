use std::any::Any;

pub trait DynCompare: Any {
    fn as_any(&self) -> &dyn Any;
    fn dyn_eq(&self, other: &dyn DynCompare) -> bool;
}

impl<T: Any + PartialEq> DynCompare for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn DynCompare) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self == o)
    }
}

impl PartialEq<dyn DynCompare> for dyn DynCompare {
    fn eq(&self, other: &dyn DynCompare) -> bool {
        self.dyn_eq(other)
    }
}
