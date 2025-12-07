#[repr(C, packed)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[unsafe(no_mangle)]
pub extern "C" fn color(color: Color) {
    println!(
        "Color: R={}, G={}, B={}, A={}",
        color.r, color.g, color.b, color.a
    );
}
