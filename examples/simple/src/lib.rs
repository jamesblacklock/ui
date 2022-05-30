include!("./dist/simple.rs");

impl ui::Component for Simple {
    fn on_init(&mut self) {
        self.state = 420;
    }
    fn on_update(&mut self) {
        self.state += 1;
    }
}