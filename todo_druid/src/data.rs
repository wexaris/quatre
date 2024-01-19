use druid::im::Vector;
use druid::text::{Attribute, RichText};
use druid::{Data, Env, EventCtx, FontStyle, Lens, Selector};
use uuid::Uuid;

pub const REBUILD: Selector<Uuid> = Selector::new("todo.rebuild");
pub const SELECT: Selector<Uuid> = Selector::new("todo.select");
pub const UNSELECT: Selector<Uuid> = Selector::new("todo.unselect");
pub const EDIT: Selector<Uuid> = Selector::new("todo.edit");
pub const DELETE: Selector<Uuid> = Selector::new("todo.delete");

#[derive(Clone, Data, Lens)]
pub struct TodoItem {
    #[data(same_fn = "PartialEq::eq")]
    pub id: Uuid,
    pub done: bool,
    pub editing: bool,
    pub selected: bool,
    text: String,
    // We use this to remember what the text was before an edit, in case it's cancelled
    stash: String,
    rendered: RichText,
}

impl TodoItem {
    pub fn new(text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            done: false,
            editing: false,
            selected: false,
            text: text.to_string(),
            stash: text.to_string(),
            rendered: TodoItem::render(text, false),
        }
    }

    pub fn double_click(ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        data.editing = true;
        ctx.request_layout();
        ctx.submit_command(EDIT.with(data.id));
    }

    pub fn select(ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        if !data.selected {
            ctx.submit_command(SELECT.with(data.id));
            ctx.request_focus();
        }
    }

    pub fn delete(ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        ctx.submit_command(DELETE.with(data.id));
    }

    fn render(text: &str, done: bool) -> RichText {
        if done {
            RichText::new(format!("~{}~", text).into())
                .with_attribute(0.., Attribute::style(FontStyle::Italic))
                .with_attribute(0.., Attribute::text_color(druid::theme::PLACEHOLDER_COLOR))
        } else {
            RichText::new(text.into())
        }
    }

    pub fn rebuild(&mut self) {
        self.rendered = Self::render(&self.text, self.done);
    }

    pub fn gain_selection(&mut self) {
        self.selected = true;
        self.editing = true;
        self.stash = self.text.clone();
    }

    pub fn cancel_edit(&mut self) {
        self.text = self.stash.clone();
    }

    pub fn lose_selection(&mut self) {
        self.editing = false;
        self.selected = false;
        self.rendered = Self::render(&self.text, self.done);
    }
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub todos: Vector<TodoItem>,
    pub filtered_ids: Vector<usize>,
    new_todo: String,
    filter: Filter,
    #[data(same_fn = "PartialEq::eq")]
    pub selected: Option<Uuid>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Data)]
enum Filter { All, Active, Completed }

impl AppState {
    pub fn new() -> Self {
        Self {
            todos: Vector::new(),
            filtered_ids: Vector::new(),
            new_todo: String::new(),
            filter: Filter::All,
            selected: None,
        }
    }

    pub fn has_completed_todos(&self) -> bool {
        self.todos.iter().any(|todo| todo.done)
    }

    pub fn add_todo(&mut self) {
        self.todos.push_front(TodoItem::new(&self.new_todo));
        self.new_todo = String::new();
        self.update_filtered();
    }

    pub fn remove_todo(&mut self, id: &Uuid) {
        let mut idx = None;
        for (i, todo) in self.todos.iter().enumerate() {
            if todo.id == *id {
                idx = Some(i);
            }
        }
        if let Some(idx) = idx {
            self.todos.remove(idx);
        }
    }

    pub fn toggle_all(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        let check = data.todos.iter().filter(|todo| !todo.done).count() > 0;
        data.todos.iter_mut().for_each(|todo| todo.done = check);
    }

    pub fn clear_completed(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        let new_todos: Vector<TodoItem> = data.todos.iter()
            .cloned()
            .filter(|item| !item.done)
            .collect();

        data.todos = new_todos;
        data.update_filtered();
    }

    pub fn filter_all(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        data.filter = Filter::All;
        data.update_filtered();
    }

    pub fn filter_active(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        data.filter = Filter::Active;
        data.update_filtered();
    }

    pub fn filter_completed(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        data.filter = Filter::Completed;
        data.update_filtered();
    }

    pub fn update_filtered(&mut self) {
        match self.filter {
            Filter::All => {
                self.filtered_ids = (0..self.todos.len()).collect();
            },
            Filter::Active => {
                self.filtered_ids = self.todos.iter()
                    .enumerate()
                    .filter(|(_, todo)| !todo.done)
                    .map(|(idx, _)| idx)
                    .collect();
            },
            Filter::Completed => {
                self.filtered_ids = self.todos.iter()
                    .enumerate()
                    .filter(|(_, todo)| todo.done)
                    .map(|(idx, _)| idx)
                    .collect();
            },
        }
    }
}
