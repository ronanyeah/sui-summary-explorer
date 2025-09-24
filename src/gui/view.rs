use iced::{
    widget::{
        button, checkbox, column, container, horizontal_space, row, scrollable, text, text_input,
    },
    Alignment, Background, Color, Element, Length,
};
use move_symbol_pool::symbol::Symbol;

use super::{type_to_string, DefType, Message, SearchItem, Selection, State, View};

pub fn view(state: &State) -> Element<'_, Message> {
    if state.packages.is_none() {
        return container(
            column![
                text("No package_summaries folder selected").size(24),
                button("Select Folder").on_press(Message::PickFolder)
            ]
            .spacing(20)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into();
    }

    let explorer_button = if state.view == View::Explorer {
        button(text("Explorer"))
            .on_press(Message::SetView(View::Explorer))
            .style(selected_button_style)
    } else {
        button(text("Explorer"))
            .on_press(Message::SetView(View::Explorer))
            .style(default_button_style)
    };

    let search_button = if state.view == View::Search {
        button(text("Search"))
            .on_press(Message::SetView(View::Search))
            .style(selected_button_style)
    } else {
        button(text("Search"))
            .on_press(Message::SetView(View::Search))
            .style(default_button_style)
    };

    let clear_button = button("Clear").on_press(Message::ClearPackages);

    let public_only_checkbox =
        checkbox("public only", state.public_only).on_toggle(Message::TogglePublicOnly);

    let view_buttons = row![
        row![explorer_button, search_button, public_only_checkbox].spacing(10),
        horizontal_space(),
        clear_button,
    ]
    .width(Length::Fill);

    let main_content: Element<Message> = match state.view {
        View::Explorer => {
            let packages_column = build_packages_column(state);
            let modules_column = build_modules_column(state);
            let definitions_column = build_definitions_column(state);
            let json_column = build_json_column(state);

            row![
                packages_column,
                modules_column,
                definitions_column,
                json_column
            ]
            .spacing(10)
            .into()
        }
        View::Search => build_search_view(state),
    };

    column![view_buttons, main_content]
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn build_packages_column(state: &State) -> Element<'_, Message> {
    let selected_addr = match &state.selection {
        Selection::PackageSelected(addr)
        | Selection::ModuleSelected(addr, _)
        | Selection::DefinitionSelected(addr, _, _, _) => Some(*addr),
        _ => None,
    };

    let package_buttons: Vec<_> = state
        .packages
        .as_ref()
        .expect("state.packages == None")
        .iter()
        .filter_map(|(addr, pkg)| {
            pkg.name.as_ref().map(|name| {
                let is_selected = Some(*addr) == selected_addr;
                let mut btn = button(text(name.to_string()))
                    .on_press(Message::SelectPackage(*addr))
                    .width(Length::Fill);

                if is_selected {
                    btn = btn.style(|_, _| button::Style {
                        background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                        text_color: Color::WHITE,
                        ..Default::default()
                    });
                }

                btn.into()
            })
        })
        .collect();

    column![
        text("Packages").size(16),
        scrollable(column(package_buttons).spacing(2)).height(Length::Fill)
    ]
    .width(Length::FillPortion(1))
    .spacing(5)
    .into()
}

fn build_modules_column(state: &State) -> Element<'_, Message> {
    let selected_module = match &state.selection {
        Selection::ModuleSelected(_, module_name)
        | Selection::DefinitionSelected(_, module_name, _, _) => Some(*module_name),
        _ => None,
    };

    let content = match &state.selection {
        Selection::PackageSelected(addr)
        | Selection::ModuleSelected(addr, _)
        | Selection::DefinitionSelected(addr, _, _, _) => {
            if let Some(package) = state.packages.as_ref().and_then(|p| p.get(addr)) {
                let module_buttons: Vec<_> = package
                    .modules
                    .keys()
                    .map(|module_name| {
                        let is_selected = Some(*module_name) == selected_module;
                        let mut btn = button(text(module_name.to_string()))
                            .on_press(Message::SelectModule(*module_name))
                            .width(Length::Fill);

                        if is_selected {
                            btn = btn.style(|_, _| button::Style {
                                background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                                text_color: Color::WHITE,
                                ..Default::default()
                            });
                        }

                        btn.into()
                    })
                    .collect();

                column![
                    text("Modules").size(16),
                    scrollable(column(module_buttons).spacing(2)).height(Length::Fill)
                ]
            } else {
                column![text("Modules").size(16), text("No package found").size(14)]
            }
        }
        Selection::NoSelection => {
            column![text("Modules").size(16), text("Select a package").size(14)]
        }
    };

    content.width(Length::FillPortion(1)).spacing(5).into()
}

