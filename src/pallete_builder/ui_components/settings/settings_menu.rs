use sfml::{graphics::RenderWindow, window::Event as SFMLEvent};
use ui::{
    dom_controller::{DomController, DomControllerInterface},
    events::Event,
    ui_settings::UISettings,
};
use utils::resource_manager::ResourceManager;

use crate::pallete_builder::color_grid::load_save::list_of_files_with_pacrb_extension;

use self::{
    confirm_file_deletion::ConfirmFileDeletion,
    settings_menu_content::{perform_events, reload_list_of_files, sync_events},
};

mod confirm_file_deletion;
mod settings_menu_content;

#[derive(Debug, Default)]
pub struct SettingsMenu {
    file_to_load: Option<String>,
    list_of_files: Vec<String>,
    current_list_of_files_idx: usize,
    settings_menu_dom: DomController,
    pub display: bool,
    confirm_file_deletion: ConfirmFileDeletion,
}

impl SettingsMenu {
    pub fn new(resource_manager: &ResourceManager, ui_settings: &UISettings) -> Self {
        let mut settings_menu_dom = DomController::new(
            resource_manager,
            ui_settings,
            include_str!("settings_menu/settings_menu_content.xml"),
        );
        let list_of_files = list_of_files_with_pacrb_extension();
        sync_events(&mut settings_menu_dom, ui_settings);
        let mut sm = Self {
            file_to_load: None,
            settings_menu_dom,
            current_list_of_files_idx: 0,
            display: false,
            list_of_files,
            confirm_file_deletion: ConfirmFileDeletion::new(resource_manager, ui_settings),
        };
        reload_list_of_files(&mut sm);

        sm
    }

    pub fn file_to_load(&self) -> Option<&str> {
        self.file_to_load.as_deref()
    }

    pub fn clear_file_to_load(&mut self) {
        self.file_to_load = None;
    }
}

impl DomControllerInterface for SettingsMenu {
    fn event_handler(
        &mut self,
        window: &mut RenderWindow,
        ui_settings: &mut UISettings,
        event: SFMLEvent,
    ) -> Vec<Event> {
        if !self.display && !matches!(event, SFMLEvent::Resized { .. }) {
            return Default::default();
        }

        let mut events = self
            .confirm_file_deletion
            .event_handler(window, ui_settings, event);
        if !events.is_empty() || self.confirm_file_deletion.is_displaying() {
            match self.confirm_file_deletion.file_deletion_selection() {
                confirm_file_deletion::ConfirmFileDeletionSelection::Delete
                | confirm_file_deletion::ConfirmFileDeletionSelection::Cancel => {
                    self.confirm_file_deletion.set_display(false);
                    self.confirm_file_deletion.set_file_to_delete("");
                    self.current_list_of_files_idx = 0;
                    self.list_of_files = list_of_files_with_pacrb_extension();
                    reload_list_of_files(self);
                }
                confirm_file_deletion::ConfirmFileDeletionSelection::None => {}
            }
            return events;
        }

        events.append(
            &mut self
                .settings_menu_dom
                .event_handler(window, ui_settings, event),
        );
        perform_events(&events, window, ui_settings, self);
        events
    }

    fn render(&mut self, window: &mut RenderWindow) {
        if !self.display {
            return;
        }
        self.settings_menu_dom.render(window);
        self.confirm_file_deletion.render(window);
    }

    fn update(&mut self, resource_manager: &ResourceManager) -> Vec<Event> {
        if !self.display {
            return Default::default();
        }
        let mut events = self.settings_menu_dom.update(resource_manager);
        events.append(&mut self.confirm_file_deletion.update(resource_manager));
        events
    }
}
