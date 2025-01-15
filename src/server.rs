use std::collections::HashMap;

use crate::config;
use crate::error::*;
use crate::reminder::*;
use rocket::get;
use rocket::post;
use rocket::response::status;
use rocket::routes;
use rocket::serde::json::Json;
use rocket::tokio::sync::RwLock;
use rocket::State;
use rocket_basicauth::BasicAuth;
use serde::Deserialize;

type UserToken = String;

#[derive(Debug)]
struct Store {
    id_counter: u64,
    config: ServerConfig,
    user_states: HashMap<String, UserState>,
    user_auth: HashMap<String, UserToken>,
}

#[derive(Debug)]
struct UserState {
    expired: ReminderSet,
    pending: ReminderSet,
    upcoming: ReminderSet,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    users: Vec<User>,
}

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    auth: UserToken,
}

impl Store {
    fn new() -> Self {
        Store {
            id_counter: 0,
            config: ServerConfig::default(),
            user_states: HashMap::from([("test".to_string(), UserState::new())]),
            user_auth: HashMap::from([("test".to_string(), "test".to_string())]),
        }
    }

    fn new_with_config(config: ServerConfig) -> Self {
        let mut store = Self::new();

        for user in &config.users {
            store.user_auth.insert(user.name.to_string(), user.auth.to_string());
            store.user_states.insert(user.name.to_string(), UserState::new());
        }

        store.config = config;
        store
    }

    fn add(&mut self, user: &str, request: ReminderRequest) -> Result<Reminder, ()> {
        let reminder = Reminder::new(request, self.id_counter);
        self.user_mut(user).ok_or(())?
            .upcoming.insert(reminder.clone());
        self.id_counter += 1;
        Ok(reminder)
    }

    fn check_user_auth(&self, name: &str, pass: &str) -> bool {
        println!("checking '{}:{}'", name, pass);
        match self.user_auth.get(name) {
            Some(p) => p == pass,
            None => false,
        }
    }

    fn user(&self, name: &str) -> Option<&UserState> {
        self.user_states.get(name)
    }

    fn user_mut(&mut self, name: &str) -> Option<&mut UserState> {
        self.user_states.get_mut(name)
    }

    fn update_all(&mut self) {
        // TODO only if last update was some time ago

        for state in self.user_states.values_mut() {
            let mut new_pending: Vec<_> = state.upcoming.iter()
                .filter(|rm| rm.expired())
                .cloned()
                .collect();
            for np in &mut new_pending {
                state.upcoming.remove(np);
                np.set_state(ReminderState::Pending);
            }
            state.pending.extend(new_pending);
        }
    }

    fn acknowledge(&mut self, user: &str, id: ReminderId) -> Result<(), ()> {
        let state = self.user_mut(user).ok_or(())?;
        let reminder_opt = state.pending.iter()
            .find(|rm| rm.id() == id)
            .cloned();
        if let Some(mut rm) = reminder_opt {
            state.pending.remove(&rm);
            rm.set_state(ReminderState::Expired);
            state.expired.insert(rm);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl UserState {
    fn new() -> Self {
        UserState {
            expired: ReminderSet::new(),
            pending: ReminderSet::new(),
            upcoming: ReminderSet::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            users: Vec::new(),
        }
    }
}


#[post("/reminders/new", format = "application/json", data = "<reminder>")]
async fn reminders_new(auth: BasicAuth, store: &State<RwLock<Store>>, reminder: Json<ReminderRequest>) -> Result<Json<Reminder>, status::Unauthorized<()>> {
    if !store.read().await.check_user_auth(&auth.username, &auth.password) {
        return Err(status::Unauthorized(()));
    }

    let reminder = store.write().await.add(&auth.username, reminder.0)
        .map_err(|_| status::Unauthorized(()))?;
    Ok(Json(reminder))
}

#[post("/reminders/<id>/ack")]
async fn reminders_ack(auth: BasicAuth, store: &State<RwLock<Store>>, id: ReminderId) -> Result<(), status::Unauthorized<()>> {
    if !store.read().await.check_user_auth(&auth.username, &auth.password) {
        return Err(status::Unauthorized(()));
    }

    store.write().await.acknowledge(&auth.username, id)
        .map_err(status::Unauthorized)?;
    Ok(())
}

#[get("/reminders/all")]
async fn reminders_all(auth: BasicAuth, store: &State<RwLock<Store>>) -> Result<Json<Vec<Reminder>>, status::Unauthorized<()>> {
    if !store.read().await.check_user_auth(&auth.username, &auth.password) {
        return Err(status::Unauthorized(()));
    }

    store.write().await.update_all();

    let mut vector = Vec::new();
    let store = store.read().await;
    let user_state = store.user(&auth.username)
        .ok_or(status::Unauthorized(()))?;
    vector.extend(user_state.pending.iter().cloned());
    vector.extend(user_state.upcoming.iter().cloned());
    vector.extend(user_state.expired.iter().cloned());
    Ok(Json(vector))
}

#[get("/reminders/pending")]
async fn reminders_pending(auth: BasicAuth, store: &State<RwLock<Store>>) -> Result<Json<Vec<Reminder>>, status::Unauthorized<()>> {
    if !store.read().await.check_user_auth(&auth.username, &auth.password) {
        return Err(status::Unauthorized(()));
    }

    store.write().await.update_all();

    let mut vector = Vec::new();
    let store = store.read().await;
    let user_state = store.user(&auth.username)
        .ok_or(status::Unauthorized(()))?;
    vector.extend(user_state.pending.iter().cloned());
    Ok(Json(vector))
}

pub fn serve() -> RemindmeResult<()> {
    let config: ServerConfig = config::read_config("server")?;
    let store = RwLock::new(Store::new_with_config(config));

    let rkt = rocket::build()
        .manage(store)
        .mount("/", routes![
            reminders_new,
            reminders_ack,
            reminders_all,
            reminders_pending,
        ]);
    let _ = rocket::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { rkt.launch().await });
    todo!()
}
