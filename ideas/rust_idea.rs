component! {
    pub default_color: Brush;
    pub hovered_color: Brush;
    pub pressed_color: Brush;
    pub click: Callback;
    pub text_color: Brush;
    pub text: String;

    pointer_in: Callback;
    pointer_out: Callback;
    pointer_press: Callback;
    pointer_release: Callback;
    background: Brush;

    root {
        row.grow {
            background: (background);
            events.pointer.click: (click);
            events.pointer.in: (pointer_in);
            events.pointer.out: (pointer_out);
            events.pointer.press: (pointer_press);
            events.pointer.release: (pointer_release);
            padding: 10px;
            x: 100px;
            y: 100px;
            span { color: (text_color); (text) }
        }
    }
}

// auto-generated
// struct Component {
//     default_color: Brush,
//     hovered_color: Brush,
//     pressed_color: Brush,
//     click: Callback,
//     text_color: Brush,
//     text: String,
//     pointer_in: Callback,
//     pointer_out: Callback,
//     pointer_press: Callback,
//     pointer_release: Callback,
//     background: Brush,
// }

impl Component {
    fn init() -> Self {
        Self {
            default_color: Brush::color_rgb(204, 204, 204),
            hovered_color: Brush::color_rgb(221, 221, 221),
            pressed_color: Brush::color_rgb(170, 170, 170),
            background:    Brush::color_rgb(204, 204, 204),
            ..Default::default()
        }
    }
    fn pointer_in(&mut self) {
        self.background = self.hovered_color.clone();
    }
    fn pointer_out(&mut self) {
        self.background = self.default_color.clone();
    }
    fn pointer_press(&mut self) {
        self.background = self.pressed_color.clone();
    }
    fn pointer_release(&mut self) {
        self.background = self.hovered_color.clone();
    }
}

fn main() {
    ui_native::create_window(Component::construct)
        .event_loop(|window_handle, e| {
            match e {
                ui_native::Event::Exit => {
                    window_handle.close();
                    return ui_native::EventResult::Terminate;
                },
                _ => {}
            }
        })
        .block_until_closed();
}

