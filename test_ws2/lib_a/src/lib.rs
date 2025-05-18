pub fn core_func<T>() {}
pub mod submod {
    pub fn sub_func() {}
}
pub use core_func as alias_func;
pub use submod::sub_func as sub_alias;
