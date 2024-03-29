use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Target};

use crate::data::{AppState, DELETE, REBUILD, SELECT, UNSELECT};

pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(id) = cmd.get(SELECT) {
            data.selected = Some(id.clone());
            for todo in data.todos.iter_mut() {
                if id == &todo.id {
                    todo.gain_selection();
                } else {
                    todo.lose_selection();
                }
            }
            Handled::Yes
        } else if let Some(id) = cmd.get(UNSELECT) {
            data.selected = None;
            for todo in data.todos.iter_mut() {
                if id == &todo.id {
                    todo.lose_selection();
                }
            }
            Handled::Yes
        } else if let Some(id) = cmd.get(REBUILD) {
            for todo in data.todos.iter_mut() {
                if id == &todo.id {
                    todo.rebuild();
                }
            }
            Handled::Yes
        } else if let Some(id) = cmd.get(DELETE) {
            data.remove_todo(id);
            Handled::Yes
        } else {
            println!("cmd forwarded: {:?}", cmd);
            Handled::No
        }
    }
}
