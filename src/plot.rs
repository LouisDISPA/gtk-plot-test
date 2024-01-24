use adw::subclass::prelude::*;
use gtk::{glib, prelude::*};

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
    use std::cell::{Cell, RefCell};

    use adw::subclass::prelude::*;
    use gtk::{glib, graphene, prelude::*};

    use super::{PlotType, Point};

    #[derive(Debug, Default)]
    pub struct PlotViewImpl {
        pub title: RefCell<String>,
        pub typ: Cell<PlotType>,
        pub x_label: RefCell<String>,
        pub y_label: RefCell<String>,
        pub x_max: Cell<Option<f64>>,
        pub y_max: Cell<Option<f64>>,
        pub x_min: Cell<Option<f64>>,
        pub y_min: Cell<Option<f64>>,
        pub font_name: RefCell<String>,
        pub axis_color: Option<gtk::gdk::RGBA>,
        pub line_color: Option<gtk::gdk::RGBA>,
        pub text_color: Option<gtk::gdk::RGBA>,
        pub grid_color: Option<gtk::gdk::RGBA>,
        pub point_list: RefCell<Vec<Point>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlotViewImpl {
        const NAME: &'static str = "PlotView";
        type Type = super::PlotView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("plotview");
            klass.set_accessible_role(gtk::AccessibleRole::Widget);
        }
    }

    impl ObjectImpl for PlotViewImpl {}

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

            let text_color = self.text_color.unwrap_or(color);
            let line_color = self.line_color.unwrap_or_else(|| {
                style_context
                    .lookup_color("theme_selected_bg_color")
                    .expect("Unable to get theme color")
            });
            let grid_color = self.grid_color.unwrap_or(color.with_alpha(0.1));
            let axis_color = self.axis_color.unwrap_or(color);

            // Do one pass to get the min and max values for x and y
            let (x_max, x_min, y_max, y_min) = {
                let borrow = &self.point_list.borrow();
                let mut point_iter = borrow.iter();
                if let Some(first) = point_iter.next() {
                    let mut x_max = first.x;
                    let mut x_min = first.x;
                    let mut y_max = first.y;
                    let mut y_min = first.y;

                    for point in point_iter {
                        x_max = x_max.max(point.x);
                        x_min = x_min.min(point.x);
                        y_max = y_max.max(point.y);
                        y_min = y_min.min(point.y);
                    }

                    (Some(x_max), Some(x_min), Some(y_max), Some(y_min))
                } else {
                    (None, None, None, None)
                }
            };

            // Calculate the min and max values for x and y
            let x_max = self.x_max.get().or(x_max).unwrap_or(500.);
            let x_min = self.x_min.get().or(x_min).unwrap_or(0.);
            let y_max = self.y_max.get().or(y_max).unwrap_or(100.);
            let y_min = self.y_min.get().or(y_min).unwrap_or(0.);

            // Create a cairo context
            let cairo = snapshot.append_cairo(&graphene::Rect::new(0., 0., w as f32, h as f32));
            cairo.set_antialias(gtk::cairo::Antialias::Fast);
            cairo.set_tolerance(1.5);
            cairo.select_font_face(
                &self.font_name.borrow(),
                gtk::cairo::FontSlant::Normal,
                gtk::cairo::FontWeight::Normal,
            );

            GdkCairoContextExt::set_source_rgba(&cairo, &text_color);

            // Draw title
            cairo.set_font_size(15.0 * (w / 650.));
            let extents = cairo.text_extents(self.title.borrow().as_str()).unwrap();
            cairo.move_to(
                0.5 * w - extents.width() / 2.,
                0.1 * h - extents.height() / 2.,
            );
            cairo.show_text(self.title.borrow().as_str()).unwrap();

            // Draw x-axis label
            cairo.set_font_size(11.0 * (w / 650.));
            let extents = cairo.text_extents(self.x_label.borrow().as_str()).unwrap();
            cairo.move_to(0.5 * w - extents.width() / 2., 0.925 * h);
            cairo.show_text(self.x_label.borrow().as_str()).unwrap();

            // Draw y-axis label
            let extents = cairo.text_extents(self.y_label.borrow().as_str()).unwrap();
            cairo.move_to(0.035 * w, 0.5 * h + extents.width() / 2.);
            cairo.save().unwrap();
            cairo.rotate(-std::f64::consts::PI / 2.);
            cairo.show_text(self.y_label.borrow().as_str()).unwrap();
            cairo.restore().unwrap();

            // Draw x-axis
            GdkCairoContextExt::set_source_rgba(&cairo, &axis_color);
            cairo.set_line_width(1.);
            cairo.move_to(0.9 * w, 0.8 * h);
            cairo.line_to(0.1 * w, 0.8 * h);
            cairo.stroke().unwrap();

            // Draw y-axis
            cairo.set_line_width(1.);
            cairo.move_to(0.1 * w, 0.8 * h);
            cairo.line_to(0.1 * w, 0.2 * h);
            cairo.stroke().unwrap();

            // Find what is the size of the grid section:
            // get one tenth of the width and find the
            // order of magnitude of the grid
            let grid_section = 0.3 * (x_max - x_min).abs();
            let grid_section_order = grid_section.log10().floor() as i32;
            let grid_section = 10f64.powi(grid_section_order);

            let start_x = w * 0.1;
            let end_x = w * 0.9;

            let start_y = h * 0.2;
            let end_y = h * 0.8;

            // Now draw every grid_section from x_min to x_max
            let mut grid_x = x_min + grid_section;
            while grid_x <= x_max - grid_section / 4. {
                let pos =
                    (grid_x - x_min).abs() / (x_max - x_min).abs() * (end_x - start_x) + start_x;

                // Draw grid x-line
                GdkCairoContextExt::set_source_rgba(&cairo, &grid_color);
                cairo.set_line_width(1.);
                cairo.move_to(pos, start_y);
                cairo.line_to(pos, end_y);
                cairo.stroke().unwrap();

                // Draw grid x-line value
                GdkCairoContextExt::set_source_rgba(&cairo, &text_color);
                let value = format!("{:.1}", grid_x);
                cairo.set_font_size(8.0 * (w / 650.));
                let extents = cairo.text_extents(&value).unwrap();
                cairo.move_to(pos - extents.width() / 2., 0.84 * h);
                cairo.show_text(&value).unwrap();

                grid_x += grid_section;
            }

            // If the last axis is far enough from the end of the graph
            // draw the last axis
            let pos = (x_max - x_min).abs() / (x_max - x_min).abs() * (end_x - start_x) + start_x;
            GdkCairoContextExt::set_source_rgba(&cairo, &grid_color);
            cairo.set_line_width(1.);
            cairo.move_to(pos, start_y);
            cairo.line_to(pos, end_y);
            cairo.stroke().unwrap();

            // and label it
            GdkCairoContextExt::set_source_rgba(&cairo, &text_color);
            let value = format!("{:.1}", x_max);
            cairo.set_font_size(8.0 * (w / 650.));
            let extents = cairo.text_extents(&value).unwrap();
            cairo.move_to(pos - extents.width() / 2., 0.84 * h);
            cairo.show_text(&value).unwrap();

            // Same for y-axis
            let grid_section = 0.4 * (y_max - y_min).abs();
            let grid_section_order = grid_section.log(5.).floor() as i32;
            let grid_section = 5f64.powi(grid_section_order);

            let mut grid_y = y_min + grid_section;
            while grid_y <= y_max - grid_section / 4. {
                let pos =
                    (grid_y - y_min).abs() / (y_max - y_min).abs() * (end_y - start_y) + start_y;

                // Draw grid y-line
                GdkCairoContextExt::set_source_rgba(&cairo, &grid_color);
                cairo.set_line_width(1.);
                cairo.move_to(start_x, h - pos);
                cairo.line_to(end_x, h - pos);
                cairo.stroke().unwrap();

                // Draw grid y-line value
                GdkCairoContextExt::set_source_rgba(&cairo, &text_color);
                let value = format!("{:.1}", grid_y);
                cairo.set_font_size(8.0 * (w / 650.));
                let extents = cairo.text_extents(&value).unwrap();
                cairo.move_to(0.091 * w - extents.width(), h - pos);
                cairo.show_text(&value).unwrap();

                grid_y += grid_section;
            }

            // If the last axis is far enough from the end of the graph
            // draw the last axis
            if (grid_y - y_max - grid_section).abs() > grid_section / 10. {
                let pos =
                    (y_max - y_min).abs() / (y_max - y_min).abs() * (end_y - start_y) + start_y;
                GdkCairoContextExt::set_source_rgba(&cairo, &grid_color);
                cairo.set_line_width(1.);
                cairo.move_to(start_x, h - pos);
                cairo.line_to(end_x, h - pos);
                cairo.stroke().unwrap();

                // and label it
                GdkCairoContextExt::set_source_rgba(&cairo, &text_color);
                let value = format!("{:.1}", y_max);
                cairo.set_font_size(8.0 * (w / 650.));
                let extents = cairo.text_extents(&value).unwrap();
                cairo.move_to(0.091 * w - extents.width(), h - pos);
                cairo.show_text(&value).unwrap();
            }

            // Move coordinate system to bottom left

            // Invert y-axis
            cairo.scale(1., -1.);

            // Move coordinate system to (0,0) of drawn coordinate system
            cairo.translate(0.1 * w, -0.5 * h);
            GdkCairoContextExt::set_source_rgba(&cairo, &line_color);
            cairo.set_line_width(2.0);

            // Calc scales
            let x_scale = (w - 2. * 0.1 * w) / (x_max - x_min).abs();
            let y_scale = (h - 2. * 0.2 * h) / (y_max - y_min).abs();

            // Draw data points from list
            let borrow = &self.point_list.borrow();
            let mut point_iter = borrow.iter();

            if matches!(self.typ.get(), PlotType::Line) {
                // Move to first point to start drawing the line
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
                        cairo.stroke().unwrap();
                        cairo.move_to(point.x * x_scale, point.y * y_scale);
                    }
                    PlotType::Scatter => {
                        // Draw point
                        cairo.set_line_width(3.);
                        cairo.set_line_cap(gtk::cairo::LineCap::Round);
                        cairo.move_to(point.x * x_scale, point.y * y_scale);
                        cairo.close_path();
                        cairo.stroke().unwrap();
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

    pub fn set_title(&self, title: &str) {
        self.imp().title.replace(title.to_string());
        self.queue_draw();
    }
    pub fn set_points(&self, point_list: Vec<Point>) {
        self.imp().point_list.replace(point_list);
        self.queue_draw();
    }
    pub fn set_font_name(&self, font_name: &str) {
        self.imp().font_name.replace(font_name.to_string());
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
    pub fn set_x_label(&self, x_label: &str) {
        self.imp().x_label.replace(x_label.to_string());
        self.queue_draw();
    }
    pub fn set_y_label(&self, y_label: &str) {
        self.imp().y_label.replace(y_label.to_string());
        self.queue_draw();
    }
    pub fn add_point(&self, point: Point) {
        self.imp().point_list.borrow_mut().push(point);
        self.queue_draw();
    }
}
