use crate::error::*;
use crate::reminder::*;
use rocket::*;
use rocket::serde::json::Json;
use rocket::tokio::sync::RwLock;

#[derive(Debug)]
struct Store {
    id_counter: u64,
    expired: ReminderSet,
    pending: ReminderSet,
    upcoming: ReminderSet,
}


impl Store {
    fn new() -> Self {
        Store {
            id_counter: 0,
            expired: ReminderSet::new(),
            pending: ReminderSet::new(),
            upcoming: ReminderSet::new(),
        }
    }

    fn add(&mut self, request: ReminderRequest) -> Reminder {
        let reminder = Reminder::new(request, self.id_counter);
        self.upcoming.insert(reminder.clone());
        self.id_counter += 1;
        reminder
    }

    fn update_all(&mut self) {
        // TODO only if last update was some time ago

        let mut new_pending: Vec<_> = self.upcoming.iter()
            .filter(|rm| rm.expired())
            .cloned()
            .collect();
        for np in &mut new_pending {
            self.upcoming.remove(np);
            np.set_state(ReminderState::Pending);
        }
        self.pending.extend(new_pending);
    }

    fn acknowledge(&mut self, id: ReminderId) {
        if let Some(mut rm) = self.pending.iter().find(|rm| rm.id() == id).cloned() {
            self.pending.remove(&rm);
            rm.set_state(ReminderState::Expired);
            self.expired.insert(rm);
        }
    }
}


#[post("/reminders/new", format = "application/json", data = "<reminder>")]
async fn reminders_new(store: &State<RwLock<Store>>, reminder: Json<ReminderRequest>) -> Json<Reminder> {
    Json(store.write().await.add(reminder.0))
}

#[post("/reminders/<id>/ack")]
async fn reminders_ack(store: &State<RwLock<Store>>, id: ReminderId) {
    store.write().await.acknowledge(id)
}

#[get("/reminders/all")]
async fn reminders_all(store: &State<RwLock<Store>>) -> Json<Vec<Reminder>> {
    store.write().await.update_all();

    let mut vector = Vec::new();
    let store = store.read().await;
    vector.extend(store.pending.iter().cloned());
    vector.extend(store.upcoming.iter().cloned());
    vector.extend(store.expired.iter().cloned());
    Json(vector)
}

#[get("/reminders/pending")]
async fn reminders_pending(store: &State<RwLock<Store>>) -> Json<Vec<Reminder>> {
    store.write().await.update_all();

    let mut vector = Vec::new();
    let store = store.read().await;
    vector.extend(store.pending.iter().cloned());
    Json(vector)
}

pub fn serve() -> RemindmeResult<()> {
    let store = RwLock::new(Store::new());

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
