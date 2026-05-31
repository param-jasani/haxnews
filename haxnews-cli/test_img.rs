use ratatui_image::picker::Picker;
fn main() {
    let mut picker = Picker::new((8, 16));
    let img = image::DynamicImage::new_rgb8(10, 10);
    let proto = picker.new_resize_protocol(img);
}
