use druid::widget::{Controller, Either};
use druid::{Env, Event, EventCtx, HotKey, KbKey, UpdateCtx, Widget};

use crate::data::*;

pub struct AddTodoController;

impl<W: Widget<AppState>> Controller<AppState, W> for AddTodoController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(k_e) if HotKey::new(None, KbKey::Enter).matches(k_e) => {
                data.add_todo();
                ctx.set_handled();
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}
pub struct TodoItemController;

impl Controller<TodoItem, Either<TodoItem>> for TodoItemController {
    fn event(
        &mut self,
        child: &mut Either<TodoItem>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut TodoItem,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(k_e) if HotKey::new(None, KbKey::Enter).matches(k_e) => {
                ctx.submit_command(UNSELECT.with(data.id));
                ctx.set_handled();
            }
            Event::KeyDown(k_e) if HotKey::new(None, KbKey::Tab).matches(k_e) => {
                ctx.submit_command(UNSELECT.with(data.id));
                ctx.set_handled();
            }
            Event::KeyDown(k_e) if HotKey::new(None, KbKey::Escape).matches(k_e) => {
                ctx.submit_command(UNSELECT.with(data.id));
                data.cancel_edit();
                ctx.set_handled();
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }

    fn update(
        &mut self,
        child: &mut Either<TodoItem>,
        ctx: &mut UpdateCtx,
        old_data: &TodoItem,
        data: &TodoItem,
        env: &Env,
    ) {
        // Limit our rebuilds and saves to when we lose selection or toggle the checkbox
        if old_data.done != data.done {
            ctx.submit_command(REBUILD.with(data.id));
            ctx.request_layout();
        }
        child.update(ctx, old_data, data, env);
    }
}
