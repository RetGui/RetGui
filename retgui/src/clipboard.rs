use std::cell::RefCell;
use std::collections::VecDeque;

use winit::data_transfer::{DataTransferSendBuilder, TypeHint};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, AsyncRequestSerial};

use crate::text::text_context::TextContext;

type Paste = Box<dyn FnOnce(String, &mut TextContext)>;

enum Request {
    Copy(String),
    Paste(Paste),
}

thread_local! {
    static REQUESTS: RefCell<VecDeque<Request>> = RefCell::default();
}

pub(crate) fn copy(text: String) -> bool {
    if !cfg!(all(
        feature = "clipboard",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    )) {
        return false;
    }
    REQUESTS.with_borrow_mut(|queue| queue.push_back(Request::Copy(text)));
    true
}

pub(crate) fn paste(callback: Paste) {
    if cfg!(feature = "clipboard") {
        REQUESTS.with_borrow_mut(|queue| queue.push_back(Request::Paste(callback)));
    }
}

#[derive(Default)]
pub(crate) struct Clipboard {
    pending: Vec<(AsyncRequestSerial, Paste)>,
    headless: String,
}

impl Clipboard {
    pub(crate) fn process(&mut self, event_loop: Option<&dyn ActiveEventLoop>, context: &mut TextContext) {
        let requests = REQUESTS.with_borrow_mut(std::mem::take);
        for request in requests {
            match (event_loop, request) {
                (Some(event_loop), Request::Copy(text)) => {
                    let data = DataTransferSendBuilder::new(text)
                        .with_type(TypeHint::Plaintext, |text, _| Some(text.clone()))
                        .build();
                    if let Err(error) = event_loop.set_clipboard(data) {
                        retgui_logging::warn!("Clipboard copy failed: {error}");
                    }
                }
                (Some(event_loop), Request::Paste(callback)) => {
                    if let Ok(Some(id)) = event_loop.clipboard()
                        && let Ok(serial) = event_loop.fetch_data_transfer(id, &TypeHint::Plaintext)
                    {
                        self.pending.push((serial, callback));
                    }
                }
                (None, Request::Copy(text)) => self.headless = text,
                (None, Request::Paste(callback)) => callback(self.headless.clone(), context),
            }
        }
    }

    pub(crate) fn receive(&mut self, event: &WindowEvent, context: &mut TextContext) {
        let WindowEvent::DataTransferReceived { serial, value, .. } = event else {
            return;
        };
        let Some(index) = self.pending.iter().position(|(request, _)| request == serial) else {
            return;
        };
        match value.try_as_string() {
            Ok(text) => {
                let (_, callback) = self.pending.remove(index);
                callback(text, context);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => {
                let _ = self.pending.remove(index);
            }
        }
    }

    pub(crate) fn headless_text(&self) -> &str {
        &self.headless
    }
}
