use std::{rc::Rc, sync::mpsc};

use goodboy_core::io::JoypadButton;
use wasm_bindgen::{closure::Closure, JsCast};
use winit::{dpi::LogicalSize, platform::web::WindowExtWebSys, window::Window};

use crate::io::{insert_cartridge, IoEvent, IoHandler};

pub fn init(window: Rc<Window>, ev_handler: &IoHandler, title_sender: &mpsc::Sender<String>) {
    // Initialize winit window with current dimensions of browser client
    window.set_inner_size(self::get_canvas_size());

    auto_resize_canvas(Rc::clone(&window)).ok();

    let document = web_sys::window()
        .and_then(|w| w.document())
        .expect("Could not find <document> tag");

    document
        .get_element_by_id("screen")
        .and_then(|scr| {
            scr.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("Could not create the canvas");

    let cartridge_cb = Closure::wrap(Box::new({
        let sender = ev_handler.sender.clone();
        let title_sender = title_sender.clone();
        move |_: web_sys::Event| {
            let sender = sender.clone();
            insert_cartridge(sender, title_sender.clone());
        }
    }) as Box<dyn FnMut(_)>);

    document
        .get_element_by_id("btn-insert-cartridge")
        .and_then(|btn| {
            btn.add_event_listener_with_callback("mousedown", cartridge_cb.as_ref().unchecked_ref())
                .ok()
        })
        .unwrap_or_else(|| {
            log::warn!("Error while assign a callback to the element \"btn-insert-cartridge\"")
        });
    cartridge_cb.forget();

    let ref sender = ev_handler.sender;
    bind_button("btn-a", JoypadButton::A, sender);
    bind_button("btn-b", JoypadButton::B, sender);
    bind_button("btn-up", JoypadButton::Up, sender);
    bind_button("btn-down", JoypadButton::Down, sender);
    bind_button("btn-left", JoypadButton::Left, sender);
    bind_button("btn-right", JoypadButton::Right, sender);
    bind_button("btn-start", JoypadButton::Start, sender);
    bind_button("btn-select", JoypadButton::Select, sender);
}

fn auto_resize_canvas(window: Rc<Window>) -> Result<(), &'static str> {
    let client_window = match web_sys::window() {
        Some(window) => window,
        None => return Err("Could not get the client window"),
    };

    // Listen for resize event on browser client. Adjust winit window dimensions
    // on event trigger
    let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        let size = self::get_canvas_size();
        window.set_inner_size(size)
    }) as Box<dyn FnMut(_)>);
    client_window
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    Ok(())
}

fn bind_button(id: &str, button: JoypadButton, sender: &mpsc::Sender<IoEvent>) {
    let document = web_sys::window().and_then(|win| win.document()).unwrap();

    let btn = document
        .get_element_by_id(id)
        .expect(&format!("Couldn't find #{:?}", id));

    let btn_press = self::key_press_cb(button, sender.clone());
    let btn_release = self::key_release_cb(button, sender.clone());

    btn.add_event_listener_with_callback("mousedown", btn_press.as_ref().unchecked_ref())
        .unwrap();
    btn.add_event_listener_with_callback("touchstart", btn_press.as_ref().unchecked_ref())
        .unwrap();
    btn.add_event_listener_with_callback("mouseup", btn_release.as_ref().unchecked_ref())
        .unwrap();
    btn.add_event_listener_with_callback("mouseout", btn_release.as_ref().unchecked_ref())
        .unwrap();
    btn.add_event_listener_with_callback("touchend", btn_release.as_ref().unchecked_ref())
        .unwrap();
    btn.add_event_listener_with_callback("touchcancel", btn_release.as_ref().unchecked_ref())
        .unwrap();

    btn_press.forget();
    btn_release.forget();
}

fn get_canvas_size() -> LogicalSize<f64> {
    let document = web_sys::window().and_then(|win| win.document()).unwrap();
    let canvas = document.get_element_by_id("screen-container").unwrap();

    let (width, height) = (canvas.client_width() as f64, canvas.client_height() as f64);
    LogicalSize::new(width, height)
}

fn key_press_cb(
    button: JoypadButton,
    tx: mpsc::Sender<IoEvent>,
) -> Closure<dyn FnMut(web_sys::Event)> {
    return Closure::wrap(Box::new(move |ev: web_sys::Event| {
        let ev_target = ev.target().unwrap();
        let target = ev_target.dyn_ref::<web_sys::Element>().unwrap();
        target.class_list().add_1("clicked").unwrap();
        tx.send(IoEvent::ButtonPressed(button)).ok();
    }) as Box<dyn FnMut(_)>);
}
fn key_release_cb(
    button: JoypadButton,
    tx: mpsc::Sender<IoEvent>,
) -> Closure<dyn FnMut(web_sys::Event)> {
    return Closure::wrap(Box::new(move |ev: web_sys::Event| {
        let ev_target = ev.target().unwrap();
        let target = ev_target.dyn_ref::<web_sys::Element>().unwrap();
        target.class_list().remove_1("clicked").unwrap();
        tx.send(IoEvent::ButtonReleased(button)).ok();
    }) as Box<dyn FnMut(_)>);
}
