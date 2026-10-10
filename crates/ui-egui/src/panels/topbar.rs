//! The top bar: sidebar toggle, back/forward, search, filter, and the right-hand icons.

use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::json;

use crate::LightcraftApp;
use crate::icons::{Icon, paint};
use crate::theme::Tokens;
use crate::widgets::{icon_button, register};

/// The bar doubles as the window's title bar: dragging its empty space moves the window and a
/// double-click zooms or restores it. Registered before the bar's widgets so they win the click.
fn window_handle(app: &LightcraftApp, ui: &mut egui::Ui, content: Rect, margin_left: f32, margin_right: f32) {
    use crate::titlebar::{Gesture, WindowState, command_for};
    let bar = Rect::from_min_max(pos2(content.left() - margin_left, content.top()), pos2(content.right() + margin_right, content.bottom()));
    register(ui.ctx(), "region:titlebar", bar);
    let handle = egui::Id::new("titlebar-handle");
    let resp = ui.interact(bar, handle, Sense::click_and_drag());
    // a press that began on a button or the search field is theirs: dragging off it must not move the window
    // (while pressing, egui hovers every widget under the pointer; once the drag starts, only the dragged one)
    let ctx = ui.ctx().clone();
    let memo = egui::Id::new("titlebar-press-on-widget");
    if resp.is_pointer_button_down_on() && ctx.input(|i| i.pointer.primary_pressed()) {
        let hovered = ctx.interaction_snapshot(|s| s.hovered.clone());
        let on_widget =
            hovered.iter().any(|id| *id != handle && ctx.read_response(*id).is_some_and(|r| r.sense.senses_click() || r.sense.senses_drag()));
        ctx.data_mut(|d| d.insert_temp(memo, on_widget));
    }
    let on_widget = ctx.data(|d| d.get_temp::<bool>(memo)).unwrap_or(false);
    let gesture = if resp.double_clicked_by(egui::PointerButton::Primary) {
        Some(Gesture::DoubleClicked)
    } else if resp.drag_started_by(egui::PointerButton::Primary) && !on_widget {
        Some(Gesture::DragStarted)
    } else {
        None
    };
    if let Some(g) = gesture {
        let window = WindowState { maximized: ui.input(|i| i.viewport().maximized).unwrap_or(false), fullscreen: app.window_is_fullscreen };
        if let Some(cmd) = command_for(g, window) {
            ui.ctx().send_viewport_cmd(cmd);
        }
    }
}

