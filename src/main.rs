mod run_and_reload;
mod update_workspace;

use run_and_reload::RunAndReload;
use update_workspace::UpdateWorkspace;

use zellij_tile::prelude::*;

use std::collections::BTreeMap;
use std::path::{Component, PathBuf};
use uuid::Uuid;

struct State {
    run_and_reload: RunAndReload,
    update_workspace: UpdateWorkspace,
    filepicker_request_ids: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        let reload_shortcut = KeyWithModifier::new(BareKey::Char('r'))
            .with_ctrl_modifier()
            .with_shift_modifier();
        State {
            run_and_reload: Default::default(),
            update_workspace: UpdateWorkspace::new(reload_shortcut),
            filepicker_request_ids: Default::default(),
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::RunCommands,
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::Reconfigure,
            PermissionType::ChangeApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
        ]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::Key,
            EventType::CommandPaneOpened,
            EventType::CommandPaneExited,
            EventType::PaneUpdate,
            EventType::PaneClosed,
            EventType::PermissionRequestResult,
        ]);
        let plugin_ids = get_plugin_ids();
        if let Some(reload_shortcut) = configuration.get("reload_shortcut") {
            self.update_workspace.update_reload_shortcut(reload_shortcut);
        }
        self.update_workspace
            .update_own_plugin_id(plugin_ids.plugin_id);
        self.run_and_reload.update_cwd(plugin_ids.initial_cwd);
    }
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        if pipe_message.is_private && pipe_message.name == "recompile" {
            self.run_and_reload.run_compilation();
        } else if pipe_message.name == "filepicker_result" {
            should_render = self.handle_filepicker_result(pipe_message);
        }
        should_render
    }
    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::PermissionRequestResult(_) => {
                if let Some(plugin_id) = self.update_workspace.get_own_plugin_id() {
                    rename_pane_with_id(
                        PaneId::Plugin(plugin_id),
                        format!("Develop Zellij Plugin"),
                    );
                }
            }
            Event::PaneUpdate(pane_manifest) => {
                self.update_workspace.update_pane_manifest(pane_manifest);
                self.update_workspace
                    .bind_key_if_not_bound_and_tab_is_focused();
                self.update_workspace
                    .update_plugin_pane_id(self.plugin_url());
                self.update_workspace
                    .rename_plugin_pane_if_needed(self.plugin_name());
            }
            Event::TabUpdate(tab_infos) => {
                self.update_workspace.update_tab_infos(tab_infos);
                self.update_workspace
                    .bind_key_if_not_bound_and_tab_is_focused();
            }
            Event::Key(key) => match key.bare_key {
                BareKey::Char('f') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                    self.send_filepicker_request();
                }
                _ => {}
            },
            Event::ModeUpdate(mode_info) => {
                if let Some(base_mode) = mode_info.base_mode {
                    self.update_workspace.update_base_mode(base_mode);
                    self.update_workspace
                        .bind_key_if_not_bound_and_tab_is_focused();
                }
            }
            Event::CommandPaneOpened(terminal_pane_id, _context) => {
                self.run_and_reload
                    .update_compilation_pane_id(terminal_pane_id);
                should_render = true;
            }
            Event::CommandPaneExited(terminal_pane_id, exit_code, _context) => {
                self.run_and_reload
                    .command_pane_exited(exit_code, terminal_pane_id);
            }
            Event::PaneClosed(pane_id) => {
                self.run_and_reload.pane_closed(pane_id);
                self.update_workspace.pane_closed(pane_id);
            }
            _ => {}
        }
        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let reload_shortcut = format!("{}", self.update_workspace.get_reload_shortcut());
        let current_folder = format!(
            "{}",
            self.run_and_reload
                .get_cwd()
                .map(|f| f.display().to_string())
                .unwrap_or_else(|| "<NOT SET>".to_owned())
        );
        let title_text = "Develop Zellij Plugin";
        let explanation_text_1 = "This plugin will help you develop a Zellij plugin in Rust.";
        let explanation_text_2 = format!("Press <{}> to:", reload_shortcut);
        let bulletin_1 = format!("1. Run cargo build");
        let bulletin_2 = format!("2. Load or Reload the plugin");
        let explanation_text_3 = format!("Closing the plugin window will close this plugin.");
        let folder_text = format!("Current Folder: {} <Ctrl f> to change", current_folder);

        let longest_line_length = std::cmp::max(
            folder_text.chars().count(),
            explanation_text_1.chars().count(),
        );
        let centered_x = cols.saturating_sub(longest_line_length) / 2;
        let centered_y = rows.saturating_sub(10) / 2;

        let title_text = Text::new(title_text).color_range(2, ..);
        let explanation_text_1 = Text::new(explanation_text_1);
        let explanation_text_2 =
            Text::new(explanation_text_2).color_range(3, 6..=7 + reload_shortcut.chars().count());
        let bulletin_1 = Text::new(bulletin_1).color_range(0, 7..=17);
        let bulletin_2 = Text::new(bulletin_2);
        let explanation_text_3 = Text::new(explanation_text_3);
        let folder_text = Text::new(folder_text)
            .color_range(0, 16..=16 + current_folder.chars().count())
            .color_range(
                3,
                16 + current_folder.chars().count() + 1..=16 + current_folder.chars().count() + 9,
            );

        print_text_with_coordinates(title_text, centered_x, centered_y, None, None);
        print_text_with_coordinates(explanation_text_1, centered_x, centered_y + 1, None, None);
        print_text_with_coordinates(explanation_text_2, centered_x, centered_y + 3, None, None);
        print_text_with_coordinates(bulletin_1, centered_x + 2, centered_y + 4, None, None);
        print_text_with_coordinates(bulletin_2, centered_x + 2, centered_y + 5, None, None);
        print_text_with_coordinates(explanation_text_3, centered_x, centered_y + 7, None, None);
        print_text_with_coordinates(folder_text, centered_x, centered_y + 9, None, None);
    }
}

