use strum::{
    Display, EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoEnumIterator as _,
    IntoStaticStr, VariantNames,
};

use serde::Serialize;

#[derive(
    Debug, EnumString, EnumCount, EnumDiscriminants, EnumIs, EnumIter, IntoStaticStr, VariantNames,
)]
#[allow(unused)]
enum MyEnum {
    A,
    B(String),
    C,
}

#[allow(unused)]
#[derive(Display, Debug, Serialize)]
enum Color {
    #[strum(serialize = "redred")]
    Red,
    Green {
        range: usize,
    },
    Blue(usize),
    Yellow,
    #[strum(to_string = "purple with {sat} saturation")]
    Purple {
        sat: usize,
    },
}

fn main() -> anyhow::Result<()> {
    println!("{:?}", MyEnum::VARIANTS);
    MyEnum::iter().for_each(|v| println!("{:?}", v));

    let my_enum = MyEnum::B("hello".to_string());
    println!("{:?}", my_enum.is_b());
    println!("total number of variants: {}", MyEnum::COUNT);

    let s: &'static str = my_enum.into();
    println!("{}", s);

    let red = Color::Red;
    let green = Color::Green { range: 10 };
    let blue = Color::Blue(20);
    let yellow = Color::Yellow;
    let purple = Color::Purple { sat: 30 };
    println!(
        "red:{}, green:{}, blue:{}, yellow:{}, purple:{}",
        red, green, blue, yellow, purple
    );

    let red_str = serde_json::to_string(&red)?;
    println!("{}", red_str);
    Ok(())
}
