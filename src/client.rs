use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use colored::Colorize;
use reqwest::StatusCode;

use crate::reminder::*;
use crate::time::*;
use crate::error::*;

const BASE_URL: &str = "http://localhost:8000";

pub struct Client {
    reqwest: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Client {
            reqwest: reqwest::Client::new()
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    async fn post_new(&self, time: Time, message: String) -> RemindmeResult<Reminder>{
        let request = ReminderRequest::new(time, message);
        let response = self.reqwest.post(format!("{}/reminders/new", BASE_URL))
            .json(&request)
            .send()
            .await?
            .json()
        .await?;
        Ok(response)
    }

    async fn post_ack(&self, id: ReminderId) -> RemindmeResult<StatusCode> {
        let response = self.reqwest.post(format!("{}/reminders/{}/ack", BASE_URL, id))
            .send()
            .await?
            .status();
        Ok(response)
    }

    async fn get_all(&self) -> RemindmeResult<Vec<Reminder>>{
        let response = self.reqwest.get(format!("{}/reminders/all", BASE_URL))
            .send()
            .await?
            .json()
        .await?;
        Ok(response)
    }

    async fn get_pending(&self) -> RemindmeResult<Vec<Reminder>>{
        let response = self.reqwest.get(format!("{}/reminders/pending", BASE_URL))
            .send()
            .await?
            .json()
        .await?;
        Ok(response)
    }

    pub fn create(&self, time: Time, message: String) -> RemindmeResult<()> {
        let result = Self::runtime()
            .block_on(async { self.post_new(time, message).await })?;
        println!("{:#?}", result);
        Ok(())
    }

    pub fn list(&self) -> RemindmeResult<()> {
        let mut reminders = Self::runtime()
            .block_on(async { self.get_all().await })?;
        reminders.sort_by_key(|rm| rm.date());

        for rm in reminders {
            let line = format!("{}    {}", rm.date().format("%d.%m.%Y %H:%M"), rm.message());
            match rm.state() {
                ReminderState::Upcoming => println!("{}", line),
                ReminderState::Pending => println!("{}", line.red()),
                ReminderState::Expired => println!("{}", line.bright_black()),
            }
        }
        Ok(())
    }


    pub fn fetch(&self) -> RemindmeResult<()> {
        let runtime = Self::runtime();
        let mut reminders = runtime.block_on(async { self.get_pending().await })?;

        reminders.retain(|rm| rm.state() == ReminderState::Pending);  // should be the case anyway
        reminders.sort_by_key(|rm| rm.date());

        for rm in reminders {
            println!("{}    {}", rm.date().format("%d.%m.%Y %H:%M").to_string().bright_red(), rm.message());
            println!();

            runtime.block_on(async { self.post_ack(rm.id()).await })?;
        }
        Ok(())
    }

    pub fn listen(&self, interval: u64, command: Option<String>) -> RemindmeResult<()> {
        let runtime = Self::runtime();
        let sleep_duration = Duration::new(interval, 0);

        loop {
            let fetch_start = Instant::now();

            for rm in runtime.block_on(async { self.get_pending().await })? {
                if let Some(cmd) = &command {
                    let mut proc = Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .stdin(Stdio::piped())
                        .stderr(Stdio::inherit())
                        .spawn()
                        .unwrap();

                    let mut stream = proc.stdin.take().unwrap();
                    writeln!(stream, "{}", rm.message())?;
                    drop(stream);

                    let status = proc.wait()?;
                    if !status.success() {
                        return Err(RemindmeError::CommandExit(status.code().unwrap_or(-1)))
                    }
                }

                println!("{}    {}", rm.date().format("%d.%m.%Y %H:%M").to_string().bright_red(), rm.message());
                println!();

                runtime.block_on(async { self.post_ack(rm.id()).await })?;
            }

            let fetch_duration = Instant::now() - fetch_start;
            thread::sleep(sleep_duration - fetch_duration);
        }
    }
}
