use putpng::crop::*;

fn main() {
    match apply_crop(["sample.png".into()].into_iter()) {
        Ok(_) => println!("Image cropped successfully!"),
        Err(e) => eprintln!("Error cropping image: {}", e),
    }
}
