#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_elements::input_data::keyboard_types::Key;

fn main() {
    dioxus_desktop::launch(app);
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FilterState {
    All,
    Active,
    Completed,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TodoItem {
    pub id: u32,
    pub checked: bool,
    pub contents: String,
}

pub fn app(cx: Scope<()>) -> Element {
    let todos = use_state(cx, im_rc::HashMap::<u32, TodoItem>::default);
    let list_filter = use_state(cx, || FilterState::All);

    // Filter the todos based on the filter state
    let mut filtered_todos = todos.iter()
        .filter(|(_, item)| match **list_filter {
            FilterState::All => true,
            FilterState::Active => !item.checked,
            FilterState::Completed => item.checked,
        })
        .map(|f| *f.0)
        .collect::<Vec<_>>();
    filtered_todos.sort_unstable();

    let active_todo_count = todos.values().filter(|item| !item.checked).count();
    let active_todo_text = if active_todo_count == 1 { "task" } else { "tasks" };

    let show_clear_completed = todos.values().any(|todo| todo.checked);

    render! {
        section { class: "todoapp",
            style { { include_str!("../assets/style.css") } }
            TodoHeader { todos: todos }
            section { class: "main",
                if !todos.is_empty() { rsx! {
                    ListToolbar {
                        active_todo_count: active_todo_count,
                        active_todo_text: active_todo_text,
                        show_clear_completed: show_clear_completed,
                        todos: todos,
                        list_filter: list_filter,
                    }
                    input {
                        id: "toggle-all",
                        class: "toggle-all",
                        r#type: "checkbox",
                        onchange: move |_| {
                            let check = active_todo_count != 0;
                            for (_, item) in todos.make_mut().iter_mut() {
                                item.checked = check;
                            }
                        },
                        checked: if active_todo_count == 0 { "true" } else { "false" },
                    }
                    label { r#for: "toggle-all" }
                }}
                ul { class: "todo-list",
                    for id in filtered_todos.iter() {
                        TodoEntry {
                            key: "{id}",
                            id: *id,
                            todos: todos,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props)]
pub struct TodoHeaderProps<'a> {
    todos: &'a UseState<im_rc::HashMap<u32, TodoItem>>,
}

pub fn TodoHeader<'a>(cx: Scope<'a, TodoHeaderProps<'a>>) -> Element {
    let draft = use_state(cx, || "".to_string());
    let todo_id = use_state(cx, || 0);

    render! {
        header { class: "header",
            h1 { "todos" }
            input {
                class: "new-todo",
                placeholder: "What needs to be done?",
                value: "{draft}",
                autofocus: "true",
                oninput: move |evt| draft.set(evt.value.clone()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter && !draft.is_empty() {
                        cx.props.todos
                            .make_mut()
                            .insert(
                                **todo_id,
                                TodoItem {
                                    id: **todo_id,
                                    checked: false,
                                    contents: draft.to_string(),
                                },
                            );
                        *todo_id.make_mut() += 1;
                        draft.set("".to_string());
                    }
                }
            }
        }
    }
}

#[derive(Props)]
pub struct TodoEntryProps<'a> {
    todos: &'a UseState<im_rc::HashMap<u32, TodoItem>>,
    id: u32,
}

pub fn TodoEntry<'a>(cx: Scope<'a, TodoEntryProps<'a>>) -> Element {
    let is_editing = use_state(cx, || false);

    let todos = cx.props.todos.get();
    let todo = &todos[&cx.props.id];
    let completed = if todo.checked { "completed" } else { "" };
    let editing = if **is_editing { "editing" } else { "" };

    render! {
        li { class: "{completed} {editing}",
            div { class: "view",
                input {
                    class: "toggle",
                    r#type: "checkbox",
                    id: "cbg-{todo.id}",
                    checked: "{todo.checked}",
                    oninput: move |evt| {
                        let mut todos = cx.props.todos.make_mut();
                        todos[&cx.props.id].checked = evt.value.parse().unwrap();
                    }
                }
                label {
                    r#for: "cbg-{todo.id}",
                    ondoubleclick: move |_| is_editing.set(true),
                    prevent_default: "onclick",
                    "{todo.contents}"
                }
                button {
                    class: "change",
                    onclick: move |_| is_editing.set(true),
                    prevent_default: "onclick"
                }
                button {
                    class: "destroy",
                    onclick: move |_| {
                        cx.props.todos.make_mut().remove(&todo.id);
                    },
                    prevent_default: "onclick"
                }
            }
            if **is_editing { rsx! {
                input {
                    class: "edit",
                    value: "{todo.contents}",
                    oninput: move |evt| {
                        let mut todos = cx.props.todos.make_mut();
                        todos[&cx.props.id].contents = evt.value.clone()
                    },
                    autofocus: "true",
                    onfocusout: move |_| is_editing.set(false),
                    onkeydown: move |evt| {
                        if matches!(evt.key(), Key::Enter | Key::Escape | Key::Tab) {
                            is_editing.set(false)
                        }
                    },
                }
            }}
        }
    }
}

#[derive(Props)]
pub struct ListToolbarProps<'a> {
    todos: &'a UseState<im_rc::HashMap<u32, TodoItem>>,
    active_todo_count: usize,
    active_todo_text: &'a str,
    show_clear_completed: bool,
    list_filter: &'a UseState<FilterState>,
}

pub fn ListToolbar<'a>(cx: Scope<'a, ListToolbarProps<'a>>) -> Element {
    let active_todo_count = cx.props.active_todo_count;
    let active_todo_text = cx.props.active_todo_text;

    let selected = |state| {
        if *cx.props.list_filter == state { "selected" } else { "false" }
    };

    render! {
        footer { class: "footer",
            span { class: "todo-count",
                strong { "{active_todo_count} " }
                span { "{active_todo_text}" }
            }
            ul { class: "filters",
                for (state , state_text , url) in [
                    (FilterState::All, "All", "#/"),
                    (FilterState::Active, "Active", "#/active"),
                    (FilterState::Completed, "Completed", "#/completed"),
                ] { rsx! {
                    li {
                        a {
                            href: url,
                            class: selected(state),
                            onclick: move |_| cx.props.list_filter.set(state),
                            prevent_default: "onclick",
                            {state_text}
                        }
                    }
                }}
            }
            if cx.props.show_clear_completed { rsx! {
                button {
                    class: "clear-completed",
                    onclick: move |_| {
                        let mut todos = cx.props.todos.make_mut();
                        todos.retain(|_, todo| !todo.checked)
                    },
                    "Clear Complete"
                }
            }}
        }
    }
}
