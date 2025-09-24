mod update;
mod view;

use iced::{Color, Subscription, Task};
use move_core_types::account_address::AccountAddress;
use move_model_2::summary::{Package, Type};
use move_symbol_pool::symbol::Symbol;
use std::collections::BTreeMap;
use std::fmt::Debug;
use update::update;
use view::view;

#[derive(Debug, Clone)]
enum Message {
    SelectPackage(AccountAddress),
    SelectModule(Symbol),
    SelectDefinition(DefType, Symbol),
    SetView(View),
    SearchInputChanged(String),
    ToggleStdFilter(bool),
    ToggleSuiFilter(bool),
    TogglePublicOnly(bool),
    SelectFromSearch(AccountAddress, Symbol, Option<(DefType, Symbol)>),
    PickFolder,
    ClearPackages,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DefType {
    Function,
    Enum,
    Struct,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    Explorer,
    Search,
}

enum Selection {
    NoSelection,
    PackageSelected(AccountAddress),
    ModuleSelected(AccountAddress, Symbol),
    DefinitionSelected(AccountAddress, Symbol, DefType, Symbol),
}

#[derive(Debug, Clone)]
enum SearchItem {
    Module {
        package_addr: AccountAddress,
        module_name: Symbol,
        display: String,
    },
    Definition {
        package_addr: AccountAddress,
        module_name: Symbol,
        def_type: DefType,
        def_name: Symbol,
        display: String,
    },
}

struct State {
    selection: Selection,
    packages: Option<BTreeMap<AccountAddress, Package>>,
    view: View,
    search_input: String,
    std_filter: bool,
    sui_filter: bool,
    public_only: bool,
}

pub async fn main<P: Into<std::path::PathBuf>>(folder: Option<P>) -> anyhow::Result<()> {
    let packages = if let Some(f) = folder {
        let path: std::path::PathBuf = f.into();
        if path.exists() && path.is_dir() {
            let summary = crate::parser::parse_summaries(&path)?;
            Some(summary.packages)
        } else {
            eprintln!("Invalid path!");
            None
        }
    } else {
        None
    };

    let init_state = State {
        selection: Selection::NoSelection,
        packages,
        view: View::Explorer,
        search_input: String::new(),
        std_filter: false,
        sui_filter: false,
        public_only: false,
    };

    iced::application("Sui Summary Explorer", update, view)
        .style(move |_, _| iced::application::Appearance {
            background_color: Color::from_rgb(0.0, 0.0, 0.0),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
        })
        .subscription(subscription)
        .run_with(|| (init_state, Task::none()))?;

    Ok(())
}

fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::none()
}

pub fn type_to_string(t: &Type) -> String {
    match t {
        Type::Bool => "bool".to_string(),
        Type::U8 => "u8".to_string(),
        Type::U16 => "u16".to_string(),
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::U128 => "u128".to_string(),
        Type::U256 => "u256".to_string(),
        Type::Address => "address".to_string(),
        Type::Signer => "signer".to_string(),
        Type::Datatype(dt) => {
            let args: Vec<String> = dt
                .type_arguments
                .iter()
                .map(|arg| {
                    //let phantom = if arg.phantom { "phantom " } else { "" };
                    //format!("{}{}", phantom, type_to_string(&arg.argument))

                    type_to_string(&arg.argument)
                })
                .collect();
            let args_str = if args.is_empty() {
                "".to_string()
            } else {
                format!("<{}>", args.join(", "))
            };
            format!("{}{}", dt.name, args_str)
        }
        Type::Vector(inner) => format!("vector<{}>", type_to_string(inner)),
        Type::Reference(is_mut, inner) => {
            let mut_str = if *is_mut { "mut " } else { "" };
            format!("&{}{}", mut_str, type_to_string(inner))
        }
        Type::TypeParameter(idx) => format!("T{}", idx),
        Type::NamedTypeParameter(sym) => sym.to_string(),
        Type::Tuple(types) => {
            let type_strs: Vec<String> = types.iter().map(|t| type_to_string(t)).collect();
            format!("({})", type_strs.join(", "))
        }
        Type::Fun(args, ret) => {
            let arg_strs: Vec<String> = args.iter().map(|t| type_to_string(t)).collect();
            format!("fun({}) -> {}", arg_strs.join(", "), type_to_string(ret))
        }
        Type::Any => "_".to_string(),
    }
}
