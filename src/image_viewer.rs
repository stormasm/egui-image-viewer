use egui::{epaint::ColorImage, TextureHandle, TextureOptions};
use image::DynamicImage;

use crate::color;

pub struct ImageViewer {
    original_image: Option<DynamicImage>,
    pub image: Option<DynamicImage>,
    color_image: Option<ColorImage>,
    texture_handle: Option<TextureHandle>,
    zoom_factor: f32,
    drag_offset: [f32; 2],

    value_floor: f32,
    value_ceil: f32,

    histogram_rects: Option<Vec<egui::Shape>>,
}

impl ImageViewer {
    pub fn from_image(image: DynamicImage) -> Self {
        let original_image = Some(image.clone());
        let image = Some(image);
        let color_image = None;
        let texture_handle = None;
        let zoom_factor = 1.0;
        let drag_offset = [0.0, 0.0];
        let value_floor = 0.0;
        let value_ceil = 1.0;
        let histogram_rects = None;

        Self {
            original_image,
            image,
            color_image,
            texture_handle,
            zoom_factor,
            drag_offset,
            value_floor,
            value_ceil,
            histogram_rects,
        }
    }

    pub fn load_color_image(&mut self) {
        if let Some(image) = &self.image {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let _ = self
                .color_image
                .replace(egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice(),
                ));
        } else {
            self.color_image = None;
        }
    }

    pub fn load_texture(&mut self, ctx: &egui::Context) {
        if let Some(cimage) = &self.color_image {
            let options = TextureOptions::default();
            let texture_handle = ctx.load_texture("my image texture", cimage.clone(), options);
            let _ = self.texture_handle.replace(texture_handle);
        } else {
            self.texture_handle = None;
        }
    }

    fn handle_zoom(&mut self, rect: egui::Rect) -> (egui::Rect, egui::Rect) {
        let (rect, uv_rect) = if self.zoom_factor < 1.0 {
            let uv_size = 1.0 * self.zoom_factor;
            let margin = 1.0 - uv_size;

            let lower = margin / 2.0;
            let upper = uv_size + margin / 2.0;
            let uv_rect = egui::Rect::from_min_max(
                egui::Pos2::new(lower, lower),
                egui::Pos2::new(upper, upper),
            );
            (rect, uv_rect)
        } else {
            let uv_rect =
                egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));

            let width = rect.width() / self.zoom_factor;
            let height = rect.height() / self.zoom_factor;
            let margin_x = rect.width() - width;
            let margin_y = rect.height() - height;

            let rect = egui::Rect::from_min_max(
                egui::Pos2::new(rect.left() + margin_x / 2.0, rect.top() + margin_y / 2.0),
                egui::Pos2::new(
                    rect.left() + width + margin_x / 2.0,
                    rect.top() + height + margin_y / 2.0,
                ),
            );
            (rect, uv_rect)
        };

        return (rect, uv_rect);
    }

    fn handle_drag(
        &mut self,
        drag_delta: egui::Vec2,
        rect: &mut egui::Rect,
        uv_rect: &mut egui::Rect,
    ) {
        if drag_delta[0] != 0.0 || drag_delta[1] != 0.0 {
            self.drag_offset[0] += drag_delta[0];
            self.drag_offset[1] += drag_delta[1];
        }
        if self.zoom_factor >= 1.0 {
            rect.min.x += self.drag_offset[0];
            rect.max.x += self.drag_offset[0];
            rect.min.y += self.drag_offset[1];
            rect.max.y += self.drag_offset[1];
        } else {
            uv_rect.min.x -= self.drag_offset[0] / rect.width() * self.zoom_factor;
            uv_rect.max.x -= self.drag_offset[0] / rect.width() * self.zoom_factor;
            if uv_rect.min.x < 0.0 {
                let delta = -uv_rect.min.x;
                uv_rect.min.x = 0.0;
                rect.min.x += delta * rect.width() / self.zoom_factor;
            }
            if uv_rect.max.x > 1.0 {
                let delta = uv_rect.max.x - 1.0;
                uv_rect.max.x = 1.0;
                rect.max.x -= delta * rect.width() / self.zoom_factor;
            }

            uv_rect.min.y -= self.drag_offset[1] / rect.height() * self.zoom_factor;
            uv_rect.max.y -= self.drag_offset[1] / rect.height() * self.zoom_factor;
            if uv_rect.min.y < 0.0 {
                let delta = -uv_rect.min.y;
                uv_rect.min.y = 0.0;
                rect.min.y += delta * rect.height() / self.zoom_factor;
            }
            if uv_rect.max.y > 1.0 {
                let delta = uv_rect.max.y - 1.0;
                uv_rect.max.y = 1.0;
                rect.max.y -= delta * rect.height() / self.zoom_factor;
            }
        }
    }

    fn allocate_sizes(&self, ui: &egui::Ui) -> (egui::Vec2, egui::Vec2) {
        let width = self.image.as_ref().unwrap().width() as f32;
        let height = self.image.as_ref().unwrap().height() as f32;
        let aspect_ratio = width / height;
        let mut size = ui.available_size_before_wrap();
        let mut canvas_size = egui::Vec2::new(size.x * 0.1, size.y);
        size.x -= canvas_size.x;
        let avail_aspect_ratio = size.x / size.y;
        if aspect_ratio < avail_aspect_ratio {
            // We should scale down the width of the size
            size.x *= aspect_ratio / avail_aspect_ratio;
        } else {
            // We should scale down the height of the size
            size.y *= avail_aspect_ratio / aspect_ratio;
            canvas_size.y = size.y;
        }
        return (size, canvas_size);
    }

    pub fn content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(texture) = self.texture_handle.clone() {
            let (size, canvas_size) = self.allocate_sizes(ui);

            let original_clip = ui.clip_rect();

            ui.horizontal(|ui| {
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
                if response.hovered() {
                    ui.input(|i| {
                        self.zoom_factor *= i.zoom_delta();
                    });
                }

                ui.set_clip_rect(rect);

                // Handle zoom
                let (mut rect, mut uv_rect) = self.handle_zoom(rect);

                // Handle drag
                let drag_delta = response.drag_delta();
                self.handle_drag(drag_delta, &mut rect, &mut uv_rect);

                ui.painter().image(
                    texture.id(),
                    rect,
                    uv_rect,
                    egui::Color32::from_rgb(255, 255, 255),
                );
                ui.set_clip_rect(original_clip);
                ui.allocate_ui(canvas_size, |ui| {
                    self.canvas(ui);
                });
            });
            ui.set_clip_rect(original_clip);

            ui.horizontal(|ui| {
                if ui.button("Reset zoom").clicked() {
                    self.zoom_factor = 1.0;
                }
                if ui.button("Reset drag").clicked() {
                    self.drag_offset[0] = 0.0;
                    self.drag_offset[1] = 0.0;
                }
                if ui.button("Reset view").clicked() {
                    self.zoom_factor = 1.0;
                    self.drag_offset[0] = 0.0;
                    self.drag_offset[1] = 0.0;
                }
            });

            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.value_floor, 0.0..=1.0));
                ui.add(egui::Slider::new(&mut self.value_ceil, 0.0..=1.0));

                if self.original_image.is_some() && ui.button("Apply Scaling").clicked() {
                    let mut copy = self.original_image.as_ref().unwrap().to_rgb32f();

                    for (_ix, _iy, p) in copy.enumerate_pixels_mut() {
                        let [h, s, v] = color::rgb2hsv(p.0);

                        let new_value =
                            ((v - self.value_floor).max(0.0) + (1.0 - self.value_ceil)).min(1.0);

                        let rgb = color::hsv2rgb([h, s, new_value]);

                        p.0 = rgb;
                    }

                    self.image.replace(DynamicImage::from(copy));
                    self.load_color_image();
                    self.load_texture(ctx);
                }
            });
        } else {
            self.load_color_image();
            self.load_texture(ctx);
        }
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let space = ui.available_size_before_wrap();
        let (response, painter) = ui.allocate_painter(space, egui::Sense::click_and_drag());
        if let Some(rects) = &self.histogram_rects {
            painter.add(egui::Shape::rect_filled(
                response.rect,
                0.0,
                egui::Color32::from_rgb(0, 0, 100),
            ));

            painter.extend(rects.clone());

            let to_screen = egui::emath::RectTransform::from_to(
                egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(response.rect.width() / response.rect.height(), 1.0),
                ),
                response.rect,
            );
            let from_screen = to_screen.inverse();

            if let Some(pos) = response.interact_pointer_pos() {
                let normalized_pos = from_screen * pos;
                let dist_floor = (1.0 - normalized_pos.y - self.value_floor).abs();
                let dist_ceil = (1.0 - normalized_pos.y - self.value_ceil).abs();
                let value = (1.0 - normalized_pos.y).clamp(0.0, 1.0);
                if dist_floor < dist_ceil {
                    self.value_floor = value;
                } else {
                    self.value_ceil = value;
                }
            }

            // Draw value floor
            painter.add(egui::Shape::line_segment(
                [
                    to_screen * egui::Pos2::new(0.0, 1.0 - self.value_floor),
                    to_screen * egui::Pos2::new(1.0, 1.0 - self.value_floor),
                ],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255)),
            ));
            // Draw value ceil
            painter.add(egui::Shape::line_segment(
                [
                    to_screen * egui::Pos2::new(0.0, 1.0 - self.value_ceil),
                    to_screen * egui::Pos2::new(1.0, 1.0 - self.value_ceil),
                ],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255)),
            ));
        } else if let Some(cimage) = &self.color_image {
            let values: Vec<_> = cimage
                .pixels
                .iter()
                .map(|c| {
                    color::rgb2hsv([
                        c.r() as f32 / 255.0,
                        c.g() as f32 / 255.0,
                        c.b() as f32 / 255.0,
                    ])[2]
                })
                .collect();
            let n_bins = 10;
            let mut bin_values = vec![0; n_bins];
            let bin_size = 1.0 / (n_bins - 1) as f32;
            for &v in values.iter() {
                let b_ix = (v.clamp(0.0, 1.0) / bin_size).floor() as usize;
                bin_values[b_ix] += 1;
            }

            let max_value = *bin_values.iter().max().unwrap();
            let min_value = 0;
            let max_value = max_value + (max_value - min_value) / 5;

            let scaled_values: Vec<_> = bin_values
                .iter()
                .map(|c| (*c as f32 - min_value as f32) / (max_value as f32 - min_value as f32))
                .collect();

            let bin_size = 1.0 / (n_bins as f32);

            let transform_rect = |rect: egui::Rect| {
                let current_width = rect.width();
                let current_height = rect.height();

                let new_width = response.rect.width() * current_width;
                let new_height = response.rect.height() * current_height;

                let current_start = rect.min;
                let target_start = response.rect.min;

                let new_start_x = target_start.x + response.rect.width() * current_start.x;
                let new_start_y = target_start.y + response.rect.height() * current_start.y;

                return egui::Rect::from_min_size(
                    egui::Pos2::new(new_start_x, new_start_y),
                    egui::Vec2::new(new_width, new_height),
                );
            };

            let rects = scaled_values
                .iter()
                .enumerate()
                .map(|(ix, v)| {
                    egui::Shape::rect_filled(
                        transform_rect(egui::Rect::from_x_y_ranges(
                            (1.0 - *v)..=1.0,
                            (1.0 - (ix as f32 + 1.0) * bin_size)..=(1.0 - ix as f32 * bin_size),
                        )),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(255, 0, 0, 150),
                    )
                })
                .collect();

            self.histogram_rects.replace(rects);
        }
    }
}
