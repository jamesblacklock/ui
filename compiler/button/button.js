function init(props) {
    return {
        default_color: '#ccc',
        hovered_color: '#ddd',
        pressed_color: '#aaa',
        background:    props.default_color ?? '#ccc',
        text_color:    '#000',
        pointer_in() {
            this.bind('background').to('hovered_color');
        },
        pointer_out() {
            this.bind('background').to('default_color');
        },
        pointer_press() {
            this.bind('background').to('pressed_color');
        },
        pointer_release() {
            this.bind('background').to('hovered_color');
        },
    }
}