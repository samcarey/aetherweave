use egui::{
    vec2, Align2, Color32, Event, FontId, Grid, Id, Margin, PointerButton, Pos2, Rect, RichText,
    Rounding, Stroke, Theme, Ui, Vec2, Window,
};
use egui_plot::{Line, LineStyle, Plot, PlotPoint, PlotPoints, Points};
use std::rc::{Rc, Weak};
use std::time::Instant;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    view: Option<View>,
    #[serde(skip)]
    bodies: Vec<Rc<Body>>,
    #[serde(skip)]
    last_update: Option<Instant>,
    selected: Weak<Body>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct View {
    center: Vec2,
    scale: f32,
}

// Need interior mutability for the Rc<Body>
#[derive(serde::Deserialize, serde::Serialize)]
struct Body {
    name: String,
    mass_kg: f32,
    #[serde(skip)]
    position: std::cell::Cell<Vec2>,
    color: Color32,
    #[serde(skip)]
    velocity: std::cell::Cell<Vec2>,
}

const G: f32 = 6.67430e-11; // gravitational constant
const EARTH_MASS_KG: f32 = 5.97219e24;
const SECONDS_PER_MINUTE: f32 = 60.0;
const SIMULATION_SPEED: f32 = 60.0 * 24.0 * 365.0; // 1 Earth year per minute

impl Body {
    fn orbiting(
        name: &str,
        mass_kg: f32,
        orbital_radius_km: f32,
        color: Color32,
        degrees: f32,
    ) -> Rc<Self> {
        let radius = orbital_radius_km * 1e3;
        let radians = degrees.to_radians();
        let position = vec2(radius * radians.cos(), radius * radians.sin());
        
        // Calculate orbital velocity using v = sqrt(GM/r)
        let sun_mass = 1.9891e30;
        let velocity_magnitude = (G * sun_mass / radius).sqrt();
        
        // Velocity vector perpendicular to position vector for circular orbit
        let velocity = vec2(
            -velocity_magnitude * radians.sin(),
            velocity_magnitude * radians.cos(),
        );

        Rc::new(Self {
            name: name.to_string(),
            mass_kg,
            position: std::cell::Cell::new(position),
            color,
            velocity: std::cell::Cell::new(velocity),
        })
    }

