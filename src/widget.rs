// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

// Based on gnome-sound-recorder/src/waveform.js:
// - Copyright 2013 Meg Ford
// - Copyright 2022 Kavan Mevada
// Released under the terms of the LGPL 2.0 or later

#![allow(deprecated)]

use std::{
    cell::{Cell, RefCell},
    ops::DivAssign,
};

use adw::subclass::prelude::*;
use glib::clone;
use gtk::gdk::prelude::*;
use gtk::{gdk, glib, graphene, prelude::*};
use log::{debug, warn};

#[derive(Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum PlotType {
    #[default]
    Line,
    Scatter,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

mod imp {
    use std::borrow::Borrow;

    use super::*;
    use glib::{subclass::Signal, ParamSpec, Value};
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct PlotViewImpl {
        pub title: RefCell<String>,
        pub typ: Cell<PlotType>,
        pub x_label: RefCell<String>,
        pub y_label: RefCell<String>,
        pub x_max: Cell<Option<f64>>,
        pub y_max: Cell<Option<f64>>,
        // pub font_name: String,
        pub axis_color: Option<gtk::gdk::RGBA>,
        pub line_color: Option<gtk::gdk::RGBA>,
        pub text_color: Option<gtk::gdk::RGBA>,
        pub grid_color: Option<gtk::gdk::RGBA>,
        pub point_list: RefCell<Vec<Point>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlotViewImpl {
        const NAME: &'static str = "AmberolWaveformView";
        type Type = super::PlotView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("plotview");
            klass.set_accessible_role(gtk::AccessibleRole::Widget);
        }
    }

    impl ObjectImpl for PlotViewImpl {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                _ => unimplemented!(),
            };
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| vec![]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for PlotViewImpl {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::ConstantSize
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let w = widget.width() as f64;
            let h = widget.height() as f64;

            // Grab the colors
            let style_context = widget.style_context();
            let color = style_context.color();

            let text_color = self.text_color.borrow().unwrap_or(color);
            let line_color = self.line_color.borrow().unwrap_or_else(|| {
                style_context
                    .lookup_color("theme_selected_bg_color")
                    .expect("Unable to get theme color")
            });
            let grid_color = self.grid_color.unwrap_or(color.with_alpha(0.1));
            let axis_color = self.axis_color.unwrap_or(color);

            let x_max = match self.x_max.get() {
                None => self
                    .point_list
                    .borrow()
                    .iter()
                    .map(|p| p.x)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(0.),
                Some(value) => value,
            };

            let y_max = match self.y_max.get() {
                None => self
                    .point_list
                    .borrow()
                    .iter()
                    .map(|p| p.y)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(0.),
                Some(value) => value,
            };

            let cairo = snapshot.append_cairo(&graphene::Rect::new(0., 0., w as f32, h as f32));
            cairo.set_antialias(gtk::cairo::Antialias::Fast);
            cairo.set_tolerance(1.5);
            // GdkCairoContextExt::set_source_rgba(&cairo, &text_color);
            // cairo.select_font_face(
            //     &self.font_name,
            //     gtk::cairo::FontSlant::Normal,
            //     gtk::cairo::FontWeight::Normal,
            // );

            // Move coordinate system to bottom left
            cairo.translate(0., h as f64);

            // Invert y-axis
            cairo.scale(1., -1.);

            // Draw title
            cairo.set_font_size(15.0 * (w / 650.));
            let extents = cairo.text_extents(self.title.borrow().as_str()).unwrap();
            cairo.move_to(
                0.5 * w - extents.width() / 2.,
                0.9 * h - extents.height() / 2.,
            );
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(self.title.borrow().as_str());
            cairo.restore();

            // Draw x-axis label
            cairo.set_font_size(11.0 * (w / 650.));
            let extents = cairo.text_extents(self.x_label.borrow().as_str()).unwrap();
            cairo.move_to(0.5 * w - extents.width() / 2., 0.075 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(self.x_label.borrow().as_str());
            cairo.restore();

            // Draw y-axis label
            let extents = cairo.text_extents(self.y_label.borrow().as_str()).unwrap();
            cairo.move_to(0.035 * w, 0.5 * h - extents.width() / 2.);
            cairo.save();
            cairo.rotate(std::f64::consts::PI / 2.);
            cairo.scale(1., -1.);
            cairo.show_text(self.y_label.borrow().as_str());
            cairo.restore();

            // Draw x-axis
            GdkCairoContextExt::set_source_rgba(&cairo, &axis_color);
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.2 * h);
            cairo.line_to(0.9 * w, 0.2 * h);
            cairo.stroke();

            // Draw y-axis
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.8 * h);
            cairo.line_to(0.1 * w, 0.2 * h);
            cairo.stroke();