fn build_definitions_column(state: &State) -> Element<'_, Message> {
    let selected_definition = match &state.selection {
        Selection::DefinitionSelected(_, _, def_type, def_name) => Some((*def_type, *def_name)),
        _ => None,
    };

    let content = match &state.selection {
        Selection::ModuleSelected(addr, module_name)
        | Selection::DefinitionSelected(addr, module_name, _, _) => {
            match state
                .packages
                .as_ref()
                .and_then(|p| p.get(addr))
                .and_then(|pkg| pkg.modules.get(module_name))
            {
                Some(module) => {
                    let definition_buttons =
                        build_definition_buttons(module, selected_definition, state.public_only);
                    column![
                        text("Definitions").size(16),
                        scrollable(column(definition_buttons).spacing(2))
                            .height(Length::Fill)
                            .width(Length::Fill)
                    ]
                    .width(Length::Fill)
                }
                None => column![
                    text("Definitions").size(16),
                    text("Module not found").size(14)
                ],
            }
        }
        _ => column![
            text("Definitions").size(16),
            text("Select a module").size(14)
        ],
    };

    content.width(Length::FillPortion(1)).spacing(5).into()
}

fn build_definition_buttons(
    module: &move_model_2::summary::Module,
    selected_definition: Option<(DefType, Symbol)>,
    public_only: bool,
) -> Vec<Element<'_, Message>> {
    let mut buttons = Vec::new();

    // Add functions
    module.functions.iter().for_each(|(name, fun)| {
        let is_selected = Some((DefType::Function, *name)) == selected_definition;
        let visibility = format!("{:?}", fun.visibility);

        if !public_only || visibility == "Public" {
            let mut btn = button(text(format!("fun {} [{}]", name, visibility)))
                .on_press(Message::SelectDefinition(DefType::Function, *name))
                .width(Length::Fill);

            if is_selected {
                btn = btn.style(|_, _| button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                    text_color: Color::WHITE,
                    ..Default::default()
                });
            }

            buttons.push(btn.into());
        }
    });

    // Add structs
    buttons.extend(module.structs.keys().map(|name| {
        let is_selected = Some((DefType::Struct, *name)) == selected_definition;
        let mut btn = button(text(format!("struct {}", name)))
            .on_press(Message::SelectDefinition(DefType::Struct, *name))
            .width(Length::Fill);

        if is_selected {
            btn = btn.style(|_, _| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                text_color: Color::WHITE,
                ..Default::default()
            });
        }

        btn.into()
    }));

    // Add enums
    buttons.extend(module.enums.keys().map(|name| {
        let is_selected = Some((DefType::Enum, *name)) == selected_definition;
        let mut btn = button(text(format!("enum {}", name)))
            .on_press(Message::SelectDefinition(DefType::Enum, *name))
            .width(Length::Fill);

        if is_selected {
            btn = btn.style(|_, _| button::Style {
                background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
                text_color: Color::WHITE,
                ..Default::default()
            });
        }

        btn.into()
    }));

    buttons
}

fn build_json_column(state: &State) -> Element<'_, Message> {
    let content = match &state.selection {
        Selection::DefinitionSelected(addr, module_name, def_type, def_name) => {
            match state
                .packages
                .as_ref()
                .and_then(|p| p.get(addr))
                .and_then(|pkg| pkg.modules.get(module_name))
            {
                Some(module) => {
                    let json_string = serialize_definition(module, def_type, def_name);
                    column![
                        text("Definition JSON").size(16),
                        scrollable(
                            container(text(json_string).size(16))
                                .padding(5)
                                .width(Length::Fill)
                        )
                        .height(Length::Fill)
                        .width(Length::Fill)
                    ]
                    .width(Length::Fill)
                }
                None => column![
                    text("Definition JSON").size(16),
                    text("Module not found").size(14)
                ],
            }
        }
        _ => column![
            text("Definition JSON").size(16),
            text("Select a definition").size(14)
        ],
    };

    content.width(Length::FillPortion(1)).spacing(5).into()
}

fn format_param(param: &move_model_2::summary::Parameter) -> String {
    // hack: Parameter fields are private
    let (name, v) = (|| -> Option<(String, String)> {
        let hack = serde_json::to_value(param).ok()?;
        let name = hack.get("name")?.as_str()?;
        let type_attr = hack.get("type_")?;
        let tpp: move_model_2::summary::Type = serde_json::from_value(type_attr.clone()).ok()?;
        Some((name.to_string(), type_to_string(&tpp)))
    })()
    .expect("failed to parse Parameter");
    format!("{}: {}", name, v)
}

