use std::{collections::HashSet, marker::PhantomData, path::PathBuf};

use futures::try_join;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, Select, Set, TransactionTrait,
};
use tokio::fs;
use uuid::Uuid;

use entity::{history_doc_type, history_event, history_event_doc_type, keyed_data, mdoc, mdoc_copy};
use nl_wallet_mdoc::{
    holder::MdocCopies,
    utils::serialization::{cbor_deserialize, cbor_serialize, CborError},
};
use wallet_common::keys::SecureEncryptionKey;

use super::{
    data::KeyedData,
    database::{Database, SqliteUrl},
    event_log::WalletEvent,
    key_file::{delete_key_file, get_or_create_key_file},
    sql_cipher_key::SqlCipherKey,
    Storage, StorageError, StorageResult, StorageState, StoredMdocCopy,
};

const DATABASE_NAME: &str = "wallet";
const KEY_FILE_SUFFIX: &str = "_db";
const DATABASE_FILE_EXT: &str = "db";

fn key_file_alias_for_name(database_name: &str) -> String {
    // Append suffix to database name to get key file alias
    format!("{}{}", database_name, KEY_FILE_SUFFIX)
}

/// This is the implementation of [`Storage`] as used by the [`crate::Wallet`]. Its responsibilities are:
///
/// * Managing the lifetime of one or more [`Database`] instances by combining its functionality with
///   encrypted key files. This also includes deleting the database and key file when the [`clear`]
///   method is called.
/// * Communicating the current state of the database through the [`state`] method.
/// * Executing queries on the database by accepting / returning data structures that are used by
///   [`crate::Wallet`].
#[derive(Debug)]
pub struct DatabaseStorage<K> {
    storage_path: PathBuf,
    database: Option<Database>,
    _key: PhantomData<K>,
}

impl<K> DatabaseStorage<K> {
    pub fn init(storage_path: PathBuf) -> Self {
        DatabaseStorage {
            storage_path,
            database: None,
            _key: PhantomData,
        }
    }
}

impl<K> DatabaseStorage<K>
where
    K: SecureEncryptionKey,
{
    // Helper method, should be called before accessing database.
    fn database(&self) -> StorageResult<&Database> {
        self.database.as_ref().ok_or(StorageError::NotOpened)
    }

    fn database_path_for_name(&self, name: &str) -> PathBuf {
        // Get path to database as "<storage_path>/<name>.db"
        self.storage_path.join(format!("{}.{}", name, DATABASE_FILE_EXT))
    }

    /// This helper method uses [`get_or_create_key_file`] and the utilities in [`platform_support`]
    /// to construct a [`SqliteUrl`] and a [`SqlCipherKey`], which in turn are used to create a [`Database`]
    /// instance.
    async fn open_encrypted_database(&self, name: &str) -> StorageResult<Database> {
        let key_file_alias = key_file_alias_for_name(name);
        let database_path = self.database_path_for_name(name);

        // Get database key of the correct length including a salt, stored in encrypted file.
        let key_bytes =
            get_or_create_key_file::<K>(&self.storage_path, &key_file_alias, SqlCipherKey::size_with_salt()).await?;
        let key = SqlCipherKey::try_from(key_bytes.as_slice())?;

        // Open database at the path, encrypted using the key
        let database = Database::open(SqliteUrl::File(database_path), key).await?;

        Ok(database)
    }

    async fn query_unique_mdocs<F>(&self, transform_select: F) -> StorageResult<Vec<StoredMdocCopy>>
    where
        F: FnOnce(Select<mdoc_copy::Entity>) -> Select<mdoc_copy::Entity>,
    {
        let database = self.database()?;

        // As this query only contains one `MIN()` aggregate function, the columns that
        // do not appear in the `GROUP BY` clause are taken from whichever `mdoc_copy`
        // row has the lowest disclosure count. This uses the "bare columns in aggregate
        // queries" feature that SQLite provides.
        //
        // See: https://www.sqlite.org/lang_select.html#bare_columns_in_an_aggregate_query
        let select = mdoc_copy::Entity::find()
            .select_only()
            .columns([
                mdoc_copy::Column::Id,
                mdoc_copy::Column::MdocId,
                mdoc_copy::Column::Mdoc,
            ])
            .column_as(mdoc_copy::Column::DisclosureCount.min(), "disclosure_count")
            .group_by(mdoc_copy::Column::MdocId);

        let mdoc_copies = transform_select(select).all(database.connection()).await?;

        let mdocs = mdoc_copies
            .into_iter()
            .map(|model| {
                let mdoc = cbor_deserialize(model.mdoc.as_slice())?;
                let stored_mdoc_copy = StoredMdocCopy {
                    mdoc_id: model.mdoc_id,
                    mdoc_copy_id: model.id,
                    mdoc,
                };

                Ok(stored_mdoc_copy)
            })
            .collect::<Result<_, CborError>>()?;

        Ok(mdocs)
    }
}

