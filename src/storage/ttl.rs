use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{error, info};
use time::OffsetDateTime;
use tokio::sync::Notify;

use crate::storage::DBEvent;

use super::{db::DB, DBError};

pub struct KeyEvictor {
    expiries: Arc<Mutex<BTreeSet<(OffsetDateTime, String)>>>,
    db: Arc<DB>,
    eviction_notifier: Arc<Notify>,
}

impl KeyEvictor {
    pub fn new(db: Arc<DB>, eviction_notifier: Arc<Notify>) -> KeyEvictor {
        KeyEvictor {
            expiries: Arc::new(Mutex::new(BTreeSet::new())),
            db,
            eviction_notifier,
        }
    }

    pub async fn run(&mut self) {
        let mut db_events_rx = self.db.subscribe_events();
        let expiries_arc = self.expiries.clone();
        let eviction_notifier_arc = self.eviction_notifier.clone();

        tokio::spawn(async move {
            loop {
                let expiries = expiries_arc.clone();
                let eviction_notifier = eviction_notifier_arc.clone();

                // let db_event = db_events_rx.recv().await;
                info!("key evictor db_event");
                match db_events_rx.recv().await {
                    Ok(evt) => match evt {
                        DBEvent::SetKeyExpiry(key_exp) => {
                            if let Err(e) = Self::update_key_expiry(key_exp, expiries) {
                                error!("Error while updating key expiry: {}", e);
                            }

                            eviction_notifier.notify_one();
                        }
                    },
                    Err(e) => {
                        error!("Error while receiving DB events: {}", e);
                    }
                }
            }
        });

        loop {
            info!("key evictor loop");
            let utc_now = OffsetDateTime::now_utc();
            let next_eviction_utc_ts = match self.evict_keys(utc_now) {
                Ok(when) => when,
                Err(e) => {
                    error!("Key eviction failed due to: {}", e);
                    None
                }
            };

            if let Some(next_trigger_utc_ts) = next_eviction_utc_ts {
                info!("key evictor trigger");
                let next_trigger_ts_diff =
                    (next_trigger_utc_ts - utc_now).whole_milliseconds() as u64;
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(next_trigger_ts_diff)) => {}
                    _ = self.eviction_notifier.notified() => {}
                }
            } else {
                self.eviction_notifier.notified().await;
            }
        }
    }

    fn update_key_expiry(
        key_expiry: (OffsetDateTime, String),
        expiries: Arc<Mutex<BTreeSet<(OffsetDateTime, String)>>>,
    ) -> Result<(), DBError> {
        let mut expiries = match expiries.lock() {
            Ok(exp) => exp,
            Err(e) => {
                error!("Failed to evict expired keys: {}", e);
                return Err(DBError::Other("Failed to evict expired keys".to_string()));
            }
        };

        expiries.insert(key_expiry);

        Ok(())
    }

    fn evict_keys(
        &mut self,
        expire_till: OffsetDateTime,
    ) -> Result<Option<OffsetDateTime>, DBError> {
        let mut expiries = match self.expiries.lock() {
            Ok(exp) => exp,
            Err(e) => {
                error!("Failed to evict expired keys: {}", e);
                return Err(DBError::Other("Failed to evict expired keys".to_string()));
            }
        };

        while let Some((when, key)) = expiries.first().cloned() {
            if when > expire_till {
                return Ok(Some(when));
            }

            self.db.del(key.as_str())?;
            expiries.remove(&(when, key));
        }

        Ok(None)
    }
}