fn build_function_signature(
    def_name: &Symbol,
    function: &move_model_2::summary::Function,
) -> String {
    let mut signature = String::new();
    signature.push_str("fun ");
    signature.push_str(&def_name.to_string());

    // Add type parameters if any
    if !function.type_parameters.is_empty() {
        signature.push('<');
        for (i, tparam) in function.type_parameters.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&tparam.name.expect("tparam.name not found"));
        }
        signature.push('>');
    }

    // Add parameters
    signature.push('(');
    for (i, param) in function.parameters.iter().enumerate() {
        if i > 0 {
            signature.push_str(",\n    ");
        } else {
            signature.push_str("\n    ");
        }
        signature.push_str(&format_param(param));
    }
    if !function.parameters.is_empty() {
        signature.push_str(",\n");
    }
    signature.push(')');

    // Add return type if any
    if !function.return_.is_empty() {
        signature.push_str(": ");
        if function.return_.len() == 1 {
            signature.push_str(&type_to_string(&function.return_[0]));
        } else {
            signature.push('(');
            for (i, ret_type) in function.return_.iter().enumerate() {
                if i > 0 {
                    signature.push_str(", ");
                }
                signature.push_str(&type_to_string(&ret_type));
            }
            signature.push(')');
        }
    }

    signature
}

fn build_struct_signature(def_name: &Symbol, struct_def: &move_model_2::summary::Struct) -> String {
    let mut signature = String::new();
    signature.push_str("struct ");
    signature.push_str(&def_name.to_string());

    // Add type parameters if any
    if !struct_def.type_parameters.is_empty() {
        signature.push('<');
        for (i, tparam) in struct_def.type_parameters.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&tparam.tparam.name.expect("tparam.name not found"));
        }
        signature.push('>');
    }

    // Add fields
    signature.push_str(" {\n");
    for (field_name, field) in &struct_def.fields.fields {
        signature.push_str(&format!(
            "    {}: {},\n",
            field_name,
            type_to_string(&field.type_)
        ));
    }
    signature.push('}');

    signature
}

fn build_enum_signature(def_name: &Symbol, enum_def: &move_model_2::summary::Enum) -> String {
    let mut signature = String::new();
    signature.push_str("enum ");
    signature.push_str(&def_name.to_string());

    // Add type parameters if any
    if !enum_def.type_parameters.is_empty() {
        signature.push('<');
        for (i, tparam) in enum_def.type_parameters.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&tparam.tparam.name.expect("tparam.name not found"));
        }
        signature.push('>');
    }

    // Add variants
    signature.push_str(" {\n");
    for (variant_name, variant) in &enum_def.variants {
        signature.push_str(&format!("    {}", variant_name));

        // Add variant fields if any
        if !variant.fields.fields.is_empty() {
            if variant.fields.positional_fields {
                // Positional fields: Variant(type1, type2, ...)
                signature.push('(');
                let field_types: Vec<String> = variant
                    .fields
                    .fields
                    .values()
                    .map(|field| type_to_string(&field.type_))
                    .collect();
                signature.push_str(&field_types.join(", "));
                signature.push(')');
            } else {
                // Named fields: Variant { name1: type1, name2: type2, ... }
                signature.push_str(" { ");
                let field_strs: Vec<String> = variant
                    .fields
                    .fields
                    .iter()
                    .map(|(name, field)| format!("{}: {}", name, type_to_string(&field.type_)))
                    .collect();
                signature.push_str(&field_strs.join(", "));
                signature.push_str(" }");
            }
        }
        signature.push_str(",\n");
    }
    signature.push('}');

    signature
}