impl<K> Storage for DatabaseStorage<K>
where
    K: SecureEncryptionKey,
{
    /// Indicate whether there is no database on disk, there is one but it is unopened
    /// or the database is currently open.
    async fn state(&self) -> StorageResult<StorageState> {
        if self.database.is_some() {
            return Ok(StorageState::Opened);
        }

        let database_path = self.database_path_for_name(DATABASE_NAME);

        if fs::try_exists(database_path).await? {
            return Ok(StorageState::Unopened);
        }

        Ok(StorageState::Uninitialized)
    }

    /// Load a database, creating a new key file and database file if necessary.
    async fn open(&mut self) -> StorageResult<()> {
        if self.database.is_some() {
            return Err(StorageError::AlreadyOpened);
        }

        let database = self.open_encrypted_database(DATABASE_NAME).await?;
        self.database.replace(database);

        Ok(())
    }

    /// Clear the contents of the database by closing it and removing both database and key file.
    async fn clear(&mut self) -> StorageResult<()> {
        // Take the Database from the Option<> so that close_and_delete() can consume it.
        let database = self.database.take().ok_or(StorageError::NotOpened)?;
        let key_file_alias = key_file_alias_for_name(DATABASE_NAME);

        // Close and delete the database, only if this succeeds also delete the key file.
        database.close_and_delete().await?;
        delete_key_file(&self.storage_path, &key_file_alias).await;

        Ok(())
    }

    /// Get data entry from the key-value table, if present.
    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>> {
        let database = self.database()?;

        let data = keyed_data::Entity::find_by_id(D::KEY)
            .one(database.connection())
            .await?
            .map(|m| serde_json::from_value::<D>(m.data))
            .transpose()?;

        Ok(data)
    }

    /// Insert data entry in the key-value table, which will return an error when one is already present.
    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        let database = self.database()?;

        let _ = keyed_data::ActiveModel {
            key: Set(D::KEY.to_string()),
            data: Set(serde_json::to_value(data)?),
        }
        .insert(database.connection())
        .await?;

        Ok(())
    }

    /// Update data entry in the key-value table using the provided key.
    async fn update_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        let database = self.database()?;

        keyed_data::Entity::update_many()
            .col_expr(keyed_data::Column::Data, Expr::value(serde_json::to_value(data)?))
            .filter(keyed_data::Column::Key.eq(D::KEY.to_string()))
            .exec(database.connection())
            .await?;

        Ok(())
    }

    async fn insert_mdocs(&mut self, mdocs: Vec<MdocCopies>) -> StorageResult<()> {
        // Construct a vec of tuples of 1 `mdoc` and 1 or more `mdoc_copy` models,
        // based on the unique `MdocCopies`, to be inserted into the database.
        let mdoc_models = mdocs
            .into_iter()
            .filter(|mdoc_copies| !mdoc_copies.cred_copies.is_empty())
            .map(|mdoc_copies| {
                let mdoc_id = Uuid::new_v4();

                let copy_models = mdoc_copies
                    .cred_copies
                    .iter()
                    .map(|mdoc| {
                        let model = mdoc_copy::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            mdoc_id: Set(mdoc_id),
                            mdoc: Set(cbor_serialize(&mdoc)?),
                            ..Default::default()
                        };

                        Ok(model)
                    })
                    .collect::<Result<Vec<_>, CborError>>()?;

                // `mdoc_copies.cred_copies` is guaranteed to contain at least one value because of the filter() above.
                let doc_type = mdoc_copies.cred_copies.into_iter().next().unwrap().doc_type;
                let mdoc_model = mdoc::ActiveModel {
                    id: Set(mdoc_id),
                    doc_type: Set(doc_type),
                };

                Ok((mdoc_model, copy_models))
            })
            .collect::<Result<Vec<_>, CborError>>()?;

        // Make two separate vecs out of the vec of tuples.
        let (mdoc_models, copy_models): (Vec<_>, Vec<_>) = mdoc_models.into_iter().unzip();

        let transaction = self.database()?.connection().begin().await?;

        mdoc::Entity::insert_many(mdoc_models).exec(&transaction).await?;
        mdoc_copy::Entity::insert_many(copy_models.into_iter().flatten())
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    async fn increment_mdoc_copies_usage_count(&mut self, mdoc_copy_ids: Vec<Uuid>) -> StorageResult<()> {
        mdoc_copy::Entity::update_many()
            .col_expr(
                mdoc_copy::Column::DisclosureCount,
                Expr::col(mdoc_copy::Column::DisclosureCount).add(1),
            )
            .filter(mdoc_copy::Column::Id.is_in(mdoc_copy_ids))
            .exec(self.database()?.connection())
            .await?;

        Ok(())
    }

    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<StoredMdocCopy>> {
        self.query_unique_mdocs(|select| select).await
    }

    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<StoredMdocCopy>> {
        let doc_types_iter = doc_types.iter().copied();

        self.query_unique_mdocs(move |select| {
            select
                .inner_join(mdoc::Entity)
                .filter(mdoc::Column::DocType.is_in(doc_types_iter))
        })
        .await
    }

    async fn log_wallet_event(&mut self, event: WalletEvent) -> StorageResult<()> {
        let transaction = self.database()?.connection().begin().await?;

        let event_doc_types = event.associated_doc_types();

        // Find existing doc_type entities
        let existing_doc_type_entities = history_doc_type::Entity::find()
            .filter(history_doc_type::Column::DocType.is_in(event_doc_types.clone()))
            .all(&transaction)
            .await?;

        // Get Vec of existing doc_types
        let existing_doc_types = existing_doc_type_entities
            .iter()
            .map(|e| e.doc_type.as_str())
            .collect::<Vec<_>>();

        // Determine what new doc_type entries need to be inserted
        let new_doc_type_entities = event_doc_types
            .into_iter()
            .filter(|doc_type| !existing_doc_types.contains(doc_type))
            .map(|doc_type| history_doc_type::Model {
                id: Uuid::new_v4(),
                doc_type: doc_type.to_owned(),
            })
            .collect::<Vec<_>>();

        // Create the main history event
        let event_entity: history_event::ActiveModel = history_event::Model::try_from(event)?.into();

        // Prepare the event <-> doc_type mapping entities.
        // This is done before inserting the `event_entity`, in order to avoid cloning.
        let event_doc_type_entities = new_doc_type_entities
            .iter()
            .chain(existing_doc_type_entities.iter())
            .map(|doc_type| history_event_doc_type::ActiveModel {
                history_event_id: event_entity.id.clone(),
                history_doc_type_id: Set(doc_type.id),
            })
            .collect::<Vec<_>>();

        // Insert the event and the new doc_types simultaneously
        let insert_events = history_event::Entity::insert(event_entity).exec(&transaction);
        let insert_new_doc_types = async {
            if !new_doc_type_entities.is_empty() {
                let doc_type_entities = new_doc_type_entities
                    .into_iter()
                    .map(history_doc_type::ActiveModel::from)
                    .collect::<Vec<_>>();

                history_doc_type::Entity::insert_many(doc_type_entities)
                    .exec(&transaction)
                    .await?;
            }
            Ok(())
        };
        try_join!(insert_events, insert_new_doc_types)?;

        // Insert the event <-> doc_type mappings
        if !event_doc_type_entities.is_empty() {
            history_event_doc_type::Entity::insert_many(event_doc_type_entities)
                .exec(&transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let entities = history_event::Entity::find()
            .order_by_desc(history_event::Column::Timestamp)
            .all(connection)
            .await?;

        let events = entities
            .into_iter()
            .map(WalletEvent::try_from)
            .collect::<Result<_, _>>()?;
        Ok(events)
    }

    async fn fetch_wallet_events_by_doc_type(&self, doc_type: &str) -> StorageResult<Vec<WalletEvent>> {
        let connection = self.database()?.connection();

        let entities = history_event::Entity::find()
            .join_rev(
                JoinType::InnerJoin,
                history_event_doc_type::Relation::HistoryEvent.def(),
            )
            .join(
                JoinType::InnerJoin,
                history_event_doc_type::Relation::HistoryDocType.def(),
            )
            .filter(history_doc_type::Column::DocType.eq(doc_type))
            .order_by_desc(history_event::Column::Timestamp)
            .all(connection)
            .await?;

        let events = entities
            .into_iter()
            .map(WalletEvent::try_from)
            .collect::<Result<_, _>>()?;
        Ok(events)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::{TimeZone, Utc};
    use tokio::fs;

    use nl_wallet_mdoc::{examples::Examples, mock as mdoc_mock, utils::x509::Certificate};
    use platform_support::utils::{software::SoftwareUtilities, PlatformUtilities};
    use wallet_common::{
        account::messages::auth::WalletCertificate, keys::software::SoftwareEncryptionKey, utils::random_bytes,
    };

    use crate::storage::data::RegistrationData;

    use super::*;

    const PID_DOCTYPE: &str = "com.example.pid";
    const ADDRESS_DOCTYPE: &str = "com.example.address";

    #[test]
    fn test_key_file_alias_for_name() {
        assert_eq!(key_file_alias_for_name("test_database"), "test_database_db");
    }

    #[tokio::test]
    async fn test_database_open_encrypted_database() {
        let storage = DatabaseStorage::<SoftwareEncryptionKey>::init(SoftwareUtilities::storage_path().await.unwrap());

        let name = "test_open_encrypted_database";
        let key_file_alias = key_file_alias_for_name(name);
        let database_path = storage.database_path_for_name(name);

        // Make sure we start with a clean slate.
        delete_key_file(&storage.storage_path, &key_file_alias).await;
        _ = fs::remove_file(database_path).await;

        let database = storage
            .open_encrypted_database(name)
            .await
            .expect("Could not open encrypted database");

        assert!(matches!(&database.url, SqliteUrl::File(path)
            if path.to_str().unwrap().contains("test_open_encrypted_database.db")));

        database
            .close_and_delete()
            .await
            .expect("Could not close and delete database");
    }

    async fn open_test_database_storage() -> DatabaseStorage<SoftwareEncryptionKey> {
        let mut storage =
            DatabaseStorage::<SoftwareEncryptionKey>::init(SoftwareUtilities::storage_path().await.unwrap());

        // Create a test database, override the database field on Storage.
        let key_bytes = random_bytes(SqlCipherKey::size_with_salt());
        let database = Database::open(SqliteUrl::InMemory, key_bytes.as_slice().try_into().unwrap())
            .await
            .expect("Could not open in-memory database");
        storage.database = Some(database);

        storage
    }

    #[tokio::test]
    async fn test_database_storage() {
        let registration = RegistrationData {
            pin_salt: vec![1, 2, 3, 4].into(),
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid"),
        };

        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Try to fetch the registration, none should be there.
        let no_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(no_registration.is_none());

        // Save the registration and fetch it again.
        storage
            .insert_data(&registration)
            .await
            .expect("Could not save registration");

        let fetched_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");

        assert!(fetched_registration.is_some());
        let fetched_registration = fetched_registration.unwrap();
        assert_eq!(fetched_registration.pin_salt.0, registration.pin_salt.0);
        assert_eq!(
            fetched_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Save the registration again, should result in an error.
        let save_result = storage.insert_data(&registration).await;
        assert!(save_result.is_err());

        // Update registration
        let new_salt = random_bytes(64).into();
        let updated_registration = RegistrationData {
            pin_salt: new_salt,
            wallet_certificate: registration.wallet_certificate.clone(),
        };
        storage
            .update_data(&updated_registration)
            .await
            .expect("Could not update registration");

        let fetched_after_update_registration = storage
            .fetch_data::<RegistrationData>()
            .await
            .expect("Could not get registration");
        assert!(fetched_after_update_registration.is_some());
        let fetched_after_update_registration = fetched_after_update_registration.unwrap();
        assert_eq!(
            fetched_after_update_registration.pin_salt.0,
            updated_registration.pin_salt.0
        );
        assert_eq!(
            fetched_after_update_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Clear database, state should be uninitialized.
        storage.clear().await.expect("Could not clear storage");

        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Uninitialized));
    }

    #[tokio::test]
    async fn test_mdoc_storage() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        // Create MdocsMap from example Mdoc
        let trust_anchors = Examples::iaca_trust_anchors();
        let mdoc = mdoc_mock::mdoc_from_example_device_response(trust_anchors);
        let mdoc_copies = MdocCopies::from([mdoc.clone(), mdoc.clone(), mdoc].to_vec());

        // Insert mdocs
        storage
            .insert_mdocs(vec![mdoc_copies.clone()])
            .await
            .expect("Could not insert mdocs");

        // Fetch unique mdocs
        let fetched_unique = storage
            .fetch_unique_mdocs()
            .await
            .expect("Could not fetch unique mdocs");

        // Only one unique `Mdoc` should be returned and it should match all copies.
        assert_eq!(fetched_unique.len(), 1);
        let mdoc_copy1 = fetched_unique.first().unwrap();
        assert_eq!(&mdoc_copy1.mdoc, mdoc_copies.cred_copies.first().unwrap());

        // Increment the usage count for this mdoc.
        storage
            .increment_mdoc_copies_usage_count(vec![mdoc_copy1.mdoc_copy_id])
            .await
            .expect("Could not increment usage count for mdoc copy");

        // Fetch unique mdocs based on doctype
        let fetched_unique_doctype = storage
            .fetch_unique_mdocs_by_doctypes(&HashSet::from(["foo", "org.iso.18013.5.1.mDL"]))
            .await
            .expect("Could not fetch unique mdocs by doctypes");

        // One matching `Mdoc` should be returned and it should be a different copy than the fist one.
        assert_eq!(fetched_unique_doctype.len(), 1);
        let mdoc_copy2 = fetched_unique_doctype.first().unwrap();
        assert_eq!(&mdoc_copy2.mdoc, mdoc_copies.cred_copies.first().unwrap());
        assert_ne!(mdoc_copy1.mdoc_copy_id, mdoc_copy2.mdoc_copy_id);

        // Increment the usage count for this mdoc.
        storage
            .increment_mdoc_copies_usage_count(vec![mdoc_copy2.mdoc_copy_id])
            .await
            .expect("Could not increment usage count for mdoc copy");

        // Fetch unique mdocs twice, which should result in exactly the same
        // copy, since it is the last one that has a `usage_count` of 0.
        let fetched_unique_remaining1 = storage
            .fetch_unique_mdocs()
            .await
            .expect("Could not fetch unique mdocs");
        let fetched_unique_remaining2 = storage
            .fetch_unique_mdocs()
            .await
            .expect("Could not fetch unique mdocs");

        // Test that the copy identifiers are the same and that they
        // are different from the other two mdoc copy identifiers.
        assert_eq!(fetched_unique_remaining1.len(), 1);
        let remaning_mdoc_copy_id1 = fetched_unique_remaining1.first().unwrap().mdoc_copy_id;
        assert_eq!(fetched_unique_remaining2.len(), 1);
        let remaning_mdoc_copy_id2 = fetched_unique_remaining2.first().unwrap().mdoc_copy_id;

        assert_eq!(remaning_mdoc_copy_id1, remaning_mdoc_copy_id2);
        assert_ne!(mdoc_copy1.mdoc_copy_id, remaning_mdoc_copy_id1);
        assert_ne!(mdoc_copy2.mdoc_copy_id, remaning_mdoc_copy_id1);

        // Fetch unique mdocs based on non-existent doctype
        let fetched_unique_doctype_mismatch = storage
            .fetch_unique_mdocs_by_doctypes(&HashSet::from(["foo", "bar"]))
            .await
            .unwrap();

        // No entries should be returned
        assert!(fetched_unique_doctype_mismatch.is_empty());
    }

    #[tokio::test]
    async fn test_event_log_storage_ordering() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));
        test_history_ordering(&mut storage).await;
    }

    #[tokio::test]
    async fn test_event_log_storage_by_doc_type() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        test_history_by_doc_type(&mut storage).await;
    }

    #[tokio::test]
    async fn test_storing_disclosure_cancel_event() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let (certificate, _) = Certificate::new_ca("test-ca").unwrap();
        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let disclosure_cancel = WalletEvent::disclosure_cancel(timestamp, certificate.clone());
        storage.log_wallet_event(disclosure_cancel.clone()).await.unwrap();
        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![disclosure_cancel.clone(),]
        );
    }

    #[tokio::test]
    async fn test_storing_disclosure_error_event() {
        let mut storage = open_test_database_storage().await;

        // State should be Opened.
        let state = storage.state().await.unwrap();
        assert!(matches!(state, StorageState::Opened));

        let (certificate, _) = Certificate::new_ca("test-ca").unwrap();
        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let disclosure_error = WalletEvent::disclosure_error(timestamp, certificate.clone(), "Some ERROR".to_string());
        storage.log_wallet_event(disclosure_error.clone()).await.unwrap();
        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![disclosure_error.clone(),]
        );
    }

    pub(crate) async fn test_history_ordering(storage: &mut impl Storage) {
        let (certificate, _) = Certificate::new_ca("test-ca").unwrap();

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();
        let timestamp_older = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();
        let timestamp_even_older = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();

        let disclosure_at_timestamp =
            WalletEvent::disclosure_from_str(vec![PID_DOCTYPE], timestamp, certificate.clone());
        let issuance_at_older_timestamp =
            WalletEvent::issuance_from_str(vec![ADDRESS_DOCTYPE], timestamp_older, certificate.clone());
        let issuance_at_even_older_timestamp =
            WalletEvent::issuance_from_str(vec![PID_DOCTYPE], timestamp_even_older, certificate.clone());
        // Insert events, in non-standard order, from new to old
        storage.log_wallet_event(disclosure_at_timestamp.clone()).await.unwrap();
        storage
            .log_wallet_event(issuance_at_older_timestamp.clone())
            .await
            .unwrap();
        storage
            .log_wallet_event(issuance_at_even_older_timestamp.clone())
            .await
            .unwrap();

        // Fetch and verify events are sorted descending by timestamp
        assert_eq!(
            storage.fetch_wallet_events().await.unwrap(),
            vec![
                disclosure_at_timestamp.clone(),
                issuance_at_older_timestamp.clone(),
                issuance_at_even_older_timestamp.clone()
            ]
        );
        // Fetch event by pid and verify events are sorted descending by timestamp
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(PID_DOCTYPE).await.unwrap(),
            vec![
                disclosure_at_timestamp.clone(),
                issuance_at_even_older_timestamp.clone()
            ]
        );
        // Fetch event by address
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(ADDRESS_DOCTYPE).await.unwrap(),
            vec![issuance_at_older_timestamp]
        );
        // Fetching for unknown-doc-type returns empty Vec
        assert_eq!(
            storage
                .fetch_wallet_events_by_doc_type("unknown-doc-type")
                .await
                .unwrap(),
            vec![]
        );
    }

    pub(crate) async fn test_history_by_doc_type(storage: &mut impl Storage) {
        // Prepare test data
        let (certificate, _) = Certificate::new_ca("test-ca").unwrap();

        let timestamp = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();
        let timestamp_newer = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();
        let timestamp_newest = Utc.with_ymd_and_hms(2023, 11, 29, 10, 50, 45).unwrap();

        // Log Issuance of pid and address cards
        let issuance =
            WalletEvent::issuance_from_str(vec![PID_DOCTYPE, ADDRESS_DOCTYPE], timestamp, certificate.clone());
        storage.log_wallet_event(issuance.clone()).await.unwrap();

        // Log Disclosure of pid and address cards
        let disclosure_pid_and_address =
            WalletEvent::disclosure_from_str(vec![PID_DOCTYPE, ADDRESS_DOCTYPE], timestamp_newer, certificate.clone());
        storage
            .log_wallet_event(disclosure_pid_and_address.clone())
            .await
            .unwrap();

        // Log Disclosure of pid card only
        let disclosure_pid_only =
            WalletEvent::disclosure_from_str(vec![PID_DOCTYPE], timestamp_newest, certificate.clone());
        storage.log_wallet_event(disclosure_pid_only.clone()).await.unwrap();

        // Fetch event by pid and verify events contain issuance of pid, and both full disclosure transactions with pid
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(PID_DOCTYPE).await.unwrap(),
            vec![
                disclosure_pid_only.clone(),
                disclosure_pid_and_address.clone(),
                issuance.clone(),
            ]
        );
        // Fetch event by address and verify events contain issuance of address, and one full disclosure transactions
        // with address
        assert_eq!(
            storage.fetch_wallet_events_by_doc_type(ADDRESS_DOCTYPE).await.unwrap(),
            vec![disclosure_pid_and_address, issuance,]
        );
    }
}
