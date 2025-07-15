use image::GrayImage;

pub fn run_sobel_job(ref_copy: GrayImage) -> GrayImage {
    // Get temp buffer
    let width = ref_copy.width();
    let height = ref_copy.height();
    let mut gray_copy = ref_copy.clone();

    // Run operation
    // TODO: Just iterate over raw pixels and manually extract gray scale value

    macro_rules! get {
        ($buffer:ident, $ix:expr, $iy:expr) => {{
            if $ix >= 0 && $iy >= 0 && $ix < width as i32 && $iy < height as i32 {
                $buffer[($ix as u32, $iy as u32)].0[0] as f32
            } else {
                0.0
            }
        }};
    }

    for (ix, iy, _p) in ref_copy.enumerate_pixels() {
        let this_row: f32 = 2.0 * get!(ref_copy, ix as i32 - 1, iy as i32)
            - 2.0 * get!(ref_copy, ix as i32 + 1, iy as i32);
        let upper_row: f32 = get!(ref_copy, ix as i32 - 1, iy as i32 - 1)
            - get!(ref_copy, ix as i32 + 1, iy as i32 - 1);
        let lower_row: f32 = get!(ref_copy, ix as i32 - 1, iy as i32 + 1)
            - get!(ref_copy, ix as i32 + 1, iy as i32 + 1);

        let this_col: f32 = 2.0 * get!(ref_copy, ix as i32, iy as i32 - 1)
            - 2.0 * get!(ref_copy, ix as i32, iy as i32 + 1);

        let left_col: f32 = get!(ref_copy, ix as i32 - 1, iy as i32 - 1)
            - get!(ref_copy, ix as i32 - 1, iy as i32 + 1);

        let right_col: f32 = get!(ref_copy, ix as i32 + 1, iy as i32 - 1)
            - get!(ref_copy, ix as i32 + 1, iy as i32 + 1);
        gray_copy[(ix, iy)].0[0] = (this_row.powf(2.0)
            + upper_row.powf(2.0)
            + lower_row.powf(2.0)
            + this_col.powf(2.0)
            + left_col.powf(2.0)
            + right_col.powf(2.0))
        .sqrt()
        .clamp(0.0, 255.0)
        .round() as u8;
    }

    return gray_copy;
}
