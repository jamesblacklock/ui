include!("./dist/simple.rs");

impl ui::DefaultProps for Props {
    fn default() -> Props {
        Props {
            toggle_show: Callback::from(&|this: &mut Simple| this.state += 1),
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