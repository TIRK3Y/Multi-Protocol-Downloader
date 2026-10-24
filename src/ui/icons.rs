use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use std::f32::consts::PI;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Icon {
    Home,
    Download,
    History,
    Platforms,
    Settings,
    About,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    Video,
    Magnet,
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, stroke: Stroke) {
    let center = rect.center();
    let size = rect.width().min(rect.height());
    let half = size * 0.5;

    match icon {
        Icon::Home => {
            // Peak and roof
            let peak = Pos2::new(center.x, center.y - half + 2.5);
            let left_eave = Pos2::new(center.x - half + 2.5, center.y - 0.5);
            let right_eave = Pos2::new(center.x + half - 2.5, center.y - 0.5);

            painter.line_segment([left_eave, peak], stroke);
            painter.line_segment([peak, right_eave], stroke);

            // Walls
            let left_wall_top = Pos2::new(center.x - half + 4.0, center.y - 0.5);
            let left_wall_bot = Pos2::new(center.x - half + 4.0, center.y + half - 2.5);
            let right_wall_top = Pos2::new(center.x + half - 4.0, center.y - 0.5);
            let right_wall_bot = Pos2::new(center.x + half - 4.0, center.y + half - 2.5);

            painter.line_segment([left_wall_top, left_wall_bot], stroke);
            painter.line_segment([right_wall_top, right_wall_bot], stroke);
            painter.line_segment([left_wall_bot, right_wall_bot], stroke);

            // Door opening
            let door_w = half * 0.35;
            let door_top = center.y + half * 0.15;
            let door_bot = center.y + half - 2.5;
            painter.line_segment(
                [Pos2::new(center.x - door_w, door_bot), Pos2::new(center.x - door_w, door_top)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(center.x - door_w, door_top), Pos2::new(center.x + door_w, door_top)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(center.x + door_w, door_top), Pos2::new(center.x + door_w, door_bot)],
                stroke,
            );
        }
        Icon::Download => {
            // Bottom tray
            let tray_y = center.y + half - 2.5;
            let tray_lip_y = center.y + half - 5.5;
            let left_x = center.x - half + 3.0;
            let right_x = center.x + half - 3.0;

            painter.line_segment([Pos2::new(left_x, tray_lip_y), Pos2::new(left_x, tray_y)], stroke);
            painter.line_segment([Pos2::new(left_x, tray_y), Pos2::new(right_x, tray_y)], stroke);
            painter.line_segment([Pos2::new(right_x, tray_y), Pos2::new(right_x, tray_lip_y)], stroke);

            // Arrow shaft
            let arrow_top = Pos2::new(center.x, center.y - half + 2.5);
            let arrow_tip = Pos2::new(center.x, center.y + half - 6.5);
            painter.line_segment([arrow_top, arrow_tip], stroke);

            // Arrow head
            let head_span = half * 0.45;
            let head_y = arrow_tip.y - head_span;
            painter.line_segment([Pos2::new(center.x - head_span, head_y), arrow_tip], stroke);
            painter.line_segment([Pos2::new(center.x + head_span, head_y), arrow_tip], stroke);
        }
        Icon::History => {
            // Circular clock
            let r = half - 2.5;
            painter.circle_stroke(center, r, stroke);
            // Hour hand and minute hand
            let hour_tip = Pos2::new(center.x, center.y - r * 0.55);
            let min_tip = Pos2::new(center.x + r * 0.5, center.y);
            painter.line_segment([center, hour_tip], stroke);
            painter.line_segment([center, min_tip], stroke);
        }
        Icon::Platforms => {
            // 2x2 grid (4 square blocks)
            let gap = 2.0;
            let sq_size = (size - 7.0 - gap) * 0.5;
            let x0 = center.x - half + 3.5;
            let y0 = center.y - half + 3.5;

            for row in 0..2 {
                for col in 0..2 {
                    let rx = x0 + (col as f32) * (sq_size + gap);
                    let ry = y0 + (row as f32) * (sq_size + gap);
                    let block = Rect::from_min_size(Pos2::new(rx, ry), Vec2::new(sq_size, sq_size));
                    painter.rect_stroke(block, 1.0, stroke, StrokeKind::Inside);
                }
            }
        }
        Icon::Settings => {
            // Center ring
            let inner_r = half * 0.42;
            painter.circle_stroke(center, inner_r, stroke);

            // 6 radial cogs
            let r_start = half * 0.55;
            let r_end = half - 2.0;
            for i in 0..6 {
                let angle = (i as f32) * (PI / 3.0);
                let cos = angle.cos();
                let sin = angle.sin();
                let p1 = Pos2::new(center.x + cos * r_start, center.y + sin * r_start);
                let p2 = Pos2::new(center.x + cos * r_end, center.y + sin * r_end);
                painter.line_segment([p1, p2], stroke);
            }
        }
        Icon::About => {
            // Info circle
            let r = half - 2.5;
            painter.circle_stroke(center, r, stroke);

            // Dot
            let dot_center = Pos2::new(center.x, center.y - r * 0.42);
            painter.circle_filled(dot_center, stroke.width * 0.8, stroke.color);

            // Stem
            let stem_top = Pos2::new(center.x, center.y - r * 0.05);
            let stem_bot = Pos2::new(center.x, center.y + r * 0.5);
            painter.line_segment([stem_top, stem_bot], stroke);
        }
        Icon::ChevronLeft => {
            let dx = half * 0.35;
            let dy = half * 0.5;
            let p1 = Pos2::new(center.x + dx, center.y - dy);
            let p2 = Pos2::new(center.x - dx, center.y);
            let p3 = Pos2::new(center.x + dx, center.y + dy);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        }
        Icon::ChevronRight => {
            let dx = half * 0.35;
            let dy = half * 0.5;
            let p1 = Pos2::new(center.x - dx, center.y - dy);
            let p2 = Pos2::new(center.x + dx, center.y);
            let p3 = Pos2::new(center.x - dx, center.y + dy);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        }
        Icon::ChevronDown => {
            let dx = half * 0.45;
            let dy = half * 0.28;
            let p1 = Pos2::new(center.x - dx, center.y - dy);
            let p2 = Pos2::new(center.x, center.y + dy);
            let p3 = Pos2::new(center.x + dx, center.y - dy);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
        }
        Icon::Video => {
            // Video monitor / film rect
            let w = size - 6.0;
            let h = (size - 6.0) * 0.75;
            let r = Rect::from_center_size(center, Vec2::new(w, h));
            painter.rect_stroke(r, 2.0, stroke, StrokeKind::Inside);

            // Triangle play symbol
            let tw = 3.0;
            let th = 3.5;
            let p1 = Pos2::new(center.x - tw * 0.6, center.y - th);
            let p2 = Pos2::new(center.x + tw * 1.0, center.y);
            let p3 = Pos2::new(center.x - tw * 0.6, center.y + th);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.line_segment([p3, p1], stroke);
        }
        Icon::Magnet => {
            // U-shape magnet
            let leg_w = half * 0.45;
            let leg_top = center.y - half + 3.0;
            let bend_y = center.y + half * 0.3;
            let bot_y = center.y + half - 2.5;

            let left_x = center.x - leg_w;
            let right_x = center.x + leg_w;

            painter.line_segment([Pos2::new(left_x, leg_top), Pos2::new(left_x, bend_y)], stroke);
            painter.line_segment([Pos2::new(right_x, leg_top), Pos2::new(right_x, bend_y)], stroke);
            painter.line_segment([Pos2::new(left_x, bend_y), Pos2::new(center.x, bot_y)], stroke);
            painter.line_segment([Pos2::new(center.x, bot_y), Pos2::new(right_x, bend_y)], stroke);

            // Pole marks
            let mark_y = leg_top + 4.0;
            painter.line_segment(
                [Pos2::new(left_x - 1.5, mark_y), Pos2::new(left_x + 1.5, mark_y)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(right_x - 1.5, mark_y), Pos2::new(right_x + 1.5, mark_y)],
                stroke,
            );
        }
    }
}

