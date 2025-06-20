struct UIApp {}

impl eframe::App for UIApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        // egui::...
    }
}

impl UIApp {
    fn new() -> Self {
        UIApp {}
    }
}
