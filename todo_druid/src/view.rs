use druid::{theme::*, widget::Painter, widget::Scroll, widget::{Button, Checkbox, Either, Flex, List, RawLabel, TextBox}, Color, Insets, RenderContext, Widget, WidgetExt, LensExt, lens, UnitPoint};
use druid::im::Vector;
use druid::widget::{DisabledIf, Label};

use crate::controllers::{AddTodoController, TodoItemController};
use crate::data::{AppState, TodoItem};
use crate::double_click::DoubleClick;

pub fn todo_item() -> impl Widget<TodoItem> {
    let painter = Painter::new(move |ctx, data: &TodoItem, env| {
        let selected = data.selected;
        let bounds = ctx.size().to_rect().inset(-2.).to_rounded_rect(3.);
        if ctx.is_hot() && !ctx.is_active() {
            ctx.fill(bounds, &Color::rgb8(0xEE, 0xEE, 0xEE))
        } else {
            ctx.fill(bounds, &Color::WHITE)
        }

        if selected {
            ctx.fill(bounds, &Color::rgb8(0xEE, 0xEE, 0xEE));
            ctx.stroke(bounds, &env.get(PRIMARY_LIGHT), 2.);
        }
    });

    let checkbox = Checkbox::new("").lens(TodoItem::done);

    let delete_btn = Button::new("x").on_click(TodoItem::delete);

    let label = RawLabel::new()
        .with_text_size(TEXT_SIZE_NORMAL)
        .padding(Insets::new(-2., 0., -2., 0.))
        .lens(TodoItem::rendered)
        .env_scope(|env, _data| {
            env.set(TEXT_SIZE_NORMAL, 20.);
            env.set(TEXT_COLOR, Color::BLACK);
            env.set(CURSOR_COLOR, Color::BLACK);
            env.set(PRIMARY_LIGHT, Color::BLACK);
        });

    let text_box = TextBox::new()
        .with_text_size(TEXT_SIZE_NORMAL)
        .lens(TodoItem::text)
        .expand_width()
        .env_scope(|env, _data| {
            env.set(TEXT_SIZE_NORMAL, 20.);
            env.set(TEXTBOX_INSETS, 2.5);
            env.set(TEXTBOX_BORDER_WIDTH, 0.);
            env.set(TEXT_COLOR, Color::BLACK);
            env.set(CURSOR_COLOR, Color::BLACK);
            env.set(BACKGROUND_LIGHT, Color::rgb8(0xEE, 0xEE, 0xEE));
            env.set(SELECTED_TEXT_BACKGROUND_COLOR, env.get(PRIMARY_DARK));
        });
    let edit_label = Flex::row().with_flex_child(text_box, 1.);

    let either = Either::new(
        |data, _env| data.selected && data.editing,
        edit_label, label)

    .controller(TodoItemController)
    .expand_width()
    .controller(DoubleClick::new(TodoItem::double_click));

    Flex::row()
        .with_child(checkbox)
        .with_spacer(5.)
        .with_flex_child(either, 1.)
        .with_child(delete_btn)
        .background(painter)
        .on_click(TodoItem::select)
}

pub fn build_ui() -> impl Widget<AppState> {
    let title = Label::new("todos")
        .with_text_size(TEXT_SIZE_LARGE)
        .align_horizontal(UnitPoint::CENTER)
        .env_scope(|env, _data| {
            env.set(TEXT_SIZE_LARGE, 75.);
            env.set(TEXT_COLOR, Color::rgb8(0x88, 0x88, 0x99));
        });

    let toggle_all_btn = Button::new("Ø")
        .on_click(AppState::toggle_all);

    let new_todo_textbox = TextBox::new()
        .with_placeholder("What needs to be done?")
        .with_text_size(TEXT_SIZE_LARGE)
        .expand_width()
        .lens(AppState::new_todo)
        .env_scope(|env, _data| {
            env.set(TEXTBOX_INSETS, (10., 5., 10., 5.));
            env.set(TEXTBOX_BORDER_WIDTH, 2.);
            env.set(BACKGROUND_LIGHT, Color::WHITE);
            env.set(TEXT_COLOR, Color::BLACK);
            env.set(CURSOR_COLOR, Color::BLACK);
            env.set(PRIMARY_LIGHT, Color::BLACK);
            env.set(BORDER_DARK, Color::BLACK);
        })
        .controller(AddTodoController);

    let create = Flex::row()
        .with_child(toggle_all_btn)
        .with_spacer(5.)
        .with_flex_child(new_todo_textbox, 1.)
        .padding(10.)
        .background(Color::WHITE);

    let todo_list = List::new(todo_item)
        .with_spacing(5.)
        .padding(10.)
        .lens(lens::Identity.map(
            |d: &AppState| d.todos
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, todo)| d.filtered_ids.contains(&idx).then_some(todo))
                    .cloned()
                    .collect(),
            |d: &mut AppState, list: Vector<TodoItem>| {
                for todo in list {
                    if let Some(d) = d.todos.iter_mut().find(|x| x.id == todo.id) {
                        *d = todo;
                    }
                }
            }
        ));

    let task_count = Label::dynamic(|data: &AppState, _| {
        let count = data.todos.iter()
            .filter(|todo| !todo.done)
            .count();
        format!("{} {}", count, if count == 1 { "task" } else { "tasks" })
    })
        .padding(10.)
        .env_scope(|env, _data| {
            env.set(TEXT_COLOR, Color::BLACK);
        });

    let filters = Flex::row()
        .with_flex_child(
            Button::new("All")
                .on_click(AppState::filter_all)
                .expand_width().padding(2.), 1.)
        .with_flex_child(
            Button::new("Active")
                .on_click(AppState::filter_active)
                .expand_width().padding(2.), 1.)
        .with_flex_child(
            Button::new("Completed")
                .on_click(AppState::filter_completed)
                .expand_width().padding(2.), 1.)
        .padding(10.);

    let clear_completed_button = Button::new("Clear Complete")
        .on_click(AppState::clear_completed)
        .padding(10.)
        .env_scope(|env, _data| {
            env.set(FOREGROUND_LIGHT, Color::WHITE);
            env.set(CURSOR_COLOR, Color::BLACK);
            env.set(PRIMARY_DARK, Color::BLACK);
            env.set(BORDER_DARK, Color::BLACK);
        });

    let mby_clear_completed = DisabledIf::new(
        clear_completed_button,|data, _| !data.has_completed_todos());

    let actions_row = Flex::row()
        .with_child(task_count)
        .with_spacer(5.)
        .with_flex_child(filters, 1.)
        .with_spacer(5.)
        .with_child(mby_clear_completed);

    Flex::column()
        .with_flex_child(
            Flex::column()
                .with_child(title)
                .with_child(create)
                .with_child(actions_row)
                .with_flex_child(Scroll::new(todo_list).vertical(), 1.)
                .padding((15., 30., 15., 30.)),
            1.,
        )
        .background(Color::WHITE)
}