impl State {
    fn plugin_url(&self) -> Option<String> {
        if let Some(cwd) = &self.run_and_reload.get_cwd() {
            if let Some(Component::Normal(project_dir_name)) = cwd.components().last() {
                if let Some(project_dir_name) = project_dir_name.to_str() {
                    let mut plugin_path = cwd.clone();
                    plugin_path.extend(
                        PathBuf::from(format!(
                            "target/wasm32-wasi/debug/{}.wasm",
                            project_dir_name.to_string()
                        ))
                        .components(),
                    );
                    return Some(format!("file:{}", plugin_path.display()));
                }
            }
        }
        None
    }
    fn plugin_name(&self) -> Option<String> {
        if let Some(cwd) = &self.run_and_reload.get_cwd() {
            if let Some(Component::Normal(project_dir_name)) = cwd.components().last() {
                if let Some(project_dir_name) = project_dir_name.to_str() {
                    return Some(project_dir_name.to_owned());
                }
            }
        }
        None
    }
    pub fn send_filepicker_request(&mut self) {
        let mut args = BTreeMap::new();
        let request_id = Uuid::new_v4();
        self.filepicker_request_ids.push(request_id.to_string());
        let mut config = BTreeMap::new();
        config.insert("request_id".to_owned(), request_id.to_string());
        args.insert("request_id".to_owned(), request_id.to_string());
        pipe_message_to_plugin(
            MessageToPlugin::new("filepicker")
                .with_plugin_url("filepicker")
                .with_plugin_config(config)
                .new_plugin_instance_should_have_pane_title(
                    "Select a a folder in which to develop your Zellij plugin...",
                )
                .with_args(args),
        );
    }
    fn handle_filepicker_result(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        match (pipe_message.payload, pipe_message.args.get("request_id")) {
            (Some(payload), Some(request_id)) => {
                match self
                    .filepicker_request_ids
                    .iter()
                    .position(|p| p == request_id)
                {
                    Some(request_id_position) => {
                        self.filepicker_request_ids.remove(request_id_position);
                        let chosen_plugin_location = std::path::PathBuf::from(payload);
                        self.run_and_reload.update_cwd(chosen_plugin_location);
                        should_render = true;
                    }
                    None => {
                        eprintln!("request id not found");
                    }
                }
            }
            _ => {}
        }
        should_render
    }
}
