use iced::alignment::{self, Alignment, Horizontal};
use iced::theme::{self, Theme};
use iced::widget::{self, button, checkbox, column, container, row, scrollable, text, text_input};
use iced::window;
use iced::{Application, Element};
use iced::{Color, Command, Length, Settings, Subscription};
use iced::keyboard::{Event, KeyCode};
use iced::subscription::events_with;

pub fn main() -> iced::Result {
    Todos::run(Settings {
        window: window::Settings {
            size: (500, 800),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug)]
struct Todos {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    CreateTask,
    FilterChanged(Filter),
    TaskMessage(usize, TaskMessage),
    SelectAll,
    DeleteCompleted,
    TabPressed { shift: bool },
}

impl Application for Todos {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Todos, Command<Message>) {
        (Todos {
            input_value: "".to_string(),
            filter: Filter::All,
            tasks: vec![],
        }, Command::none())
    }

    fn title(&self) -> String { "Iced app".to_string() }

    fn update(&mut self, message: Message) -> Command<Message> {
        let command = match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Command::none()
            }
            Message::CreateTask => {
                if !self.input_value.is_empty() {
                    self.tasks.push(Task::new(self.input_value.clone()));
                    self.input_value.clear();
                }
                Command::none()
            }
            Message::FilterChanged(filter) => {
                self.filter = filter;
                Command::none()
            }
            Message::SelectAll => {
                let check = self.tasks.iter().filter(|task| !task.completed).count() > 0;
                self.tasks.iter_mut().for_each(|task| task.completed = check);
                Command::none()
            }
            Message::DeleteCompleted => {
                self.tasks = self.tasks.iter()
                    .filter(|task| !task.completed)
                    .cloned()
                    .collect();
                Command::none()
            }
            Message::TaskMessage(i, TaskMessage::Delete) => {
                self.tasks.remove(i);
                Command::none()
            }
            Message::TaskMessage(i, task_message) => {
                if let Some(task) = self.tasks.get_mut(i) {
                    let should_focus = matches!(task_message, TaskMessage::Edit);

                    task.update(task_message);

                    if should_focus {
                        let id = Task::text_input_id(i);
                        Command::batch(vec![
                            text_input::focus(id.clone()),
                            text_input::select_all(id),
                        ])
                    } else {
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::TabPressed { shift } => {
                if shift {
                    widget::focus_previous()
                } else {
                    widget::focus_next()
                }
            }
        };

        Command::batch(vec![command])
    }

    fn view(&self) -> Element<Message> {
        let title = text("todos")
            .width(Length::Fill)
            .size(100)
            .style(Color::from([0.5, 0.5, 0.5]))
            .horizontal_alignment(alignment::Horizontal::Center);

        let check_all = button(text("Ø").size(30))
            .style(theme::Button::Text)
            .padding([15, 15, 30, 0])
            .on_press(Message::SelectAll);

        let input = text_input("What needs to be done?", &self.input_value, Message::InputChanged)
            .on_submit(Message::CreateTask)
            .padding(15)
            .size(30);

        let input_line = row![check_all, input];

        let controls = view_controls(&self.tasks, self.filter);
        let filtered_tasks = self.tasks.iter()
            .filter(|task| self.filter.matches(task));

        let tasks: Option<Element<_>> = (filtered_tasks.count() > 0).then_some(
            column(self.tasks.iter()
                .enumerate()
                .filter(|(_, task)| self.filter.matches(task))
                .map(|(i, task)| {
                    task.view(i).map(move |message| {
                        Message::TaskMessage(i, message)
                    })
                })
                .collect())
                .spacing(10)
                .into()
        );

        let content = if let Some(tasks) = tasks {
            column![title, input_line, controls, tasks]
        } else {
            column![title, input_line, controls]
        }
            .spacing(20)
            .max_width(800);

        scrollable(container(content).padding(40).center_x()).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        events_with(|evt, _| {
            match evt {
                iced::Event::Keyboard(Event::KeyPressed { key_code, modifiers}) => {
                    match (key_code, modifiers) {
                        (KeyCode::Tab, _) => {
                            Some(Message::TabPressed { shift: modifiers.shift() })
                        },
                        _ => None,
                    }
                }
                _ => None,
            }
        })
    }
}

#[derive(Debug, Clone)]
struct Task {
    description: String,
    completed: bool,
    is_editing: bool,
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

impl Task {
    fn text_input_id(i: usize) -> text_input::Id {
        text_input::Id::new(format!("task-{i}"))
    }

    fn new(description: String) -> Self {
        Task {
            description,
            completed: false,
            is_editing: false,
        }
    }

    fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Completed(completed) => {
                self.completed = completed;
            }
            TaskMessage::Edit => {
                self.is_editing = true;
            }
            TaskMessage::DescriptionEdited(new_description) => {
                self.description = new_description;
            }
            TaskMessage::FinishEdition => {
                if !self.description.is_empty() {
                    self.is_editing = false;
                }
            }
            TaskMessage::Delete => {}
        }
    }

    fn view(&self, i: usize) -> Element<TaskMessage> {
        if self.is_editing {
            let text_input =
                text_input("", &self.description, TaskMessage::DescriptionEdited)
                    .id(Self::text_input_id(i))
                    .on_submit(TaskMessage::FinishEdition)
                    .padding(10);

            row![
                text_input,
                button("×")
                .on_press(TaskMessage::Delete)
                .padding(10)
                .style(theme::Button::Destructive)
            ]
                .spacing(20)
                .align_items(Alignment::Center)
                .into()
        }
        else {
            let formatted = if self.completed {
                format!("~{}~", self.description)
            } else {
                self.description.clone()
            };

            let checkbox = checkbox(
                &formatted,
                self.completed,
                TaskMessage::Completed,
            ).width(Length::Fill);

            row![
                checkbox,
                button("edit")
                .on_press(TaskMessage::Edit)
                .padding(10)
                .style(theme::Button::Text),
            ]
                .spacing(20)
                .align_items(Alignment::Center)
                .into()
        }
    }
}

fn view_controls(tasks: &[Task], current_filter: Filter) -> Element<Message> {
    let tasks_left = tasks.iter().filter(|task| !task.completed).count();

    let filter_button = |label, filter, current_filter| {
        let label = text(label);
        let button = button(label).style(
            if filter == current_filter {
                theme::Button::Primary
            } else {
                theme::Button::Text
            }
        );

        button.on_press(Message::FilterChanged(filter)).padding(8)
    };

    let clear_btn = button(text("Clear Complete"))
        .on_press(Message::DeleteCompleted)
        .padding(8);

    let clear_cont = iced::widget::container(clear_btn)
        .align_x(Horizontal::Right);

    row![
        text(format!("{tasks_left} {}", if tasks_left == 1 { "task" } else { "tasks" }))
            .width(Length::Fill),
        row![
            filter_button("All", Filter::All, current_filter),
            filter_button("Active", Filter::Active, current_filter),
            filter_button("Completed", Filter::Completed, current_filter,),
        ]
            .width(Length::Shrink)
            .spacing(10),
        clear_cont.width(Length::Fill),
    ]
        .spacing(20)
        .align_items(Alignment::Center)
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter { All, Active, Completed }

impl Filter {
    fn matches(self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.completed,
            Filter::Completed => task.completed,
        }
    }
}
