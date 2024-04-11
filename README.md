# amass

<!-- cargo-rdme start -->

Automatically generate `From` impls for nested enums, even across crates.  

All that is required is an attribute applied to each nesting enum:
### Example
```rust
pub mod letter {
    pub struct A;
    pub struct B;
    pub struct C;

    #[amass::amass_telety(crate::letter)]
    pub enum Letter {
        A(A),
        B(B),
        C(C),
    }
}

pub mod number {
    pub struct One;
    pub struct Two;
    pub struct Three;

    #[amass::amass_telety(crate::number)]
    pub enum Number {
        One(One),
        Two(Two),
        Three(Three),
    }
}

use letter::Letter;
use number::Number;

#[amass::amass_telety(crate)]
pub enum Alphanumeric {
    Letter(Letter),
    Number(Number),
}

pub struct Underscore;

#[amass::amass_telety(crate)]
pub enum IdentifierChar {
    Alphanumeric(Alphanumeric),
    Underscore(Underscore),
}

fn main() {
    let _: &[IdentifierChar] = &[
        letter::A.into(),     // IdentifierChar::Alphanumeric(Alphanumeric::Letter(Letter::A(A)))
        letter::B.into(),     // IdentifierChar::Alphanumeric(Alphanumeric::Letter(Letter::B(B)))
        letter::C.into(),     // IdentifierChar::Alphanumeric(Alphanumeric::Letter(Letter::C(C)))
        Underscore.into(),    // IdentifierChar::Underscore(Underscore)
        number::One.into(),   // IdentifierChar::Alphanumeric(Alphanumeric::Number(Number::One(One)))
        number::Two.into(),   // IdentifierChar::Alphanumeric(Alphanumeric::Number(Number::Two(Two)))
        number::Three.into(), // IdentifierChar::Alphanumeric(Alphanumeric::Number(Number::Three(Three)))
    ];
}
```

amass is powered by [telety](https://crates.io/crates/telety), 
which still has [limitations in the language features it supports.](https://docs.rs/telety/latest/telety/#limitations)
You can use the `#[amass]` and `#[telety(...)]` attributes separately, 
or simply use the combined `#[amass_telety(...)]` attribute.

### Specifying implementations
amass has customizable behavior for applicable fields. The following options exist:  
* ignore - No From impl will be created for the field type or the field types contained within that type.
* shallow - A From impl will be created for the field type, but not for any field types contained within that type.
* deep - A From impl will be created for the field type, and if that type is telety-enabled, 
  for the the field types contained within that type.
* force - A From impl will be created for the field type and for the the field types contained within that type. 
  If the type is not telety-enabled, a compile error will be generated.

A default action can be specified on the main attribute: `#[amass(default = force)]`. 
If no default is provided on the attribute, `deep` is the default action.  
This default can be overriden on specific variants with the `#[amass_action(...)]` helper attribute.

```rust
#[amass_telety(crate)]
pub enum ConflictingVariants {
    Default(i32),
    #[amass_action(ignore)]
    Alternate(i32),
}

```

```rust
#[amass_telety(crate, default = ignore)]
pub enum ConflictingVariants {
    #[amass_action(shallow)]
    Default(i32),
    Alternate(i32),
}

```

Note that is not currently possible to override behavior on an upstream enum.  
In this example, two impls for `From<DiamondTop> for DiamondBottom` are generated, causing a compile error.  
```rust,compile_fail,E0119
# use amass::amass_telety;

pub struct DiamondTop;

#[amass_telety(crate)]
pub struct DiamondLeft {
    Top(DiamondTop)
}

#[amass_telety(crate)]
pub struct DiamondRight {
    Top(DiamondTop)
}

#[amass_telety(crate)]
pub struct DiamondBottom {
    Left(DiamondLeft),
    Right(DiamondRight),
}
# fn main() { }
```
To solve this, either `DiamondLeft` or `DiamondRight` must `ignore` the `Top` variant, 
or `DiamondBottom` must use `shallow` for the `Left` or `Right` variant. It is not possible to *only* skip
the `DiamondTop` -> `DiamondLeft`/`DiamondRight` -> `DiamondBottom` impl.

<!-- cargo-rdme end -->
