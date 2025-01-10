use egui::{vec2, Align2, Color32, FontId, Margin, Rect, Rounding, Stroke, Ui, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    view: Option<View>,
    #[serde(skip)]
    bodies: Vec<Body>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct View {
    center: Vec2,
    scale: f32,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Body {
    name: String,
    mass_kg: f32,
    position: Vec2,
    color: Color32,
    velocity: Vec2,
}

impl Body {
    fn orbiting(
        name: &str,
        mass_kg: f32,
        orbital_radius_km: f32,
        color: Color32,
        degrees: f32,
    ) -> Self {
        let radius = orbital_radius_km * 1e3;
        let radians = degrees.to_radians();
        Self {
            name: name.to_string(),
            mass_kg,
            position: vec2(radius * radians.cos(), radius * radians.sin()),
            color,
            velocity: Vec2::ZERO,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            view: None,
            bodies: vec![
                Body::orbiting("Sun", 1.9891e30, 0., Color32::GOLD, 0.),
                Body::orbiting("Earth", 5.97219e24, 1.5e8, Color32::BLUE, 20.),
            ],
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::containers::Frame::default().inner_margin(Margin::ZERO))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                let screen_center = rect.center();
                let body_radius = 10.;
                const MARGIN: f32 = 8.0;
                self.view = None;
                if self.view.is_none() {
                    let min_x_phys = self
                        .bodies
                        .iter()
                        .map(|b| b.position.x)
                        .fold(f32::INFINITY, |a, b| a.min(b));
                    let max_x_phys = self
                        .bodies
                        .iter()
                        .map(|b| b.position.x)
                        .fold(f32::NEG_INFINITY, |a, b| a.max(b));
                    let min_y_phys = self
                        .bodies
                        .iter()
                        .map(|b| b.position.y)
                        .fold(f32::INFINITY, |a, b| a.min(b));
                    let max_y_phys = self
                        .bodies
                        .iter()
                        .map(|b| b.position.y)
                        .fold(f32::NEG_INFINITY, |a, b| a.max(b));
                    let mid_x_phys = (max_x_phys + min_x_phys) / 2.;
                    let mid_y_phys = (max_y_phys + min_y_phys) / 2.;
                    let span_x_phys = max_x_phys - min_x_phys;
                    let span_y_phys = max_y_phys - min_y_phys;
                    let allowed_centers = rect.shrink(body_radius + MARGIN);
                    let x_scale = allowed_centers.width() / span_x_phys;
                    let y_scale = allowed_centers.height() / span_y_phys;
                    let scale = x_scale.min(y_scale);
                    self.view = Some(View {
                        center: vec2(mid_x_phys * scale, mid_y_phys * scale),
                        scale,
                    });
                }
                // let shown_centers = rect.expand(body_radius);
                let view = self.view.clone().unwrap();
                for Body {
                    name,
                    // mass_kg,
                    position,
                    color,
                    ..
                } in &self.bodies
                {
                    let mut offset = *position * view.scale - view.center;
                    offset.y = -offset.y;
                    let view_pos = screen_center + offset;
                    ui.painter().text(
                        view_pos - vec2(0., body_radius),
                        Align2::CENTER_BOTTOM,
                        name,
                        FontId::proportional(10.),
                        Color32::LIGHT_GRAY,
                    );
                    ui.painter().circle(
                        view_pos,
                        body_radius,
                        *color,
                        Stroke::new(1., color.lighten(0.5)),
                    );
                }
            });
    }
}

#[allow(unused)]
trait UiExt {
    fn debug_rect(&mut self, rect: Rect);
}

impl UiExt for Ui {
    fn debug_rect(&mut self, rect: Rect) {
        self.painter().rect(
            rect,
            Rounding::ZERO,
            Color32::from_rgba_unmultiplied(0, 255, 0, 50),
            Stroke::new(1., Color32::GREEN),
        );
    }
}

trait Color32Ext {
    fn lighten(&self, amount: f32) -> Color32;
}

impl Color32Ext for Color32 {
    fn lighten(&self, amount: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(
            lighten_channel(self.r(), amount),
            lighten_channel(self.r(), amount),
            lighten_channel(self.r(), amount),
            self.a(),
        )
    }
}

fn lighten_channel(value: u8, amount: f32) -> u8 {
    let headroom = 255 - value;
    let increase = headroom as f32 * amount;
    (value + increase as u8).min(255)
}
