use visioncortex::ColorImage;

fn read_image(path: &str) -> ColorImage {
    let img = image::open(path);
    let img = match img {
        Ok(file) => file.to_rgba8(),
        Err(_) => panic!("File not found {path}"),
    };

    let (width, height) = (img.width() as usize, img.height() as usize);
    ColorImage {
        pixels: img.as_raw().to_vec(),
        width,
        height,
    }
}

fn main() {
    let image = read_image(concat!(env!("CARGO_MANIFEST_DIR"), "/bubble code.png"));
    let code = bubble_code::decode_image(image);
    assert_eq!(code, ['🟡', '🔺', '🔷', '🟡', '🔷', '🔺', '🟡', '🔷']);
    println!("{code:?}");
}