            // Draw x-axis value at 100% mark
            GdkCairoContextExt::set_source_rgba(&cairo, &text_color);
            let value = format!("{}", x_max);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.9 * w - extents.width() / 2., 0.16 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw x-axis value at 75% mark
            let value = format!("{}", (x_max / 4.) * 3.);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.7 * w - extents.width() / 2., 0.16 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw x-axis value at 50% mark
            let value = format!("{}", x_max / 2.);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.5 * w - extents.width() / 2., 0.16 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw x-axis value at 25% mark
            let value = format!("{}", x_max / 4.);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.3 * w - extents.width() / 2., 0.16 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw x-axis value at 0% mark
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents("0").unwrap();
            cairo.move_to(0.1 * w - extents.width() / 2., 0.16 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text("0");
            cairo.restore();

            // Draw y-axis value at 0% mark
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents("0").unwrap();
            cairo.move_to(0.091 * w - extents.width(), 0.191 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text("0");
            cairo.restore();

            // Draw y-axis value at 25% mark
            let value = format!("{}", y_max / 4.);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.091 * w - extents.width(), 0.34 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw y-axis value at 50% mark
            let value = format!("{}", y_max / 2.);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.091 * w - extents.width(), 0.49 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw y-axis value at 75% mark
            let value = format!("{}", (y_max / 4.) * 3.);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.091 * w - extents.width(), 0.64 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw y-axis value at 100% mark
            let value = format!("{}", y_max);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(0.091 * w - extents.width(), 0.79 * h);
            cairo.save();
            cairo.scale(1., -1.);
            cairo.show_text(&value);
            cairo.restore();

            // Draw grid x-line 25%
            GdkCairoContextExt::set_source_rgba(&cairo, &grid_color);
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.35 * h);
            cairo.line_to(0.9 * w, 0.35 * h);
            cairo.stroke();

            // Draw grid x-line 50%
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.5 * h);
            cairo.line_to(0.9 * w, 0.5 * h);
            cairo.stroke();

            // Draw grid x-line 75%
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.65 * h);
            cairo.line_to(0.9 * w, 0.65 * h);
            cairo.stroke();

            // Draw grid x-line 100%
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.8 * h);
            cairo.line_to(0.9 * w, 0.8 * h);
            cairo.stroke();

            // Draw grid y-line 25%
            cairo.set_line_width(1.);
            cairo.move_to(0.3 * w, 0.8 * h);
            cairo.line_to(0.3 * w, 0.2 * h);
            cairo.stroke();

            // Draw grid y-line 50%
            cairo.set_line_width(1.);
            cairo.move_to(0.5 * w, 0.8 * h);
            cairo.line_to(0.5 * w, 0.2 * h);
            cairo.stroke();

            // Draw grid y-line 75%
            cairo.set_line_width(1.);
            cairo.move_to(0.7 * w, 0.8 * h);
            cairo.line_to(0.7 * w, 0.2 * h);
            cairo.stroke();

            // Draw grid y-line 100%
            cairo.set_line_width(1.);
            cairo.move_to(0.9 * w, 0.8 * h);
            cairo.line_to(0.9 * w, 0.2 * h);
            cairo.stroke();

            // Move coordinate system to (0,0) of drawn coordinate system
            cairo.translate(0.1 * w, 0.2 * h);
            GdkCairoContextExt::set_source_rgba(&cairo, &line_color);
            cairo.set_line_width(2.0);

            // Calc scales
            let x_scale = (w - 2. * 0.1 * w) / x_max;
            let y_scale = (h - 2. * 0.2 * h) / y_max;

            // Draw data points from list
            // for (l = self.point_list; l != NULL; l = l.next)
            let borrow = &self.point_list.borrow();
            let mut point_iter = borrow.iter();

            if matches!(self.typ.get(), PlotType::Line) {
                // Move to first point
                if let Some(point) = point_iter.next() {
                    cairo.move_to(point.x * x_scale, point.y * y_scale);
                }
            }
            for point in point_iter {
                match self.typ.get() {
                    PlotType::Line => {
                        // Draw line to next point
                        cairo.line_to(point.x * x_scale, point.y * y_scale);
                        cairo.set_line_join(gtk::cairo::LineJoin::Round);
                        cairo.stroke();
                        cairo.move_to(point.x * x_scale, point.y * y_scale);
                    }
                    PlotType::Scatter => {
                        // Draw square
                        //cairo.rectangle(point.x * x_scale, point.y * y_scale, 4, 4);
                        //cairo.fill();

                        // Draw point
                        cairo.set_line_width(3.);
                        cairo.set_line_cap(gtk::cairo::LineCap::Round);
                        cairo.move_to(point.x * x_scale, point.y * y_scale);
                        cairo.close_path();
                        cairo.stroke();
                    }
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct PlotView(ObjectSubclass<imp::PlotViewImpl>)
        @extends gtk::Widget,
        @implements gtk::Accessible;
}

impl Default for PlotView {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl PlotView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_points(&self, point_list: Vec<Point>) {
        self.imp().point_list.replace(point_list);
        self.queue_draw();
    }
    pub fn set_x_max(&self, x_max: f64) {
        self.imp().x_max.replace(Some(x_max));
        self.queue_draw();
    }
    pub fn set_y_max(&self, y_max: f64) {
        self.imp().y_max.replace(Some(y_max));
        self.queue_draw();
    }
    pub fn set_type(&self, plot_type: PlotType) {
        self.imp().typ.replace(plot_type);
        self.queue_draw();
    }
}
