extern crate gio;
extern crate gtk;
extern crate regex;
extern crate cascade;
use cascade::cascade;

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

    let search_btn = gtk::Button::new_from_icon_name(Some("spacefm-find"), 
                                                    gtk::IconSize::SmallToolbar);

    let header_bar = cascade! {
        gtk::HeaderBar::new();
        ..set_show_close_button(true);
        ..set_title(Some("IconView Example"));
    };
    header_bar.pack_end(&search_btn);

    let window = cascade! {
        gtk::ApplicationWindow::new(application);
        ..set_titlebar(Some(&header_bar));
        ..set_border_width(10);
        ..set_position(gtk::WindowPosition::Center);
        ..set_default_size(500, 500);
    };
    
    // 0 = App name, 1 = Pixbuf Icon, 2 = cmd
    let col_types: [glib::Type; 3] = [glib::Type::String, gdk_pixbuf::Pixbuf::static_type(), glib::Type::String];
    let icon_view_model = gtk::ListStore::new(&col_types);

    let icon_view = cascade! {
        gtk::IconView::new();
        ..set_item_padding(0);
        ..set_columns(3);
        ..set_column_spacing(0);
        // User can select only one item at a time
        ..set_selection_mode(gtk::SelectionMode::Single);
        ..set_activate_on_single_click(true);
        ..set_model(Some(&icon_view_model));
        ..connect_item_activated(|widget, path| {
            let icon_view_model_res = widget.get_model().unwrap().downcast::<gtk::ListStore>();
            let icon_view_model = icon_view_model_res.ok().unwrap();
    
            let iter = icon_view_model.get_iter(path).unwrap();
            let val = icon_view_model.get_value(&iter, IconViewColumnType::CmdColumn as i32);
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
                },
                Err(e) => {println!("{:?}", e)}
            }
        });
        ..set_text_column(IconViewColumnType::TextColumn as i32);
        ..set_pixbuf_column(IconViewColumnType::PixbufColumn as i32);
    };


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

    window.add(&icon_view);
    window.show_all();
}
