use zellij_tile::prelude::*;

use std::path::{Component, PathBuf};

#[derive(Default, Debug)]
pub struct RunAndReload {
    cwd: Option<PathBuf>,
    compilation_pane_id: Option<u32>,
}

impl RunAndReload {
    pub fn update_cwd(&mut self, cwd: PathBuf) {
        self.cwd = Some(cwd);
    }
    pub fn get_cwd(&self) -> Option<PathBuf> {
        self.cwd.clone()
    }
    pub fn update_compilation_pane_id(&mut self, compilation_pane_id: u32) {
        self.compilation_pane_id = Some(compilation_pane_id);
    }
    pub fn command_pane_exited(&mut self, exit_code: Option<i32>, terminal_pane_id: u32) {
        if Some(terminal_pane_id) == self.compilation_pane_id && exit_code == Some(0) {
            self.start_or_reload_plugin();
            if exit_code == Some(0) {
                hide_pane_with_id(PaneId::Terminal(terminal_pane_id));
            }
        }
    }
    pub fn pane_closed(&mut self, pane_id: PaneId) {
        if let PaneId::Terminal(terminal_pane_id) = pane_id {
            if Some(terminal_pane_id) == self.compilation_pane_id {
                self.compilation_pane_id = None;
            }
        }
    }
    pub fn run_compilation(&self) {
        if let Some(compilation_pane_id) = self.compilation_pane_id {
            show_pane_with_id(PaneId::Terminal(compilation_pane_id), true);
            rerun_command_pane(compilation_pane_id);
            hide_self();
        } else {
            let mut command = CommandToRun::new_with_args("cargo", vec!["build"]);
            if let Some(cwd) = &self.cwd {
                command.cwd = Some(cwd.clone());
                hide_self();
                open_command_pane_floating(command, None, Default::default());
            } else {
                eprintln!("no cwd found!");
            }
        }
    }
    pub fn start_or_reload_plugin(&self) {
        if let Some(cwd) = &self.cwd {
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
                    start_or_reload_plugin(&format!("file:{}", plugin_path.display()));
                }
            }
        } else {
            eprintln!("no cwd found!");
        }
    }
}

// // update_workspace.rs
// #[derive(Debug)]
// pub struct UpdateWorkspace {
//     base_mode: Option<InputMode>,
//     own_plugin_id: Option<u32>,
//     own_tab_index: Option<usize>,
//     own_tab_is_active: bool,
//     bound_key: bool,
//     reload_shortcut: KeyWithModifier,
//     tab_infos: Vec<TabInfo>,
//     pane_manifest: PaneManifest,
//     plugin_pane_id: Option<PaneId>, // this is the plugin we're reloading
//     renamed_plugin_pane: bool,
// }
//
// impl UpdateWorkspace {
//     pub fn new(reload_shortcut: KeyWithModifier) -> Self {
//         UpdateWorkspace {
//             reload_shortcut,
//             base_mode: Default::default(),
//             own_plugin_id: Default::default(),
//             own_tab_index: Default::default(),
//             own_tab_is_active: Default::default(),
//             bound_key: Default::default(),
//             tab_infos: Default::default(),
//             pane_manifest: Default::default(),
//             plugin_pane_id: Default::default(),
//             renamed_plugin_pane: Default::default(),
//         }
//     }
//     pub fn update_own_plugin_id(&mut self, plugin_id: u32) {
//         self.own_plugin_id = Some(plugin_id);
//     }
//     pub fn get_own_plugin_id(&mut self) -> Option<u32> {
//         self.own_plugin_id
//     }
//     pub fn update_pane_manifest(&mut self, pane_manifest: PaneManifest) {
//         self.pane_manifest = pane_manifest;
//     }
//     pub fn update_tab_infos(&mut self, tab_infos: Vec<TabInfo>) {
//         self.tab_infos = tab_infos;
//     }
//     pub fn update_base_mode(&mut self, base_mode: InputMode) {
//         self.base_mode = Some(base_mode);
//     }
//     pub fn pane_closed(&self, pane_id: PaneId) {
//         if Some(pane_id) == self.plugin_pane_id {
//             // if the plugin we're reloading closed, we close ourselves
//             close_self();
//         }
//     }
//     pub fn get_reload_shortcut(&self) -> &KeyWithModifier {
//         &self.reload_shortcut
//     }
//     fn bind_key_if_not_bound_and_tab_is_focused(&mut self) {
//         self.update_own_tab_index();
//         self.update_own_tab_is_active();
//         match (self.base_mode, self.own_plugin_id) {
//             (Some(base_mode), Some(own_plugin_id)) if self.own_tab_is_active && !self.bound_key => {
//                 bind_key(base_mode, own_plugin_id, &self.reload_shortcut);
//                 self.bound_key = true;
//             }
//             _ => {}
//         }
//     }
//     fn update_own_tab_is_active(&mut self) {
//         if let Some(own_tab) = self.tab_infos.iter().find(|t| Some(t.position) == self.own_tab_index) {
//             self.own_tab_is_active = own_tab.active;
//             if !own_tab.active {
//                 // this is so that when we our tab loses focus, we'll rebind the key once it gains
//                 // it
//                 self.bound_key = false;
//             }
//         }
//     }
//     fn update_own_tab_index(&mut self) {
//         if let Some(own_tab_index) = self.own_plugin_id.and_then(|own_pane_id| {
//             get_tab_index_of_pane(own_pane_id, &self.pane_manifest)
//         }) {
//             self.own_tab_index = Some(own_tab_index);
//         }
//     }
//     fn update_plugin_pane_id(&mut self, plugin_url: Option<String>) {
//         if self.plugin_pane_id.is_none() {
//             for (_tab_index, panes) in &self.pane_manifest.panes {
//                 for pane in panes {
//                     if pane.plugin_url == plugin_url {
//                         self.plugin_pane_id = Some(PaneId::Plugin(pane.id));
//                         return;
//                     }
//                 }
//             }
//         }
//     }
//     fn rename_plugin_pane_if_needed(&mut self, plugin_name: Option<String>) {
//         if !self.renamed_plugin_pane {
//             if let Some(plugin_pane_id) = self.plugin_pane_id.as_ref() {
//                 if let Some(plugin_name) = plugin_name {
//                     rename_pane_with_id(*plugin_pane_id, format!("{} (<Ctrl-Shift r> to rebuild)", plugin_name));
//                     self.renamed_plugin_pane = true;
//                 }
//             }
//         }
//     }
//
// }
