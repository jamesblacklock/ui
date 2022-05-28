use std::panic;
use std::sync::Once;

#[inline]
pub fn set_once() {
    fn hook(info: &panic::PanicInfo) {
        super::throw_error(info.to_string());
    }

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook));
    });
}