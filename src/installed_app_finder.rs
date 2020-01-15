use std::collections::HashMap;
use glib::GString;
use std::path::Path;
use gio::prelude::*;
use gtk::prelude::*;

use gio::{AppInfo, DesktopAppInfo, Icon};
use gtk::{IconTheme, IconLookupFlags};
use gdk_pixbuf::Pixbuf;
use regex::Regex;

#[derive(Debug)]
pub struct AppDetails {
    pub app_name: String,
    pub cmd: String,
    pub icon: Icon,
    pub icon_file_path: String,
    pub icon_pix_buf: Option<Pixbuf>
}


impl PartialEq for AppDetails {
    fn eq(&self, other: &Self) -> bool {
        self.app_name == other.app_name && self.icon_file_path == other.icon_file_path
    }
}

pub struct InstalledAppsFinder {
    pub applications: Option<HashMap<String, Vec<AppDetails>>>
}

impl InstalledAppsFinder {
    pub fn new() -> Self {
        let mut iaf = InstalledAppsFinder {
            applications: None
        };
        iaf.set_applications();
        iaf
    }

    pub fn get_apps_for_category(&self, category: String) -> Option<Vec<&AppDetails>> {
        let mut result = Vec::new();
        if let Some(app) = &self.applications {
            let cartegory_core_apps = app.get(&category.to_string());
            if let Some(cca) = cartegory_core_apps {
                for app in cca {
                    result.push(app);
                } 
                return Some(result);
            }
        }
        return None;
    }

    fn get_pixbuf_from_file_path(&self, file_path: &String) -> Option<Pixbuf> {
        let path = Path::new(file_path);
        // let pixbuf = Pixbuf::new_from_file(path).ok();
        let pixbuf = Pixbuf::new_from_file_at_size(path, 64, 64).ok();
        return pixbuf;
    }

    fn lookup_icon_using_theme(&self, icon_name: GString) -> Option<std::path::PathBuf> {
        // Lookup icon using Gtk Theme
        let icon_theme: Option<IconTheme> = IconTheme::get_default();
        
        if let Some(it) = icon_theme {
            let icon_name_str = String::from(icon_name);
            let icon_info = it.lookup_icon(icon_name_str.as_str(), 64, IconLookupFlags::FORCE_SVG);
            let filename = icon_info.and_then(|icon_info| {
                return icon_info.get_filename();
            });
            return filename;
        }
        return None;
    }

    fn get_icon_file_path(&self, icon: &Icon) -> String {
        let mut new_file_name: String = String::from("");
        let icon_str: Option<GString> = IconExt::to_string(icon);
        if let Some(is) = icon_str {
            let icon_string = is.as_str();
            let icon_str_path = Path::new(icon_string);
            if icon_str_path.exists() {
               new_file_name = String::from(is);
            } else {
                let f_name = self.lookup_icon_using_theme(is);
                if let Some(fname) = f_name {
                    new_file_name = String::from(fname.to_str().unwrap());
                }
            }

        }

        return new_file_name;
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut result = vec![];
        if let Some(a) = &self.applications {
            for (key, _) in a.iter() {
                result.push(String::from(key));
            }
        }

        result
    }

    pub fn pretty_print(&self) {
        if let Some(a) = &self.applications {
            for (key, _) in a.iter() {
                println!("---- Category: {}", key);
                for app_details in a.get(key).unwrap() {
                    let ic = IconExt::to_string(&app_details.icon);
                    let ic_str_opt = ic.unwrap();
                    let ic_str = ic_str_opt.as_str();
                    println!("Name: {}, {}, icon: {}, icon path: {}", app_details.app_name, app_details.cmd, ic_str, app_details.icon_file_path);
                }
            }
        }
    }

    fn set_applications(&mut self) {
        self.applications = Some(self.get_installed_apps());
    }

    fn get_installed_apps(&self)-> HashMap<String, Vec<AppDetails>> {
        let mut applications: HashMap<String, Vec<AppDetails>> = HashMap::new();
        let app_info_list: Vec<AppInfo> = AppInfo::get_all();
        for app_info in app_info_list.iter() {
            
            let (categories, icon) = self.get_icon_and_categories(app_info);
    
            if icon.is_none() || categories.is_none() {
                continue;
            }
    
            if let (Some(cat), Some(icon)) = (categories, icon) {
                let icon_file_path = self.get_icon_file_path(&icon);
                let icon_pix_buf = self.get_pixbuf_from_file_path(&icon_file_path);
    
                let command_line = app_info.get_commandline();
                // Returns command for the app, can be either ie. (gnome-contacts)
                // or full path i.e /usr/share/code/code --no-sandbox --unity-launch
                let cmd_string: Option<String> = command_line.and_then(|cl| {
                    let cl_str_res = cl.into_os_string().into_string();
                    return cl_str_res.ok();
                });
    
                if let Some(cmd_str) = cmd_string {
                    // Remove "%U" and "%F" placeholders
                    let command = self.clean_command_for_file(&cmd_str);
                    let name = app_info.get_name().unwrap();
    
                    let app_details = AppDetails {
                        app_name: String::from(name),
                        cmd: command.into_owned(),
                        icon: icon,
                        icon_file_path: icon_file_path,
                        icon_pix_buf: icon_pix_buf
                    };
    
                    let split_cat = cat.as_str().split(";");
                    for category in split_cat {
                        if category.trim() != "GTK" && category.trim() != "GNOME" && category.trim() != "" {
                            if !applications.contains_key(&category.to_string()) {
                                applications.insert(category.to_string(), Vec::new());
                            }
                            let app_cat = applications.get_mut(&category.to_string());
                            if let Some(ac) = app_cat {
                                // Notes: had to impl PartialEq for AppDetails
                                if !ac.contains(&app_details) {
                                    ac.push(app_details);
                                }
                            }
                            break;
                        }
                    }
                }
            }
    
        }
        return applications;
    }
    

    fn get_icon_and_categories(&self, app_info: &AppInfo) -> (Option<GString>, Option<Icon>) {
        let icon = app_info.get_icon();
        let app_id = app_info.get_id();
        let categories = app_id.and_then(|ai| {
            return DesktopAppInfo::new(&ai);
        }).and_then(|dai| {
            return dai.get_categories();
        });
    
        return (categories, icon);
    }

    fn clean_command_for_file<'t>(&self, cl_str: &'t String) -> std::borrow::Cow<'t, str> {
        let re = Regex::new("%\\w").unwrap();
        return re.replace(cl_str.as_str(), "");
    }
}