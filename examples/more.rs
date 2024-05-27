use derive_more::{Add, Display, From, Into};

#[derive(PartialEq, From, Add, Display, Clone, Copy, Into)]
struct MyInt(i32);

#[derive(PartialEq, From, Into)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(PartialEq, From, Add, Display)]
enum MyEnum {
    #[display(fmt = "int: {_0}")]
    Int(i32),
    Uint(u32),
    #[display(fmt = "nothing")]
    Nothing,
}

fn main() -> anyhow::Result<()> {
    let myint: MyInt = 10.into();
    let v = myint + 20.into();
    let v1: i32 = v.into();

    println!("v = {}, my_int = {}, v1 = {}", v, myint, v1);

    let e: MyEnum = 10i32.into();
    let e1: MyEnum = 20u32.into();
    let e2 = MyEnum::Nothing;
    println!("e = {}, e1 = {}, e2 = {}", e, e1, e2);
    Ok(())
}
