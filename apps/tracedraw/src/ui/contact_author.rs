//! The author's contact card, embedded unchanged and decoded only on demand.

use crate::i18n::tr;
use egui::{Context, TextureHandle};

const CARD: &[u8] = include_bytes!("../../assets/author-contact.jpg");

fn texture(ctx: &Context) -> Option<TextureHandle> {
    let id = egui::Id::new("rui_author_contact_texture");
    if let Some(texture) = ctx.data(|data| data.get_temp::<TextureHandle>(id)) {
        return Some(texture);
    }
    let image = match image::load_from_memory(CARD) {
        Ok(image) => image.to_rgba8(),
        Err(error) => {
            log::warn!("could not decode the author contact card: {error}");
            return None;
        }
    };
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize], image.as_raw(),
    );
    let texture = ctx.load_texture("rui_author_contact", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    Some(texture)
}

pub fn show(ctx: &Context, close: &mut bool) {
    let texture = texture(ctx);
    let screen = ctx.content_rect();
    crate::ui::chrome::window(tr("contact.title")).show(ctx, |ui| {
        let height = (screen.height() - 180.0).clamp(120.0, 620.0);
        let width = (screen.width() - 80.0).clamp(120.0, 470.0);
        ui.set_width(width);
        ui.label(tr("contact.hint"));
        ui.add_space(8.0);
        if let Some(texture) = texture.as_ref() {
            let size = texture.size_vec2();
            let scale = (width / size.x).min(height / size.y);
            ui.vertical_centered(|ui| {
                ui.image((texture.id(), size * scale));
            });
        } else {
            ui.label(tr("contact.unavailable"));
        }
        ui.add_space(8.0);
        if crate::ui::chrome::button_row(ui, &[&tr("dialog.close")], false).is_some()
            || crate::ui::dialogs::enter_pressed(ui)
        {
            *close = true;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::App, ui::dialogs::Dialog};
    use tracedraw_core::Command;

    #[test]
    fn the_original_contact_card_is_valid_and_cached() {
        let ctx = crate::theme::ui_context();
        let first = texture(&ctx).unwrap();
        assert_eq!(first.size(), [1110, 1491]);
        assert_eq!(first.id(), texture(&ctx).unwrap().id());
    }

    #[test]
    fn contact_opens_without_editing_the_drawing() {
        let mut app = App::headless();
        app.engine.mark_saved();
        let before = serde_json::to_vec(app.engine.document()).unwrap();
        let revision = app.engine.revision();
        app.run(Command::ContactAuthor);
        assert!(matches!(app.dialog, Dialog::ContactAuthor));
        assert!(!app.engine.is_dirty());
        assert_eq!(revision, app.engine.revision());
        assert_eq!(before, serde_json::to_vec(app.engine.document()).unwrap());
    }
}
