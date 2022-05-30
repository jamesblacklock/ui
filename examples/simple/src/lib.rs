include!("./dist/simple.rs");

impl ui::DefaultProps for Props {
    fn default() -> Props {
        Props {
            toggle_show: Callback::from(&Simple::toggle_show),
            ..Default::default()
        }
    }
}

impl ui::Component for Simple {
    fn on_init(&mut self) {
        self.state = 420;
    }
    fn on_update(&mut self) {
        self.state += 1;
    }
}

impl Simple {
    fn toggle_show(&mut self) {
        self.show = !self.show;
        self.x = ui::Length::Px(self.x.to_px() + 4.0);
    }
}