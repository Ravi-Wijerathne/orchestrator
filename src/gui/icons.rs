use eframe::egui;

pub struct AppIcons {
    pub logo: egui::TextureHandle,
    pub nav_welcome: egui::TextureHandle,
    pub nav_dashboard: egui::TextureHandle,
    pub nav_drives: egui::TextureHandle,
    pub nav_settings: egui::TextureHandle,
    pub watcher: egui::TextureHandle,
    pub folder: egui::TextureHandle,
    pub rules: egui::TextureHandle,
    pub cat_images: egui::TextureHandle,
    pub cat_videos: egui::TextureHandle,
    pub cat_music: egui::TextureHandle,
    pub cat_docs: egui::TextureHandle,
    pub cat_archives: egui::TextureHandle,
}

impl AppIcons {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            logo: load_texture(ctx, "app_logo", include_bytes!("../../icons/app_logo.png")),
            nav_welcome: load_texture(ctx, "nav_welcome", include_bytes!("../../icons/nav_welcome.png")),
            nav_dashboard: load_texture(ctx, "nav_dashboard", include_bytes!("../../icons/nav_dashboard.png")),
            nav_drives: load_texture(ctx, "nav_drives", include_bytes!("../../icons/nav_drives.png")),
            nav_settings: load_texture(ctx, "nav_settings", include_bytes!("../../icons/nav_settings.png")),
            watcher: load_texture(ctx, "icon_watcher", include_bytes!("../../icons/icon_watcher.png")),
            folder: load_texture(ctx, "icon_folder", include_bytes!("../../icons/icon_folder.png")),
            rules: load_texture(ctx, "icon_rules", include_bytes!("../../icons/icon_rules.png")),
            cat_images: load_texture(ctx, "cat_images", include_bytes!("../../icons/cat_images.png")),
            cat_videos: load_texture(ctx, "cat_videos", include_bytes!("../../icons/cat_videos.png")),
            cat_music: load_texture(ctx, "cat_music", include_bytes!("../../icons/cat_music.png")),
            cat_docs: load_texture(ctx, "cat_docs", include_bytes!("../../icons/cat_docs.png")),
            cat_archives: load_texture(ctx, "cat_archives", include_bytes!("../../icons/cat_archives.png")),
        }
    }

    pub fn category_icon(&self, category: &str) -> &egui::TextureHandle {
        match category.to_lowercase().as_str() {
            "images" | "image" | "img" => &self.cat_images,
            "videos" | "video" | "vid" => &self.cat_videos,
            "music" | "audio" | "aud" => &self.cat_music,
            "documents" | "docs" | "doc" => &self.cat_docs,
            "archives" | "archive" | "arc" => &self.cat_archives,
            _ => &self.folder,
        }
    }
}

fn load_texture(ctx: &egui::Context, name: &str, png_bytes: &[u8]) -> egui::TextureHandle {
    let img = image::load_from_memory(png_bytes).expect("Failed to load embedded icon image");
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
}

pub fn render_icon(ui: &mut egui::Ui, texture: &egui::TextureHandle, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let mut mesh = egui::Mesh::with_texture(texture.id());
        mesh.add_rect_with_uv(
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        ui.painter().add(egui::Shape::mesh(mesh));
    }
    response
}

pub fn load_window_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../../icons/app_logo.png")).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}
