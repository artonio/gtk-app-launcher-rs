extern crate gio;
extern crate gtk;
extern crate regex;


use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use gio::{AppInfoExt};

use glib::GString;

mod installed_app_finder;
use installed_app_finder::InstalledAppsFinder;

enum IconViewColumnType {
    TextColumn = 0,
    PixbufColumn = 1,
    CmdColumn = 2
}


fn main() {
    let application = gtk::Application::new(
        Some("com.github.artonio.gtkapplauncher"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("IconView Example");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(500, 500);

    let icon_view = gtk::IconView::new();
    icon_view.set_item_padding(0);
    icon_view.set_columns(3);
    icon_view.set_column_spacing(0);
    // User can select only one item at a time
    icon_view.set_selection_mode(gtk::SelectionMode::Single);

    // 0 = App name, 1 = Pixbuf Icon, 2 = cmd
    let col_types: [glib::Type; 3] = [glib::Type::String, gdk_pixbuf::Pixbuf::static_type(), glib::Type::String];
    let icon_view_model = gtk::ListStore::new(&col_types);
    icon_view.set_model(Some(&icon_view_model));

    let iaf = InstalledAppsFinder::new();
    iaf.pretty_print();

    let core_apps = iaf.get_apps_for_category("Office".to_string());
    for cp in core_apps {
        icon_view_model.insert_with_values(
            None,
            &[
                IconViewColumnType::TextColumn as u32,
                IconViewColumnType::PixbufColumn as u32,
                IconViewColumnType::CmdColumn as u32
            ],
            &[&cp.app_name, &cp.icon_pix_buf, &cp.cmd]
        );
    }
    icon_view.set_activate_on_single_click(true);
    icon_view.connect_item_activated(|widget, path| {
        let icon_view_model_res = widget.get_model().unwrap().downcast::<gtk::ListStore>();
        let icon_view_model = icon_view_model_res.ok().unwrap();

        let iter = icon_view_model.get_iter(path).unwrap();
        let iter_str = icon_view_model.get_string_from_iter(&iter).unwrap();
        let val = icon_view_model.get_value(&iter, 2);
        let t: Result<glib::value::TypedValue<String>, glib::value::Value>  = val.downcast();
        match t {
            Ok(o) => {
                let command = o.get().unwrap();
                let a_info = gio::AppInfo::create_from_commandline(command.as_str(), None, gio::AppInfoCreateFlags::NONE).ok();
                if let Some(a) = a_info {
                    let empty_file_array: [gio::File; 0] = [];
                    let result: Result<(), glib::error::Error> = a.launch(&empty_file_array, Some(&gio::AppLaunchContext::new()));
                    match result {
                        Ok(o) => {},
                        Err(e) => println!("{:?}", e)
                    }
                }
                // a.launch(None, None);
                // println!("{:?}", o.get().unwrap());
            },
            Err(e) => {println!("{:?}", e)}
        }
    });

    icon_view.set_text_column(IconViewColumnType::TextColumn as i32);
    icon_view.set_pixbuf_column(IconViewColumnType::PixbufColumn as i32);

    window.add(&icon_view);
    window.show_all();
}
