mod util;

use util::types::*;

#[test]
fn test() {
    let beers: &[Beer] = &[
        IPA::WestCoast.into(),
        IPA::NewEngland.into(),
        IPA::Imperial.into(),
        Ale::IPA(IPA::WestCoast).into(),
        Stout::Irish.into(),
        Wheat::Hefeweizen.into(),
        Pilsner.into(),
    ];

    for beer in beers {
        println!("{beer:?}");
    }
}
