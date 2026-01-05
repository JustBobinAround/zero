/// Allows a tupple to be used as a functions arguments
pub trait TupleApply<F> {
    type Output;
    fn apply(self, f: F) -> Self::Output;
}

macro_rules! impl_tuple_apply {
    ( $( $T:ident ),+ ) => {
        impl<Z, Y, $( $T ),+> TupleApply<Z> for ( $( $T ),+ )
        where
            Z: FnOnce( $( $T ),+ ) -> Y,
        {
            type Output = Y;

            fn apply(self, f: Z) -> Self::Output {
                #[allow(non_snake_case)]
                let ( $( $T ),+ ) = self;
                f( $( $T ),+ )
            }
        }
    };
}

// If only there were actual variadics :(
impl_tuple_apply!(A, B);
impl_tuple_apply!(A, B, C);
impl_tuple_apply!(A, B, C, D);
impl_tuple_apply!(A, B, C, D, E);
impl_tuple_apply!(A, B, C, D, E, F);
impl_tuple_apply!(A, B, C, D, E, F, G);

#[cfg(test)]
mod tests {
    use crate::variadics::TupleApply;

    #[test]
    fn test_variadics() {
        assert_eq!(3, (1, 2).apply(|a, b| { a + b }));
        assert_eq!(6, (1, 2, 3).apply(|a, b, c| { a + b + c }));
        assert_eq!(10, (1, 2, 3, 4).apply(|a, b, c, d| { a + b + c + d }));
        assert_eq!(
            15,
            (1, 2, 3, 4, 5).apply(|a, b, c, d, e| { a + b + c + d + e })
        );
        assert_eq!(
            21,
            (1, 2, 3, 4, 5, 6).apply(|a, b, c, d, e, f| { a + b + c + d + e + f })
        );
        assert_eq!(
            28,
            (1, 2, 3, 4, 5, 6, 7).apply(|a, b, c, d, e, f, g| { a + b + c + d + e + f + g })
        );
    }
}
