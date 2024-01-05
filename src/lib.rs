#![no_std]

/// Type level combinator for bitflags.
#[ghost::phantom]
#[derive(Debug, Default)]
pub struct Or<A, B>;

impl<A: SetMember + Default, B: SetMember<Set = A::Set> + Default> Or<A, B>  {
    pub fn contains(&self, other: impl SetMember<Set=Self>) -> bool {
        other.in_set(self)
    }

    pub fn equals(&self, other: impl SetMember<Set=Self>) -> bool {
        other.eq_set(self)
    }
}

impl<A: SetMember + Default, B: SetMember<Set = A::Set> + Default> SetMember for Or<A, B>  {
    type Set = A::Set;

    fn to_set(&self) -> Self::Set {
        A::default_set() | B::default_set()
    }

    fn eq_set(&self, set: &Self::Set) -> bool {
        &Self::default_set() == set
    }

    fn in_set(&self, set: &Self::Set) -> bool {
        A::default().in_set(set) || B::default().in_set(set)
    }
}

/// Member of a set of flags.
pub trait SetMember: Sized{
    type Set: PartialEq + core::ops::BitOr<Self::Set, Output = Self::Set>;
    fn to_set(&self) -> Self::Set;
    fn eq_set(&self, set: &Self::Set) -> bool;
    fn in_set(&self, set: &Self::Set) -> bool;
    fn and_set(self, other: impl SetMember<Set = Self::Set>) -> Self::Set {
        self.to_set() | other.to_set()
    }
    fn default_set() -> Self::Set where Self: Default {
        Self::to_set(&Default::default())
    }
}

