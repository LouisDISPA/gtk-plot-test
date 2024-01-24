use gtk::{gio, glib};

mod imp {
    use std::time::Duration;

    use adw::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    use crate::plot::{PlotView, Point};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/window.ui")]
    pub struct Window {
        #[template_child]
        pub plot_view: TemplateChild<PlotView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "PlotWindow";
        type Type = super::Window;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let points = (0..500)
                .map(|x| {
                    let x = x as f64 / 10.;
                    Point::new(x, x.sin())
                })
                .collect();
            let test = self.plot_view.clone();
            test.set_points(points);
            // test.set_type(PlotType::Scatter);
            test.set_title("Battery Charge");
            test.set_x_label("Time (h)");
            test.set_y_label("Charge (%)");

            // Gtk widgets are not thread safe, so we need to use spawn_future_local
            // to update the plot from the main thread.
            glib::spawn_future_local(async move {
                for i in 500.. {
                    glib::timeout_future(Duration::from_millis(30)).await;
                    let x = i as f64 / 10.;
                    let y = x.sin();
                    test.add_point(Point::new(x, y));
                }
            });
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(application: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
