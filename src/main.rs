use eframe::egui;
use image::{DynamicImage, GrayImage};
use std::thread;

mod color;
mod compute;
mod image_viewer;
use image_viewer::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App {
                ..Default::default()
            }))
        }),
    )
}
#[derive(Default)]
struct App {
    image_viewers: Vec<ImageViewer>,
    bg_job_handle: Option<thread::JoinHandle<GrayImage>>,
    job_index: usize,
}

#[allow(dead_code)]
impl App {
    fn start_sobel_threaded(&mut self, luma_image: GrayImage, job_index: usize) {
        let handle = thread::spawn(|| {
            return compute::run_sobel_job(luma_image);
        });

        let _ = self.bg_job_handle.replace(handle);
        self.job_index = job_index;
    }

    fn check_bg_job(&mut self, ctx: &egui::Context) {
        if let Some(handle) = self.bg_job_handle.take_if(|h| h.is_finished()) {
            let result = handle.join().unwrap();

            // Copy to image
            if let Some(iv) = self.image_viewers.get_mut(self.job_index) {
                iv.image.replace(DynamicImage::from(result));
                iv.load_color_image();
                iv.load_texture(ctx);
            }
        }
    }
}

fn load_image() -> Option<image::DynamicImage> {
    let reader = image::ImageReader::open("assets/valve.png").ok()?;
    let image = reader.decode().ok()?;
    return Some(image);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load Button").clicked() || ui.input(|i| i.key_released(egui::Key::I)) {
                let some_image = load_image();
                if let Some(image) = some_image {
                    println!("Loaded succesfully");
                    self.image_viewers.push(ImageViewer::from_image(image));
                } else {
                    println!("Failed to load");
                }
            }

            let mut started_index = None;
            for (ix, image_viewer) in self.image_viewers.iter_mut().enumerate() {
                ui.allocate_ui(egui::Vec2::new(400.0, 400.0), |ui| {
                    image_viewer.content(ui, ctx);
                });
                if ui.button("Run Sobel").clicked() && image_viewer.image.is_some() {
                    started_index = Some(ix);
                }
            }
            if let Some(index) = started_index {
                let img = self.image_viewers[index].image.as_ref().unwrap();
                let luma_image = img.to_luma8();
                self.start_sobel_threaded(luma_image, index);
            }

            if ui.input(|i| i.key_released(egui::Key::Q)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            if self.bg_job_handle.is_some() {
                self.check_bg_job(ctx);
            }
        });
    }
}
