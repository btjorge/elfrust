#![feature(macro_metavar_expr_concat)]

#[macro_export] // struct ???
macro_rules! ranged_enum {
    (
        $vis:vis struct $id:ident : $ty:ty {
            $( const $c:ident = $i:literal; )*

            $( range $cr:ident = $ir1:literal ..= $ir2:literal; )*
        }
    ) => {
        #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::cmp::PartialEq, ::std::cmp::PartialOrd, ::bytemuck::AnyBitPattern, ::bytemuck::NoUninit)]
        #[repr(transparent)]
        $vis struct $id($ty);

        impl $id {
            $( const $c : Self = $id ( $i ); )*

            $( const ${concat(LO, $cr)} : Self = $id ( $ir1 ); const ${concat(HI, $cr)} : Self = $id ( $ir2 );)*
        }

        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self.0 {
                    $( $i => ::std::write!(f, "{}", ::std::stringify!($c)), )*

                    $( $ir1 ..= $ir2 => ::std::write!(f, "{}({})", ::std::stringify!($cr), self.0), )*

                    _ => ::std::panic!("unsupported: {}", self.0),
                }
            }
        }

        impl ::std::fmt::Debug for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(self, f)
            }
        }
    }
}
