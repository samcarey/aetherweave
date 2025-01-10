use egui::{vec2, Align2, Color32, FontId, Margin, Rect, Rounding, Stroke, Ui, Vec2};
use egui_plot::{CoordinatesFormatter, Corner, Plot, PlotPoint, PlotPoints, Points};

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
                let body_radius = 10.;
                let plot = Plot::new("main_plot")
                    .show_grid(false)
                    .show_axes(false)
                    .coordinates_formatter(
                        Corner::LeftBottom,
                        CoordinatesFormatter::new(|_, _| "".to_string()),
                    )
                    .cursor_color(Color32::TRANSPARENT)
                    .show(ui, |ui| {
                        for Body {
                            name,
                            position,
                            color,
                            ..
                        } in &self.bodies
                        {
                            ui.add(
                                Points::new(PlotPoints::new(vec![[
                                    position.x as f64,
                                    position.y as f64,
                                ]]))
                                .color(*color)
                                .radius(body_radius)
                                .name(name),
                            );
                        }
                    });
                let transform = plot.transform;
                let painter = ui.painter();
                for Body { name, position, .. } in &self.bodies {
                    let center = transform
                        .position_from_point(&PlotPoint::new(position.x as f64, position.y as f64));
                    painter.circle_stroke(center, body_radius, Stroke::new(1., Color32::GREEN));
                    ui.painter().text(
                        center - vec2(0., body_radius),
                        Align2::CENTER_BOTTOM,
                        name,
                        FontId::proportional(10.),
                        Color32::LIGHT_GRAY,
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
