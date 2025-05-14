/**
 * @file notify.rs
 * @author Nguyen Le Duy
 * @date 11/05/2025
 * @brief Notification system for the simulator
 */
use egui::WidgetText;
use egui_toast::{Toast, ToastKind, Toasts};
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::Duration;

pub fn get_toasts() -> MutexGuard<'static, Toasts> {
    // Notification system
    pub static TOASTS: LazyLock<Mutex<Toasts>> = LazyLock::new(|| {
        Mutex::new(
            Toasts::new()
                .anchor(egui::Align2::RIGHT_TOP, (10.0, 10.0))
                .direction(egui::Direction::TopDown),
        )
    });

    TOASTS.lock().unwrap()
}

fn toast() -> Toast {
    Toast::new().options(
        egui_toast::ToastOptions::default()
            .duration(Duration::from_secs(15))
            .show_progress(true),
    )
}

pub fn success(message: impl Into<WidgetText>) {
    get_toasts().add(toast().text(message).kind(ToastKind::Success));
}

pub fn info(message: impl Into<WidgetText>) {
    get_toasts().add(toast().text(message).kind(ToastKind::Info));
}

pub fn error(message: impl Into<WidgetText>) {
    get_toasts().add(toast().text(message).kind(ToastKind::Error));
}

pub fn warning(message: impl Into<WidgetText>) {
    get_toasts().add(toast().text(message).kind(ToastKind::Warning));
}