fn serialize_definition(
    module: &move_model_2::summary::Module,
    def_type: &DefType,
    def_name: &Symbol,
) -> String {
    match def_type {
        DefType::Function => {
            if let Some(function) = module.functions.get(def_name) {
                let signature = build_function_signature(def_name, function);
                let json_val = serde_json::to_string_pretty(function)
                    .unwrap_or_else(|_| "Error serializing function".to_string());
                format!("{}\n\n{}", signature, json_val)
            } else {
                "Function not found".to_string()
            }
        }
        DefType::Struct => {
            if let Some(struct_def) = module.structs.get(def_name) {
                let signature = build_struct_signature(def_name, struct_def);
                let json_val = serde_json::to_string_pretty(struct_def)
                    .unwrap_or_else(|_| "Error serializing struct".to_string());
                format!("{}\n\n{}", signature, json_val)
            } else {
                "Struct not found".to_string()
            }
        }
        DefType::Enum => {
            if let Some(enum_def) = module.enums.get(def_name) {
                let signature = build_enum_signature(def_name, enum_def);
                let json_val = serde_json::to_string_pretty(enum_def)
                    .unwrap_or_else(|_| "Error serializing enum".to_string());
                format!("{}\n\n{}", signature, json_val)
            } else {
                "Enum not found".to_string()
            }
        }
    }
}

fn selected_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.3, 0.6, 0.9))),
        text_color: Color::WHITE,
        ..Default::default()
    }
}

fn default_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        text_color: Color::WHITE,
        ..Default::default()
    }
}

fn build_search_view(state: &State) -> Element<'_, Message> {
    let search_input = text_input("Search modules and definitions...", &state.search_input)
        .on_input(Message::SearchInputChanged)
        .padding(10)
        .width(Length::Fill);

    let std_checkbox = checkbox("std", state.std_filter).on_toggle(Message::ToggleStdFilter);

    let sui_checkbox = checkbox("sui", state.sui_filter).on_toggle(Message::ToggleSuiFilter);

    let search_row = row![search_input, std_checkbox, sui_checkbox].spacing(10);

    let mut items = Vec::new();
    let search_queries: Vec<String> = if state.search_input.trim().is_empty() {
        vec![]
    } else {
        state
            .search_input
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let matches_search = |name: &str| -> bool {
        if search_queries.is_empty() {
            true
        } else {
            let name_lower = name.to_lowercase();
            search_queries
                .iter()
                .all(|query| name_lower.contains(query))
        }
    };

    if let Some(packages) = &state.packages {
        for (package_addr, package) in packages {
            if let Some(package_name) = &package.name {
                if &package_name.to_string() == "sui" && !state.sui_filter {
                    continue;
                }
                if &package_name.to_string() == "std" && !state.std_filter {
                    continue;
                }
                for (module_name, module) in &package.modules {
                    if matches_search(&module_name.to_string()) {
                        items.push(SearchItem::Module {
                            package_addr: *package_addr,
                            module_name: *module_name,
                            display: format!("Module: {}.{}", package_name, module_name),
                        });
                    }

                    for function_name in module.functions.keys() {
                        if matches_search(&function_name.to_string()) {
                            items.push(SearchItem::Definition {
                                package_addr: *package_addr,
                                module_name: *module_name,
                                def_type: DefType::Function,
                                def_name: *function_name,
                                display: format!(
                                    "  Function: {}.{}.{}",
                                    package_name, module_name, function_name
                                ),
                            });
                        }
                    }

                    for struct_name in module.structs.keys() {
                        if matches_search(&struct_name.to_string()) {
                            items.push(SearchItem::Definition {
                                package_addr: *package_addr,
                                module_name: *module_name,
                                def_type: DefType::Struct,
                                def_name: *struct_name,
                                display: format!(
                                    "  Struct: {}.{}.{}",
                                    package_name, module_name, struct_name
                                ),
                            });
                        }
                    }

                    for enum_name in module.enums.keys() {
                        if matches_search(&enum_name.to_string()) {
                            items.push(SearchItem::Definition {
                                package_addr: *package_addr,
                                module_name: *module_name,
                                def_type: DefType::Enum,
                                def_name: *enum_name,
                                display: format!(
                                    "  Enum: {}.{}.{}",
                                    package_name, module_name, enum_name
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    let items_list: Vec<Element<Message>> = items
        .into_iter()
        .map(|item| match item {
            SearchItem::Module {
                module_name,
                display,
                package_addr,
                ..
            } => button(text(display).size(12))
                .on_press(Message::SelectFromSearch(package_addr, module_name, None))
                .style(default_button_style)
                .width(Length::Fill)
                .into(),
            SearchItem::Definition {
                module_name,
                def_type,
                def_name,
                display,
                package_addr,
                ..
            } => button(text(display).size(12))
                .on_press(Message::SelectFromSearch(
                    package_addr,
                    module_name,
                    Some((def_type, def_name)),
                ))
                .style(default_button_style)
                .width(Length::Fill)
                .into(),
        })
        .collect();

    column![
        search_row,
        scrollable(column(items_list).spacing(2))
            .height(Length::Fill)
            .width(Length::Fill)
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
