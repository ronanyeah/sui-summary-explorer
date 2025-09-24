use super::{Message, Selection, State, View};

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::SelectPackage(package_addr) => {
            state.selection = Selection::PackageSelected(package_addr);
        }
        Message::SelectModule(module_name) => match &state.selection {
            Selection::PackageSelected(package_addr) => {
                state.selection = Selection::ModuleSelected(*package_addr, module_name);
            }
            Selection::ModuleSelected(package_addr, _) => {
                state.selection = Selection::ModuleSelected(*package_addr, module_name);
            }
            Selection::DefinitionSelected(package_addr, _, _, _) => {
                state.selection = Selection::ModuleSelected(*package_addr, module_name);
            }
            _ => {}
        },
        Message::SelectDefinition(def_type, definition_name) => match &state.selection {
            Selection::ModuleSelected(package_addr, module_name) => {
                state.selection = Selection::DefinitionSelected(
                    *package_addr,
                    *module_name,
                    def_type,
                    definition_name,
                );
            }
            Selection::DefinitionSelected(package_addr, module_name, _, _) => {
                state.selection = Selection::DefinitionSelected(
                    *package_addr,
                    *module_name,
                    def_type,
                    definition_name,
                );
            }
            _ => {}
        },
        Message::SetView(view) => {
            state.view = view;
        }
        Message::SearchInputChanged(input) => {
            state.search_input = input;
        }
        Message::ToggleStdFilter(enabled) => {
            state.std_filter = enabled;
        }
        Message::ToggleSuiFilter(enabled) => {
            state.sui_filter = enabled;
        }
        Message::TogglePublicOnly(enabled) => {
            state.public_only = enabled;
        }
        Message::SelectFromSearch(package_addr, module_name, definition) => {
            state.view = View::Explorer;
            match definition {
                None => {
                    state.selection = Selection::ModuleSelected(package_addr, module_name);
                }
                Some((def_type, def_name)) => {
                    state.selection = Selection::DefinitionSelected(
                        package_addr,
                        module_name,
                        def_type,
                        def_name,
                    );
                }
            }
        }
        Message::PickFolder => {
            let current_dir = std::env::current_dir().expect("current_dir not found");
            let folder = rfd::FileDialog::new()
                .set_directory(current_dir)
                .pick_folder();
            if let Some(path) = folder {
                match crate::parser::parse_summaries(&path) {
                    Ok(summary) => {
                        state.packages = Some(summary.packages);
                        state.selection = Selection::NoSelection;
                    }
                    Err(_) => {
                        eprintln!("Invalid folder!")
                    }
                }
            }
        }
        Message::ClearPackages => {
            state.packages = None;
            state.selection = Selection::NoSelection;
        }
    }
}