/// Type level bitflags.
/// 
/// # Example
/// 
/// ```
/// # use tlbf::*;
/// tlbf!(
///     pub Color: u64 {
///         Red,
///         Green, 
///         Blue,
///     }
/// );
/// assert!(Color::Red.contains(Red));
/// assert!(!Color::Red.contains(Green));
/// assert!(Red|Green == Color::Red|Color::Green);
/// assert!((Red|Green).contains(Red));
/// assert!((Red|Green).contains(Green));
/// assert!(!(Red|Green).contains(Blue));
/// ```
#[macro_export]
macro_rules! tlbf {
    (
        $(#[$($flags_args: tt)*])*
        $vis: vis $flags_name: ident: $repr: ty {
            $(
                $(#[$($branch_args: tt)*])*
                $vis2: vis $name: ident
            ),* $(,)?
        }
    ) => {
        $crate::tlbf! (
            $(#[$($flags_args)*])*
            $vis $flags_name: $repr {
                $(
                    $(#[$($branch_args)*])*
                    $vis2 $name
                ),*
            }
            {} (0)
        );
    };
    (
        $(#[$($flags_args: tt)*])*
        $vis: vis $flags_name: ident: $repr: ty {
            $(#[$($first_args: tt)*])*
            $vis0: vis $first: ident
            $(  
                ,$(#[$($branch_args: tt)*])*
                $vis2: vis $name: ident
            )* $(,)?
        }
        {$($(#[$($a: tt)*])* $v: vis $x: ident = $y: expr),*} ($value: expr)
    ) => {
        $crate::tlbf! (
            $(#[$($flags_args)*])*
            $vis $flags_name: $repr {
                $(
                    $(#[$($branch_args)*])*
                    $vis2 $name
                ),*
            }
            {
                $($(#[$($a)*])* $v $x = $y,)* 
                $(#[$($first_args)*])*
                $vis0 $first = $value
            } ($value + 1)
        );
    };
    (
        $(#[$($flags_args: tt)*])*
        $vis: vis $flags_name: ident: $repr: ty {$(,)?}
        {$($(#[$($a: tt)*])* $vis2: vis $x: ident = $y: expr),*} ($value: expr)
    ) => {
        $crate::tlbf! (
            $(#[$($flags_args)*])*
            $vis $flags_name: $repr
            {$($vis2 $x = $y),*}
        );
    };
    (
        $(#[$($flags_args: tt)*])*
        $vis: vis $flags_name: ident: $repr: ty {
            $(
                $(#[$($branch_args: tt)*])*
                $vis2: vis $name: ident = $value: expr
            ),* $(,)?
        }
    ) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $(#[$($flags_args)*])*
        $vis struct $flags_name($repr);

        const _: () = {
            #[allow(non_upper_case_globals)]
            impl $flags_name {
                $($vis const $name: Self = Self(1 << ($value));)*

                pub fn is_empty(&self) -> bool {
                    self.0 == 0
                }

                pub fn contains(&self, other: impl $crate::SetMember<Set=Self>) -> bool {
                    other.in_set(self)
                }

                pub fn equals(&self, other: impl $crate::SetMember<Set=Self>) -> bool {
                    other.eq_set(self)
                }

                pub fn intersects(&self, other: impl $crate::SetMember<Set=Self>) -> bool {
                    self.0 & other.to_set().0 > 0
                }

                pub fn all() -> Self {
                    $(Self::$name)|*
                }
            }

            impl $crate::SetMember for $flags_name {
                type Set = $flags_name;
                fn to_set(&self) -> Self::Set {
                    (*self).into()
                }
                fn eq_set(&self, set: &Self::Set) -> bool {
                    self == set
                }
                fn in_set(&self, set: &Self::Set) -> bool {
                    set.0 & self.0 == self.0
                }
            }

            impl<T> ::core::ops::BitAnd<T> for $flags_name where T: $crate::SetMember<Set = Self> {
                type Output = Self;
                fn bitand(self, rhs: T) -> Self {
                    Self(self.0 & rhs.to_set().0)
                }
            }

            impl<T> ::core::ops::BitOr<T> for $flags_name where T: $crate::SetMember<Set = Self> {
                type Output = Self;
                fn bitor(self, rhs: T) -> Self {
                    Self(self.0 | rhs.to_set().0)
                }
            }

            impl<T> ::core::ops::BitXor<T> for $flags_name where T: $crate::SetMember<Set = Self> {
                type Output = Self;
                fn bitxor(self, rhs: T) -> Self {
                    Self(self.0 ^ rhs.to_set().0)
                }
            }

            impl<T> ::core::ops::BitAndAssign<T> for $flags_name where T: $crate::SetMember<Set = Self> {
                fn bitand_assign(&mut self, rhs: T) {
                    self.0 &= rhs.to_set().0
                }
            }

            impl<T> ::core::ops::BitOrAssign<T> for $flags_name where T: $crate::SetMember<Set = Self> {
                fn bitor_assign(&mut self, rhs: T) {
                    self.0 |= rhs.to_set().0
                }
            }

            impl<T> ::core::ops::BitXorAssign<T> for $flags_name where T: $crate::SetMember<Set = Self> {
                fn bitxor_assign(&mut self, rhs: T) {
                    self.0 ^= rhs.to_set().0
                }
            }
        };


        $(
            $(#[$($branch_args)*])*
            #[derive(Debug, Default, Clone, Copy, Eq, Hash)]
            $vis2 struct $name;

            const _: () = {
                use $crate::SetMember;
                impl ::core::fmt::Display for $name {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        f.write_str(stringify!($name))
                    }
                }
    
                impl ::core::convert::From<$name> for $flags_name {
                    fn from(_: $name) -> Self {
                        Self::$name
                    }
                }

                impl ::core::convert::From<&$name> for $flags_name {
                    fn from(_: &$name) -> Self {
                        Self::$name
                    }
                }
    
                impl $crate::SetMember for $name {
                    type Set = $flags_name;
                    fn to_set(&self) -> Self::Set {
                        self.into()
                    }
                    fn eq_set(&self, set: &Self::Set) -> bool {
                        set == &Self::Set::$name
                    }
                    fn in_set(&self, set: &Self::Set) -> bool {
                        *set & Self::Set::$name == Self::Set::$name
                    }
                }
    
                impl<T> ::core::ops::BitOr<T> for $name where T: SetMember<Set=$flags_name>{
                    type Output = $flags_name;
                    fn bitor(self, rhs: T) -> $flags_name {
                        $flags_name::$name | rhs.to_set()
                    }
                }

                impl<T> ::core::cmp::PartialEq<T> for $name where T: $crate::SetMember<Set=$flags_name>{
                    fn eq(&self, other: &T) -> bool {
                        $flags_name::$name == other.to_set()
                    }
                }
            };
        )*
    };
}


/// Join bitflags at the type level.
/// 
/// ```
/// # use tlbf::*;
/// # tlbf!(
/// #     pub Color: u64 {
/// #         pub Red, Green, Blue,
/// #     }
/// # );
/// let flags = tyflags!(Red|Blue);
/// assert!(flags.contains(Color::Red));
/// assert!(flags.contains(Color::Blue));
/// assert!(!flags.contains(Color::Green));
/// ```
#[macro_export]
macro_rules! tyflags {
    ($expr: expr $(,)?) => {
        $expr
    };
    ($first: expr $(,$expr: expr)* $(,)?) => {
        $crate::Or<$first, $crate::type_join!($($expr),*)>
    };
}

#[cfg(test)]
mod test {

    tlbf!(
        pub Unit1: u8 {
            Hello
        }
    );
    tlbf!(
        pub Unit2: u8 {
            Hiii,
        }
    );
    #[test]
    pub fn test(){
        tlbf!(
            pub Mascot: u8 {
                pub Ferris
            }
        );
        tlbf!(
            #[derive(Default)]
            LesserMascots: u8 {
                #[repr(C)]
                Gopher
            }
        );
        assert_eq!(Mascot::all(), Mascot::Ferris);
    }
}