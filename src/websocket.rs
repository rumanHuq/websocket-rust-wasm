use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::{
  window, Document, ErrorEvent, HtmlElement, HtmlInputElement, KeyboardEvent, MessageEvent,
  WebSocket,
};

macro_rules! console_log {
    ($($t:tt)*) => (console::log_1(&format!($($t)*).into()))
}

pub fn handle_on_send_message(ws: Rc<WebSocket>, document: &Document) {
  let input_text = Rc::new(RefCell::new(String::new()));
  let input_text_to_be_sent = Rc::new(Rc::clone(&input_text));

  let save_message_callback = Box::new(move |e: KeyboardEvent| {
    let strs = &e
      .target()
      .unwrap()
      .dyn_ref::<HtmlInputElement>()
      .unwrap()
      .value();
    *input_text.borrow_mut() = strs.into();
  });
  let save_message = Closure::wrap(save_message_callback as Box<dyn Fn(KeyboardEvent)>);
  if let Some(input) = document.query_selector("input").unwrap() {
    input
      .dyn_ref::<HtmlInputElement>()
      .unwrap()
      .set_onkeyup(Some(save_message.as_ref().unchecked_ref()));
  };
  save_message.forget();

  let send_message_callback = Box::new(move || {
    ws.send_with_str(&*input_text_to_be_sent.borrow()).unwrap();
  });
  let send_message = Closure::wrap(send_message_callback as Box<dyn Fn()>);
  if let Some(button) = document.query_selector("button").unwrap() {
    button
      .dyn_ref::<HtmlElement>()
      .unwrap()
      .set_onclick(Some(send_message.as_ref().unchecked_ref()));
  }
  send_message.forget();
}

fn handle_on_recieve_message(ws: Rc<WebSocket>) {
  let onmessage_callback = Box::new(move |e: MessageEvent| {
    let response = e
      .data()
      .as_string()
      .expect("Can't convert received data to a string");
    console_log!("message event, received data: {:?}", response);
  });
  let onmessage = Closure::wrap(onmessage_callback as Box<dyn Fn(MessageEvent)>);
  ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
  onmessage.forget();
}

fn handle_on_error(ws: Rc<WebSocket>) {
  let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
    console_log!("error event: {:?}", e);
  }) as Box<dyn Fn(ErrorEvent)>);
  ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
  onerror_callback.forget();
}

fn handle_on_open_connection(ws: Rc<WebSocket>) {
  let cloned_ws_callback = Rc::clone(&ws);
  let onopen_callback = Closure::wrap(Box::new(move |_| {
    console_log!("socket opened");
    match cloned_ws_callback.send_with_str("ping") {
      Ok(_) => console_log!("message successfully sent"),
      Err(err) => console_log!("error sending message: {:?}", err),
    }
  }) as Box<dyn Fn(JsValue)>);
  ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
  onopen_callback.forget();
}

pub fn websocket() {
  let ws = WebSocket::new("ws://127.0.0.1:3012").unwrap();
  let window = window().unwrap();
  let document = window.document().unwrap();
  let ws = Rc::new(ws);
  handle_on_open_connection(Rc::clone(&ws));
  handle_on_send_message(Rc::clone(&ws), &document);
  handle_on_recieve_message(Rc::clone(&ws));
  handle_on_error(Rc::clone(&ws));
}