    fn update_position(&self, dt: f32) {
        let current_pos = self.position.get();
        let current_vel = self.velocity.get();
        
        // Update position based on velocity
        let new_pos = current_pos + current_vel * dt;
        self.position.set(new_pos);
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            bodies: vec![
                Body::orbiting("Sun", 1.9891e30, 0., Color32::GOLD, 0.),
                Body::orbiting("Mercury", 3.285e23, 57.9e6, Color32::GRAY, 200.),
                Body::orbiting("Venus", 4.867e24, 108.2e6, Color32::GREEN, 110.),
                Body::orbiting("Earth", EARTH_MASS_KG, 1.5e8, Color32::BLUE, 40.),
                Body::orbiting("Mars", 6.39e23, 228e6, Color32::RED, 40.),
                Body::orbiting("Jupiter", 1.899e27, 778.5e6, Color32::BROWN, 75.),
                Body::orbiting("Saturn", 5.683e26, 1.434e9, Color32::YELLOW, 60.),
                Body::orbiting("Uranus", 8.681e25, 2.871e9, Color32::LIGHT_BLUE, 30.),
                Body::orbiting("Neptune", 1.024e26, 4.495e9, Color32::BLUE, 15.),
            ],
            view: None,
            last_update: None,
            selected: Default::default(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(Theme::Dark);
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
        // Update positions
        let now = Instant::now();
        if let Some(last_update) = self.last_update {
            let dt = now.duration_since(last_update).as_secs_f32();
            // Apply simulation speed scaling
            let scaled_dt = dt * SIMULATION_SPEED;
            
            // Update all bodies except the Sun (index 0)
            for body in self.bodies.iter().skip(1) {
                body.update_position(scaled_dt);
            }
        }
        self.last_update = Some(now);
        
        // Request continuous updates
        ctx.request_repaint();

        egui::CentralPanel::default()
            .frame(egui::containers::Frame::default().inner_margin(Margin::ZERO))
            .show(ctx, |ui| {
                let click = ui.get_click();
                let body_radius = 10.;
                let plot = Plot::new("main_plot")
                    .show_grid(false)
                    .show_axes(false)
                    .data_aspect(1.0)
                    .label_formatter(|_, _| "".to_string())
                    .cursor_color(Color32::TRANSPARENT)
                    .show(ui, |ui| {
                        for body in self.bodies.iter() {
                            let position = body.position.get();
                            ui.add(
                                Points::new(PlotPoints::new(vec![[
                                    position.x as f64,
                                    position.y as f64,
                                ]]))
                                .color(body.color)
                                .radius(body_radius)
                                .name(&body.name)
                                .id(Id::new(&body.name)),
                            );
                            let radius = position.length() as f64;
                            ui.add(
                                Line::new(PlotPoints::new(
                                    (0..=360)
                                        .filter(|x| *x % 2 == 0)
                                        .map(|deg| (deg as f64).to_radians())
                                        .map(|rad| [radius * rad.cos(), radius * rad.sin()])
                                        .collect::<Vec<_>>(),
                                ))
                                .style(LineStyle::Dotted { spacing: 4. })
                                .color(body.color)
                                .width(0.5),
                            );
                        }
                    });

                let mut clicked_on_body = false;
                for body_rc in self.bodies.iter() {
                    let highlighted = self
                        .selected
                        .upgrade()
                        .map(|selected| Rc::ptr_eq(&selected, &body_rc))
                        .unwrap_or_default();
                    let position = body_rc.position.get();
                    let center = plot
                        .transform
                        .position_from_point(&PlotPoint::new(position.x as f64, position.y as f64));
                    const HIGHLIGHT_RADIUS: f32 = 2.;
                    let color = if highlighted {
                        Color32::WHITE
                    } else {
                        Color32::LIGHT_GRAY
                    };
                    ui.painter().circle_stroke(
                        center,
                        body_radius,
                        Stroke::new(if highlighted { HIGHLIGHT_RADIUS } else { 0.5 }, color),
                    );
                    ui.painter().text(
                        center + vec2(body_radius + HIGHLIGHT_RADIUS + 3., -1.),
                        Align2::LEFT_CENTER,
                        &body_rc.name,
                        FontId::proportional(if highlighted { 16. } else { 12. }),
                        color,
                    );
                    if let Some(click) = click {
                        if (center - click).length() < body_radius + 5. {
                            self.selected = Rc::downgrade(body_rc);
                            clicked_on_body = true;
                        }
                    }
                }
                if click.is_some() && !clicked_on_body {
                    self.selected = Default::default();
                }
            });

        if let Some(body) = self.selected.upgrade() {
            Window::new(&body.name)
                .frame(
                    egui::containers::Frame::window(&ctx.style())
                        .stroke(Stroke::new(ctx.style().visuals.window_stroke.width, body.color)),
                )
                .anchor(Align2::CENTER_TOP, [0., 10.])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    Grid::new("stats").show(ui, |ui| {
                        ui.label(RichText::new("Mass:"));
                        let earth_masses = body.mass_kg / EARTH_MASS_KG;
                        ui.label(RichText::new(format!("{earth_masses:.1} x Earth")).monospace());
                        
                        ui.label(RichText::new("Velocity:"));
                        let velocity = body.velocity.get();
                        let speed_kms = velocity.length() / 1000.0;
                        ui.label(RichText::new(format!("{speed_kms:.1} km/s")).monospace());
                    });
                });
        }
    }
}

// Rest of the UiExt implementation remains the same...
#[allow(unused)]
trait UiExt {
    fn debug_rect(&mut self, rect: Rect);
    fn get_click(&mut self) -> Option<Pos2>;
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
    fn get_click(&mut self) -> Option<Pos2> {
        self.ctx().input(|r| {
            r.events.iter().find_map(|e| {
                if let Event::PointerButton {
                    button: PointerButton::Primary,
                    pressed: true,
                    pos,
                    ..
                } = e
                {
                    Some(*pos)
                } else {
                    None
                }
            })
        })
    }
}

// trait Color32Ext {
//     fn lighten(&self, amount: f32) -> Color32;
// }

// impl Color32Ext for Color32 {
//     fn lighten(&self, amount: f32) -> Color32 {
//         Color32::from_rgba_unmultiplied(
//             lighten_channel(self.r(), amount),
//             lighten_channel(self.r(), amount),
//             lighten_channel(self.r(), amount),
//             self.a(),
//         )
//     }
// }

// fn lighten_channel(value: u8, amount: f32) -> u8 {
//     let headroom = 255 - value;
//     let increase = headroom as f32 * amount;
//     (value + increase as u8).min(255)
// }
