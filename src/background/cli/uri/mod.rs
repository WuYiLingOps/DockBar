use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use dockbar_core::{
    resource::{DockBarResourceFile, Resource, ResourceId, ResourceKind},
    state::{CssStyles, Dialog, DialogContent},
};
use tauri::Listener;
use uuid::Uuid;

use crate::{
    app::get_app_handle, error::Result, resources::RESOURCES, state::application::FULL_STATE,
    utils::constants::DOCKBAR_COMMON, widgets::trigger_dialog_backend,
};

pub const URI: &str = "dockbar.uri:";

pub async fn process_uri(uri: &str) -> Result<()> {
    log::trace!("Loading URI: {uri}");

    if !uri.starts_with(URI) {
        let path = PathBuf::from(uri);
        if !path.is_file() || path.extension() != Some(OsStr::new("dockbar-cli")) || !path.exists()
        {
            return Err("Invalid file to load".into());
        }

        let file = DockBarResourceFile::load(&path).await?;
        store_file_on_respective_user_folder(&file).await?;

        let dialog_id = Uuid::new_v4();
        trigger_dialog_backend(Dialog {
            identifier: dialog_id,
            ..Default::default()
        })?;
        update_dialog_to_added_resource(dialog_id, &file.resource)?;
        return Ok(());
    }

    // Remote resource downloads (product server) were removed; only local
    // resource files are supported on this build.
    let _ = uri;
    Ok(())
}

fn path_by_resource_kind(kind: &ResourceKind) -> &Path {
    match kind {
        ResourceKind::IconPack => DOCKBAR_COMMON.user_icons_path(),
        ResourceKind::Widget => DOCKBAR_COMMON.user_widgets_path(),
    }
}

async fn store_file_on_respective_user_folder(file: &DockBarResourceFile) -> Result<PathBuf> {
    let mut path_to_store = path_by_resource_kind(&file.resource.kind).to_path_buf();
    if file.resource.kind == ResourceKind::IconPack {
        path_to_store.push(file.resource.id.to_string());
        tokio::fs::create_dir_all(&path_to_store).await?;
        path_to_store.push("metadata.dockbar");
    } else {
        path_to_store.push(format!("{}.dockbar", file.resource.id));
    }
    file.store(&path_to_store).await?;
    Ok(path_to_store)
}

fn update_dialog_to_added_resource(dialog_id: Uuid, resource: &Resource) -> Result<()> {
    let config = resource_to_dialog(dialog_id, resource)?;
    trigger_dialog_backend(config)?;

    let used_id = ResourceId::Remote(resource.id);
    let kind = resource.kind;

    let event = format!("resource::{}::enable", resource.id);
    get_app_handle().once(event, move |_| {
        RESOURCES.enable_resource(kind, used_id.clone());
    });

    Ok(())
}

fn resource_to_dialog(id: Uuid, resource: &Resource) -> Result<Dialog> {
    let image_styles = CssStyles::new()
        .add("width", "90px")
        .add("minWidth", "90px")
        .add("height", "90px")
        .add("borderRadius", "14px")
        .add("backgroundColor", "var(--color-gray-200)")
        .add("display", "flex")
        .add("alignItems", "center")
        .add("justifyContent", "center");

    let image = if let Some(url) = &resource.metadata.portrait {
        DialogContent::Image {
            href: url.clone(),
            styles: Some(image_styles),
        }
    } else {
        DialogContent::Icon {
            name: "GrStatusUnknown".to_string(),
            styles: Some(image_styles),
        }
    };

    let state = FULL_STATE.load();
    let locale = state.locale();

    Ok(Dialog {
        identifier: id,
        width: 500.0,
        height: 280.0,
        title: vec![DialogContent::Group {
            items: vec![
                DialogContent::Icon {
                    name: "GrCircleInformation".to_string(),
                    styles: Some(
                        CssStyles::new()
                            .add("color", "var(--color-blue-900)")
                            .add("height", "1.2rem"),
                    ),
                },
                DialogContent::Text {
                    value: t!("resource.added").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![DialogContent::Group {
            items: vec![
                image,
                DialogContent::Group {
                    items: vec![
                        DialogContent::Text {
                            value: resource.metadata.display_name.get(locale).to_owned(),
                            styles: Some(
                                CssStyles::new()
                                    .add("fontWeight", "bold")
                                    .add("fontSize", "2rem")
                                    .add("lineHeight", "1.2em"),
                            ),
                        },
                        DialogContent::Text {
                            value: resource.metadata.description.get(locale).to_owned(),
                            styles: None,
                        },
                    ],
                    styles: Some(CssStyles::new().add("flexDirection", "column")),
                },
            ],
            styles: None,
        }],
        footer: vec![DialogContent::Button {
            skin: Some("solid".to_string()),
            inner: vec![DialogContent::Text {
                value: t!("resource.enable").to_string(),
                styles: None,
            }],
            on_click: format!("resource::{}::enable", resource.id),
            styles: None,
        }],
    })
}
