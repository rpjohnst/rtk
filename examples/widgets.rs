extern crate rtk;

fn main() {
    let app = rtk::App::new();
    let window = app.new_window("RTK Window", 600, 400).unwrap();
    let _button = window.new_button(10, 10, "Hello World");
    window.show();
    app.run();
}
