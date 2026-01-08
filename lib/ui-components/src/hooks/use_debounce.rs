//! Debounce utilities untuk optimasi input
//!
//! Note: Use these in Effect::new() with your signals

use gloo::timers::callback::Timeout;

/// Create a debounced function
/// Returns a function that delays execution
pub fn create_debounced<F>(func: F, delay_ms: u32) -> impl Fn()
where
    F: Fn() + Clone + 'static,
{
    use std::cell::RefCell;
    use std::rc::Rc;

    let timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));
    let func = Rc::new(func);

    move || {
        // Cancel existing timeout
        if let Some(t) = timeout.borrow_mut().take() {
            t.forget();
        }

        // Set new timeout
        let timeout_clone = Rc::clone(&timeout);
        let func_clone = Rc::clone(&func);
        let new_timeout = Timeout::new(delay_ms, move || {
            func_clone();
            *timeout_clone.borrow_mut() = None;
        });

        *timeout.borrow_mut() = Some(new_timeout);
    }
}
