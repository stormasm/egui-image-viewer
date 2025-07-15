pub fn rgb2hsv(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let c = max - min;
    let value = max;
    let sat_value = if value == 0.0 { 0.0 } else { c / value };
    let hue = if c == 0.0 {
        0.0
    } else if r >= g && r >= b {
        (g - b) / c % 6.0
    } else if g >= r && g >= b {
        (b - r) / c + 2.0
    } else {
        (r - g) / c + 4.0
    };
    let hue = hue * 60.0;

    return [hue, sat_value, value];
}

pub fn hsv2rgb(hsv: [f32; 3]) -> [f32; 3] {
    let [hue, sat, value] = hsv;
    let hue = hue / 60.0;
    let c = value * sat;
    let x = c * (1.0 - (hue % 2.0 - 1.0).abs());

    let (r, g, b) = if (0.0..1.0).contains(&hue) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&hue) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&hue) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&hue) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&hue) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = value - c;

    let (r, g, b) = (r + m, g + m, b + m);

    return [r, g, b];
}