pub fn show(app: &mut LightcraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let left = if app.integrated_titlebar { 78 } else { 10 };
    egui::Panel::top("top_bar")
        .exact_size(t.top_bar_h)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin { left, right: 12, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            if app.integrated_titlebar {
                window_handle(app, ui, full, left as f32, 12.0);
            }
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(egui::RichText::new("Lighthome").font(t.semibold(14.0)).color(t.text));
                ui.add_space(8.0);
                if icon_button(ui, "sidebar", Icon::Sidebar, vec2(30.0, 30.0), app.ui.left_panel, true, "Show/hide My Photos panel").clicked() {
                    let _ = app.run("view.leftPanel", json!({}));
                }
                ui.add_space(6.0);
                if icon_button(ui, "back", Icon::Back, vec2(30.0, 30.0), false, true, "Back").clicked() {
                    let _ = app.run("view.back", json!({}));
                }
                icon_button(ui, "forward", Icon::Forward, vec2(30.0, 30.0), false, false, "Forward");
                if !app.native_menu {
                    // no native menu bar (web, Windows, Linux): menus in the top bar
                    ui.add_space(10.0);
                    let room = (full.center().x - 130.0) - ui.cursor().left() - 16.0;
                    if room > 40.0 {
                        crate::menubar::show_in_window(app, ui, room);
                    }
                }
            });

            // Center: Classic Lightroom Module Tabs [ Library | Develop | Export ]
            let center_x = full.center().x;
            let tab_w = 76.0;
            let tab_h = 28.0;
            let total_tabs_w = tab_w * 3.0 + 8.0 * 2.0;
            let tabs_start_x = center_x - total_tabs_w / 2.0;

            let is_develop = app.ui.right.is_edit_tool();
            let modules = [("library", "Library", !is_develop), ("develop", "Develop", is_develop), ("export", "Export", false)];

            for (i, (id, label, active)) in modules.iter().enumerate() {
                let tab_x = tabs_start_x + (tab_w + 8.0) * i as f32;
                let r = Rect::from_min_size(pos2(tab_x, full.center().y - tab_h / 2.0), vec2(tab_w, tab_h));
                let resp = ui.interact(r, egui::Id::new(("module_tab", id)), Sense::click());
                register(ui.ctx(), format!("tab:{id}"), r);

                let hovered = resp.hovered();
                let text_color = if *active || hovered { t.text } else { t.text_dim };

                let p = ui.painter();
                p.text(r.center(), Align2::CENTER_CENTER, crate::i18n::tr(label), t.semibold(13.0), text_color);

                // Subtle underline indicator for active tab
                if *active {
                    let underline = Rect::from_min_max(pos2(r.left() + 6.0, full.bottom() - 2.0), pos2(r.right() - 6.0, full.bottom()));
                    p.rect_filled(underline, 1.0, t.accent);
                } else if hovered {
                    let underline = Rect::from_min_max(pos2(r.left() + 12.0, full.bottom() - 1.5), pos2(r.right() - 12.0, full.bottom()));
                    p.rect_filled(underline, 0.5, t.text_dim);
                }

                if resp.clicked() {
                    match *id {
                        "library" => {
                            if app.ui.right.is_edit_tool() {
                                app.ui.right = crate::state::RightPanel::Info;
                            }
                        }
                        "develop" => {
                            app.ui.right = crate::state::RightPanel::Edit;
                            if matches!(app.ui.view, crate::state::ViewMode::PhotoGrid | crate::state::ViewMode::SquareGrid) {
                                app.ui.view = crate::state::ViewMode::Detail;
                            }
                        }
                        "export" => {
                            let _ = app.run("dialog.export", json!({}));
                        }
                        _ => {}
                    }
                }
            }

            let has_win_controls = !cfg!(target_os = "macos");
            let win_ctrl_w = if has_win_controls { 42.0 * 3.0 + 10.0 } else { 0.0 };

            // Search field (placed right of the module tabs)
            let search_area_left = full.center().x + total_tabs_w / 2.0 + 20.0;
            let search_area_right = (full.right() - 220.0 - win_ctrl_w).max(search_area_left + 100.0);
            let sw = (search_area_right - search_area_left - 36.0).clamp(100.0, 360.0);
            let sr = Rect::from_min_size(pos2(search_area_left, full.center().y - 14.0), vec2(sw, 28.0));
            let id = crate::text_field::id("field:search");
            if std::mem::take(&mut app.ui.focus_search) {
                ui.memory_mut(|m| m.request_focus(id));
            }
            let focused = ui.memory(|m| m.has_focus(id));
            ui.painter().rect(
                sr,
                6.0,
                if focused { t.canvas } else { t.field },
                Stroke::new(1.0, if focused { t.accent } else { t.field_border }),
                StrokeKind::Inside,
            );
            let mut child =
                ui.new_child(egui::UiBuilder::new().max_rect(sr.shrink2(vec2(10.0, 4.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
            let empty = app.ui.search.is_empty();
            if empty && !focused {
                let g = child.painter().layout_no_wrap(crate::i18n::tr("Search Photos").into(), t.font(13.5), t.text_dim);
                let w = g.size().x + 24.0;
                let x0 = sr.center().x - w / 2.0;
                paint(child.painter(), Rect::from_min_size(pos2(x0, sr.center().y - 8.0), vec2(16.0, 16.0)), Icon::Search, t.text_dim);
                child.painter().galley(pos2(x0 + 24.0, sr.center().y - g.size().y / 2.0), g, t.text_dim);
            }
            let resp = crate::text_field::TextField::singleline("field:search", &mut app.ui.search)
                .frame(false)
                .width(sr.width() - 20.0)
                .font(t.font(13.5))
                .text_color(t.text)
                .show(&mut child)
                .response;
            if resp.changed() {
                let q = app.ui.search.clone();
                let _ = app.run("library.filter", json!({"text": q}));
            }
            // filter icon right of the search field
            let fr = Rect::from_center_size(pos2(sr.right() + 20.0, sr.center().y), vec2(28.0, 28.0));
            let fresp = ui.interact(fr, egui::Id::new("filter-btn"), Sense::click());
            register(ui.ctx(), "icon:filter", fr);
            let filtering = app.session.filter != Default::default() || app.ui.filter_bar;
            paint(
                ui.painter(),
                fr.shrink(6.0),
                Icon::Filter,
                if filtering {
                    t.accent
                } else if fresp.hovered() {
                    t.text
                } else {
                    t.icon
                },
            );
            // badge: how many filters are on, even with the filter bar closed
            let active = lightcraft_engine::filter_chips(&app.session.filter, &app.session.catalog).len();
            if active > 0 {
                let c = fr.right_top() + vec2(-3.0, 8.0);
                ui.painter().circle_filled(c, 7.0, t.accent);
                ui.painter().text(c, Align2::CENTER_CENTER, active.to_string(), t.semibold(9.5), t.canvas);
            }
            let fresp = fresp.on_hover_text(if active > 0 {
                crate::i18n::tr_format!("Filter bar — {active} active filter{}", if active == 1 { "" } else { "s" }, active = active)
            } else {
                "Filter bar".into()
            });
            if fresp.clicked() {
                let _ = app.run("view.filterBar", json!({}));
            }
            // right utility icons
            let mut x = full.right() - 18.0 - win_ctrl_w;
            // saving is failing: the cloud icon turns into a warning until a save succeeds
            let unsaved = app.session.unsaved().map(|(n, e)| {
                crate::i18n::tr_format!(
                    "{n} change{} saved in memory but not written to disk: {e}\nLightCraft retries automatically; quitting now would lose {}.",
                    if n == 1 { "" } else { "s" },
                    if n == 1 { "it" } else { "them" },
                    e = e,
                    n = n
                )
            });
            let cloud_tip = unsaved.as_deref().unwrap_or("Local library — no cloud account needed");
            for (id, icon, tip, cmd) in [
                ("discord", Icon::Chat, "Join the ArtCraft community on Discord", "app.discord"),
                ("cloud", Icon::Cloud, cloud_tip, ""),
                ("help", Icon::Help, "Keyboard shortcuts", "app.shortcuts"),
                ("share", Icon::Share, "Export", "dialog.export"),
                ("bell", Icon::Bell, "Activity", "panel.activity"),
            ] {
                let r = Rect::from_center_size(pos2(x, full.center().y), vec2(28.0, 28.0));
                let resp = ui.interact(r, egui::Id::new(("top", id)), Sense::click()).on_hover_text(tip);
                register(ui.ctx(), format!("icon:{id}"), r);
                let warn = id == "cloud" && unsaved.is_some();
                let colour = if warn {
                    t.caution
                } else if resp.hovered() {
                    t.text
                } else {
                    t.icon
                };
                paint(ui.painter(), r.shrink(5.0), icon, colour);
                if warn {
                    let c = r.right_top() + vec2(-5.0, 6.0);
                    ui.painter().circle_filled(c, 6.0, t.reject);
                    ui.painter().text(c, Align2::CENTER_CENTER, "!", t.semibold(9.5), t.canvas);
                    register(ui.ctx(), "indicator:unsaved", r);
                }
                if resp.clicked() && !cmd.is_empty() {
                    let _ = app.run(cmd, json!({}));
                }
                x -= 40.0;
            }

            // Integrated window controls (Minimize, Fullscreen, Close)
            if has_win_controls {
                let btn_w = 42.0;
                let h = full.height();
                let btns = [
                    ("window:close", Icon::Close, "Close"),
                    ("window:fullscreen", Icon::Fullscreen, "Toggle Fullscreen"),
                    ("window:minimize", Icon::Minus, "Minimize"),
                ];

                for (i, (id, icon, tip)) in btns.iter().enumerate() {
                    let rx = full.right() - btn_w * (i + 1) as f32;
                    let r = Rect::from_min_size(pos2(rx, full.top()), vec2(btn_w, h));
                    let resp = ui.interact(r, egui::Id::new(("win_ctrl", id)), Sense::click()).on_hover_text(crate::i18n::tr(tip));
                    register(ui.ctx(), id.to_string(), r);

                    let hovered = resp.hovered();
                    let is_close = *id == "window:close";
                    let p = ui.painter();

                    if hovered {
                        let fill = if is_close { Color32::from_rgb(224, 74, 74) } else { t.hover.gamma_multiply(0.8) };
                        p.rect_filled(r, 0.0, fill);
                    }

                    let icon_color = if hovered && is_close {
                        Color32::WHITE
                    } else if hovered {
                        t.text
                    } else {
                        t.icon
                    };

                    paint(p, Rect::from_center_size(r.center(), vec2(13.0, 13.0)), *icon, icon_color);

                    if resp.clicked() {
                        match *id {
                            "window:close" => {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            "window:fullscreen" => {
                                let fullscreen = ui.input(|i| i.viewport().fullscreen).unwrap_or(app.window_is_fullscreen);
                                app.window_is_fullscreen = !fullscreen;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                            }
                            "window:minimize" => {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                            _ => {}
                        }
                    }
                }

                // Subtle divider between utility icons and window controls
                let div_x = full.right() - btn_w * 3.0 - 5.0;
                ui.painter().line_segment([pos2(div_x, full.center().y - 10.0), pos2(div_x, full.center().y + 10.0)], Stroke::new(1.0, t.divider));
            }
            let _ = Align2::CENTER_CENTER;
        });
}
