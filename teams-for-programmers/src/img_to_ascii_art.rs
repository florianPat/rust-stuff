use image::{GenericImageView};

fn get_str_ascii(intent :u8)-> &'static str{
    let index = intent/32;
    let ascii = [" ",".",",","-","~","+","=","@"];
    return ascii[index as usize];
}

fn get_image(dir: &str, scale: u32) -> String {
    let mut result = String::from("");

    let img = image::open(dir).unwrap();
    log::info!("{:?}", img.dimensions());
    let (width,height) = img.dimensions();
    for y in 0..height {
        for x in 0..width {
            if y % (scale * 2) == 0 && x % scale == 0 {
                let pix = img.get_pixel(x,y);
                let mut intent = pix[0] / 3 + pix[1] / 3 + pix[2] / 3;
                if pix[3] == 0 {
                    intent = 0;
                }
                result.push(get_str_ascii(intent));
            }
        }

        if y%(scale*2)==0{
            result.push("\n");
        }
    }

    return result;
}
