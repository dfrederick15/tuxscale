use std::path::PathBuf;

fn main() {
    let svg_path = PathBuf::from("assets/icon.svg");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let png_path = out_dir.join("icon.png");

    println!("cargo:rerun-if-changed=assets/icon.svg");

    let svg_data = std::fs::read_to_string(&svg_path).expect("assets/icon.svg missing");

    let tree = {
        let opt = resvg::usvg::Options::default();
        resvg::usvg::Tree::from_str(&svg_data, &opt).expect("invalid SVG")
    };

    let size = 64u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).expect("pixmap alloc failed");

    let transform = resvg::tiny_skia::Transform::from_scale(
        size as f32 / tree.size().width(),
        size as f32 / tree.size().height(),
    );

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap.save_png(&png_path).expect("failed to write icon.png");
}