pub fn render_sidebar_item(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    is_active: bool,
    expanded: bool,
    indent: bool,
    has_dropdown: bool,
    dropdown_open: bool,
) -> egui::Response {
    let desired_height = 34.0;
    let width = if expanded {
        ui.available_width()
    } else {
        38.0
    };

    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, desired_height),
        egui::Sense::click(),
    );

    let is_hovered = response.hovered();

    let bg_color = if is_active {
        Color32::from_rgb(41, 128, 185)
    } else if is_hovered {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };

    if bg_color != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 4.0, bg_color);
    }

    let icon_color = if is_active {
        Color32::WHITE
    } else if is_hovered {
        ui.visuals().widgets.hovered.fg_stroke.color
    } else {
        ui.visuals().text_color()
    };

    let stroke = Stroke::new(1.6_f32, icon_color);

    if expanded {
        let x_offset = if indent { 22.0 } else { 8.0 };
        let icon_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + x_offset, rect.center().y - 9.0),
            Vec2::new(18.0, 18.0),
        );
        paint_icon(ui.painter(), icon_rect, icon, stroke);

        // Label text
        let text_pos = Pos2::new(rect.min.x + x_offset + 24.0, rect.center().y - 7.0);
        let font_id = egui::FontId::proportional(if indent { 12.0 } else { 13.0 });
        ui.painter().text(
            text_pos,
            egui::Align2::LEFT_TOP,
            label,
            font_id,
            icon_color,
        );

        // Chevron if expandable
        if has_dropdown {
            let chevron_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - 22.0, rect.center().y - 7.0),
                Vec2::new(14.0, 14.0),
            );
            let chev_icon = if dropdown_open {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            };
            paint_icon(ui.painter(), chevron_rect, chev_icon, Stroke::new(1.4_f32, icon_color));
        }
    } else {
        // Collapsed mode: center the icon exactly in the button
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::new(20.0, 20.0));
        paint_icon(ui.painter(), icon_rect, icon, stroke);
    }

    if !expanded {
        response.on_hover_text(label)
    } else {
        response
    }
}
