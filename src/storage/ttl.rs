use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::error;
use time::OffsetDateTime;
use tokio::sync::Notify;

use crate::storage::DBEvent;

use super::{db::DB, DBError};

/// KeyEvictor maintains the TTL (Time To Live) for each key and runs a job to evict expired keys.
pub struct KeyEvictor {
    /// Stores key-expiry pairs in a BTreeSet, sorted in ascending order by expiry time.
    expiries: Arc<Mutex<BTreeSet<(OffsetDateTime, String)>>>,
    /// Reference to the database where keys are stored.
    db: Arc<DB>,
    /// Notifier for triggering key eviction jobs.
    eviction_notifier: Arc<Notify>,
}

impl KeyEvictor {
    /// Creates a new KeyEvictor instance.
    ///
    /// # Arguments
    ///
    /// * `db` - Arc reference to the database.
    /// * `eviction_notifier` - Arc reference to the Notify instance for eviction jobs.
    pub fn new(db: Arc<DB>, eviction_notifier: Arc<Notify>) -> KeyEvictor {
        KeyEvictor {
            expiries: Arc::new(Mutex::new(BTreeSet::new())),
            db,
            eviction_notifier,
        }
    }

    /// Runs background jobs for updating key expiry times and purging expired keys from the DB.
    pub async fn run(&mut self) {
        // Receiver for DB events
        let mut db_events_rx = self.db.subscribe_events();

        let expiries_arc = self.expiries.clone();
        let eviction_notifier_arc = self.eviction_notifier.clone();

        // Listen to DB events (like setting expiry to a key) and update the TTL of keys if required.
        tokio::spawn(async move {
            loop {
                let expiries = expiries_arc.clone();
                let eviction_notifier = eviction_notifier_arc.clone();

                match db_events_rx.recv().await {
                    Ok(evt) => match evt {
                        DBEvent::SetKeyExpiry(key_exp) => {
                            if let Err(e) = Self::update_key_expiry(key_exp, expiries) {
                                error!("Error while updating key expiry: {}", e);
                            }

                            eviction_notifier.notify_one();
                        }
                        DBEvent::BulkDelKeys(key_exps) => {
                            if let Err(e) = Self::remove_deleted_key(key_exps, expiries) {
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

        // Runs key eviction job.
        // This function sleeps until its time for a key to be expired.
        loop {
            let utc_now = OffsetDateTime::now_utc();
            let next_eviction_utc_ts = match self.evict_keys(utc_now) {
                Ok(when) => when,
                Err(e) => {
                    error!("Key eviction failed due to: {}", e);
                    None
                }
            };

            if let Some(next_trigger_utc_ts) = next_eviction_utc_ts {
                let next_trigger_ts_diff =
                    (next_trigger_utc_ts - utc_now).whole_milliseconds() as u64;
                tokio::select! {
                    // sleep until its time for a key to be expired
                    _ = tokio::time::sleep(Duration::from_millis(next_trigger_ts_diff)) => {}
                    // run the job if notified manually
                    _ = self.eviction_notifier.notified() => {}
                }
            } else {
                // If there's no key to be expired, wait until its notified manually
                self.eviction_notifier.notified().await;
            }
        }
    }

    /// Evicts keys that have expired up to the specified time.
    ///
    /// # Arguments
    ///
    /// * `expire_till` - OffsetDateTime specifying the cutoff time for expiration.
    ///
    /// # Returns
    ///
    /// A Result containing an Option<OffsetDateTime> representing the next expiration time,
    /// or None if there are no more keys to expire. Returns a DBError if the operation fails.
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

    /// Updates the expiry time for a key in the BTreeSet.
    ///
    /// # Arguments
    ///
    /// * `key_expiry` - A tuple containing the expiry time and the key.
    /// * `expiries` - Arc reference to the Mutex-protected BTreeSet of expiries.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DBError if the operation fails.
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

    /// Removes the key-expiry entry in the BTreeSet. This is called when a key is
    /// deleted from the DB manually, for which an expiry was already set before.
    ///
    /// # Arguments
    ///
    /// * `del_keys` - A tuple containing the list of deleted expiry-key pairs.
    /// * `expiries` - Arc reference to the Mutex-protected BTreeSet of expiries.
    ///
    /// # Returns
    ///
    /// A Result indicating success or a DBError if the operation fails.
    fn remove_deleted_key(
        del_keys: Vec<(OffsetDateTime, String)>,
        expiries: Arc<Mutex<BTreeSet<(OffsetDateTime, String)>>>,
    ) -> Result<(), DBError> {
        let mut expiries = match expiries.lock() {
            Ok(exp) => exp,
            Err(e) => {
                error!("Failed to remove deleted keys: {}", e);
                return Err(DBError::Other("Failed to remove deleted keys".to_string()));
            }
        };

        for k in del_keys {
            expiries.remove(&k);
        }

        Ok(())
    }
}
