use eframe::egui;

struct App {
    name: String,
    age: u32,
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rusty Collab",
        options,
        Box::new(|_cc| {
            // Setup
            let app = App {
                name: String::new(),
                age: 0,
            };
            Ok(Box::new(app))
        }),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_ui(ui, self);
        });
    }
}

fn render_ui(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Rusty Collab");
    ui.horizontal(|ui| {
        ui.label("Your name: ");
        ui.text_edit_singleline(&mut app.name);
    });
    ui.add(egui::Slider::new(&mut app.age, 0..=120).text("age"));
    if ui.button("Increment").clicked() {
        app.age += 1;
    }
    ui.label(format!("Hello '{}', age {}", app.name, app.age));
}
