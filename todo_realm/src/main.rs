use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::gtk::Align;
use relm4::prelude::*;

#[derive(Debug, Clone)]
struct Task {
    idx: DynamicIndex,
    name: String,
    completed: bool,
    is_editing: bool,
}

#[derive(Debug, Clone)]
enum TaskInput {
    Toggle(bool),
    Edit,
    Rename(String),
}

#[derive(Debug, Clone)]
enum TaskOutput {
    Toggle,
    Delete(DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent for Task {
    type Init = String;
    type Input = TaskInput;
    type Output = TaskOutput;
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            gtk::CheckButton {
                set_active: false,
                set_margin_all: 12,
                connect_toggled[sender] => move |checkbox| {
                    sender.input(TaskInput::Toggle(checkbox.is_active()));
                    sender.output(TaskOutput::Toggle);
                }
            },

            #[name(label)]
            gtk::Label {
                set_visible: !self.is_editing,
                #[watch]
                set_label: &self.name,
                set_hexpand: true,
                set_halign: Align::Start,
                set_margin_all: 6,
            },

            #[name(editor)]
            gtk::Entry {
                set_visible: self.is_editing,
                #[watch]
                set_text: &self.name,
                set_hexpand: true,
                set_hexpand_set: true,
                set_halign: Align::Start,
                set_margin_all: 6,

                connect_activate[sender] => move |entry| {
                    sender.input(TaskInput::Rename(entry.buffer().text().to_string()));
                },
            },

            #[name(btn_edit)]
            gtk::Button {
                set_visible: !self.is_editing,
                set_icon_name: "view-refresh",
                set_margin_all: 6,

                connect_clicked[sender] => move |_| {
                    sender.input(TaskInput::Edit);
                }
            },

            #[name(btn_delete)]
            gtk::Button {
                set_visible: self.is_editing,
                set_icon_name: "edit-delete",
                set_margin_all: 6,

                connect_clicked[sender, index] => move |_| {
                    sender.output(TaskOutput::Delete(index.clone()));
                }
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<AppMsg> {
        Some(match output {
            TaskOutput::Toggle => AppMsg::RefreshTaskCount,
            TaskOutput::Delete(index) => AppMsg::DeleteEntry(index),
        })
    }

    fn init_model(name: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            idx: index.clone(),
            name,
            completed: false,
            is_editing: false,
        }
    }

    fn update_with_view(&mut self, widgets: &mut Self::Widgets, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            TaskInput::Toggle(x) => {
                self.completed = x;

                let attrs = widgets.label.attributes().unwrap_or_default();
                attrs.change(gtk::pango::AttrInt::new_strikethrough(self.completed));
                widgets.label.set_attributes(Some(&attrs));
            },
            TaskInput::Edit => {
                self.is_editing = true;

                widgets.label.set_visible(false);
                widgets.editor.set_visible(true);
                widgets.btn_edit.set_visible(false);
                widgets.btn_delete.set_visible(true);
                widgets.editor.grab_focus();
            },
            TaskInput::Rename(name) => {
                self.is_editing = false;
                self.name = name;

                widgets.label.set_text(&self.name);

                widgets.editor.set_visible(false);
                widgets.label.set_visible(true);
                widgets.btn_delete.set_visible(false);
                widgets.btn_edit.set_visible(true);
            },
        }
    }
}

#[derive(Debug, Clone)]
enum AppMsg {
    DeleteEntry(DynamicIndex),
    AddEntry(String),
    ClearComplete,
    RefreshTaskCount,
    SetFilter(Filter),
}

struct App {
    tasks: FactoryVecDeque<Task>,
    task_count: usize,
    filter: Filter,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_width_request: 360,
            set_title: Some("Todos"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 6,

                gtk::Entry {
                    set_placeholder_text: Some("What needs to be done?"),
                    connect_activate[sender] => move |entry| {
                        let buffer = entry.buffer();
                        sender.input(AppMsg::AddEntry(buffer.text().into()));
                        buffer.delete_text(0, None);
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,

                    gtk::Label {
                        #[watch]
                        set_label: {
                            let count = model.task_count;
                            &format!("{} {}", count, if count == 1 { "task" } else { "tasks" })
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                        set_halign: Align::Center,
                        set_spacing: 3,

                        #[name(filter_btn)]
                        gtk::ToggleButton {
                            set_label: "All",
                            set_active: true,
                            connect_toggled[sender] => move |_| {
                                sender.input(AppMsg::SetFilter(Filter::All));
                            }
                        },
                        gtk::ToggleButton {
                            set_label: "Active",
                            set_group: Some(&filter_btn),
                            connect_toggled[sender] => move |_| {
                                sender.input(AppMsg::SetFilter(Filter::Active));
                            }
                        },
                        gtk::ToggleButton {
                            set_label: "Complete",
                            set_group: Some(&filter_btn),
                            connect_toggled[sender] => move |_| {
                                sender.input(AppMsg::SetFilter(Filter::Complete));
                            }
                        }
                    },

                    gtk::Button {
                        set_label: "Clear Complete",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::ClearComplete);
                        }
                    }
                },

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_min_content_height: 360,
                    set_vexpand: true,

                    #[local_ref]
                    task_list_box -> gtk::ListBox {}
                }
            }
        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::DeleteEntry(index) => {
                self.tasks.guard().remove(index.current_index());
                self.recount_tasks();
            }
            AppMsg::AddEntry(name) => {
                self.tasks.guard().push_back(name);
                self.recount_tasks();
            }
            AppMsg::ClearComplete => {
                let to_remove = self.tasks.iter()
                    .filter_map(|todo| todo.completed.then_some(todo.idx.clone()))
                    .collect::<Vec<_>>();
                for idx in to_remove {
                    self.tasks.guard().remove(idx.current_index());
                }

                self.recount_tasks();
            }
            AppMsg::SetFilter(filter) => {
                self.filter = filter;
            }
            AppMsg::RefreshTaskCount => {
                self.recount_tasks();
            }
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            tasks: FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender()),
            filter: Filter::All,
            task_count: 0,
        };

        let task_list_box = model.tasks.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

#[derive(Debug, Clone)]
enum Filter { All, Active, Complete }

impl App {
    fn recount_tasks(&mut self) {
        self.task_count = self.tasks.iter().filter(|todo| !todo.completed).count();
    }
}

fn main() {
    let app = RelmApp::new("Todo");
    app.run::<App>(());
}
