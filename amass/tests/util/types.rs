#![allow(dead_code)]

use amass::amass_telety;
use telety::telety;

#[amass_telety(crate::util::types)]
#[derive(Debug)]
pub(crate) enum Beer {
    #[amass_action(force)]
    Lager(Lager),
    Ale(Ale),
}

#[amass_telety(crate::util::types)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum Lager {
    Pilsner(Pilsner),
    Bock(Bock),
}

#[telety(crate::util::types)]
#[derive(Debug)]
pub(crate) struct Pilsner;

#[telety(crate::util::types)]
#[derive(Debug)]
pub(crate) struct Bock;

#[amass_telety(crate::util::types)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum Ale {
    IPA(IPA),
    Stout(Stout),
    Wheat(Wheat),
}

#[amass_telety(crate::util::types)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum IPA {
    WestCoast,
    NewEngland,
    Imperial,
}

#[telety(crate::util::types)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum Stout {
    Irish,
    RussianImperial,
    Oatmeal,
}

#[telety(crate::util::types)]
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum Wheat {
    Hefeweizen,
}

#[amass_telety(crate::util::types)]
#[derive(Debug)]
pub(crate) enum Generic1<T> {
    // G1(Box<Self>),
    G2(Generic2<T, T>),
}

#[amass_telety(crate::util::types)]
#[derive(Debug)]
pub(crate) enum Generic2<T, U> {
    #[amass_action(deep)]
    T((T,)),
    #[amass_action(deep)]
    TU((T, U)),
}

#[amass::amass_telety(crate::util::types)]
#[allow(dead_code)]
enum Private {
    A(i32),
}
