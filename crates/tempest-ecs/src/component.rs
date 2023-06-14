pub use tempest_ecs_macros::Component;

pub trait Component: Clone + Copy + Send + Sync + 'static {
    fn id() -> usize;
}

pub trait ComponentTuple {
    const ARITY: usize;
    type Head;
    type Rest: ComponentTuple;
    const EMPTY: bool;
}

pub trait ComponentTupleGetElement<const IDX: usize> {
    type T;
}

macro_rules! indexing {
    (
        [$N:ident $($rest:tt)*] $i:expr, $($T:tt)*
    ) => (
        impl<$($T)*> ComponentTupleGetElement<{ $i }> for ($($T)*) {
            type T = $N;
        }
        
        indexing! {
            [$($rest)*] $i + 1, $($T)*
        }
    );
    
    (
        [] $($whatever:tt)*
    ) => (
        /* nothing to do here */
    );
}

macro_rules! component_tuple_arity_impl {
    (
        $N:ident, $($k:ident ,)*
    ) => (
        impl<$N: Component $(, $k: Component)*> ComponentTuple for ($N, $($k ,)*) {
            const ARITY: usize = 1 $(+ { stringify!($k); 1 })*;
            type Head = $N;
            type Rest = ($($k, )*);
            const EMPTY: bool = false;
        }
        
        indexing! { [$N $($k)*] 0, $N, $($k ,)* }
        
        component_tuple_arity_impl!($($k ,)*);
    );
    
    () => (
        impl ComponentTuple for () {
            const ARITY: usize = 0;
            type Head = ();
            type Rest = ();
            const EMPTY: bool = true;
        }
    );
}

component_tuple_arity_impl! {
    _16, _15, _14, _13, _12, _11, _10, _9, _8, _7, _6, _5, _4, _3, _2, _1,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempest_ecs_macros::Component;
    
    #[derive(Component)]
    struct TestComponent(u32);
    
    #[derive(Component)]
    struct TestComponent2(u32);
    
    #[test]
    fn test_component_id() {
        assert_ne!(TestComponent::id(), TestComponent2::id());
        assert_eq!(TestComponent::id(), TestComponent::id());
    }
}
