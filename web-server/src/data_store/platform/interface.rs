use std::ops::DerefMut;

use sqlx::{types::Uuid, Acquire, Postgres, Transaction};

use crate::data_store::platform::wgpu_adapter_info::DataStoreWgpuBackend;
use crate::data_store::platform::wgpu_adapter_info::DataStoreWgpuDeviceType;
use crate::data_store::{non_empty_string::NonEmptyString, PostgresDataStore};

use super::Hardware;
use super::{
    user_agent_info::{
        DataStoreUserAgent, DataStoreUserAgentDevice, DataStoreUserAgentOs,
        DataStoreUserAgentStringInfo,
    },
    webgpu_adapter_info::DataStoreWebGpuAdapterInfo,
    wgpu_adapter_info::DataStoreWgpuAdapterInfo,
    DataStoreCreatePlatform, DataStorePlatform,
};

#[async_trait::async_trait]
pub trait DataStorePlatformInterface {
    async fn create_or_get_platform(
        &self,
        create: DataStoreCreatePlatform,
    ) -> Result<DataStorePlatform, sqlx::Error>;

    /// Returns a list of [Hardware]s that can later be used to retrieve the
    /// platforms that have been submitted that have those hardwares.
    ///
    /// The hardwares are decided by combining the webgpu adapter info `.vendor`
    /// and `.architecture` fields.
    ///
    /// TODO: In the future, when more data is available to build better
    /// heuristics, can also take into account the user agent device info
    /// perhaps and the wgpu info.
    async fn list_available_hardwares(
        &self,
    ) -> Result<Vec<Hardware>, sqlx::Error>;

    /// Returns a list of strings that can later be used to retrieve the
    /// platforms that have been submitted that have those operating systems.
    ///
    /// The operating systems are taken from the user agent OS info.
    ///
    /// In the future we might separate them by OS version if that were ever
    /// revelant.
    async fn list_available_operating_systems(
        &self,
    ) -> Result<Vec<String>, sqlx::Error>;
}

#[async_trait::async_trait]
impl DataStorePlatformInterface for PostgresDataStore {
    async fn create_or_get_platform(
        &self,
        create: DataStoreCreatePlatform,
    ) -> Result<DataStorePlatform, sqlx::Error> {
        let mut client = self.client().await?;

        let mut tx = client.begin().await?;

        let user_agent_id: Option<Uuid> =
            if let Some(create) = &create.user_agent {
                Some(create_user_agent_string_info(&mut tx, create).await?)
            } else {
                None
            };

        let wgpu_adapter_info_id: Uuid =
            create_wgpu_adapter_info(&mut tx, &create.wgpu_adapter_info)
                .await?;

        let webgpu_adapter_info_id: Option<Uuid> =
            if let Some(create) = &create.webgpu_adapter_info {
                Some(create_webgpu_adapter_info(&mut tx, create).await?)
            } else {
                None
            };

        let platform_id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO platform (user_agent_string_info_id, wgpu_adapter_info_id, webgpu_adapter_info_id)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING platform_id
            )
            SELECT platform_id FROM ins
            UNION
            SELECT platform_id FROM platform
            WHERE user_agent_string_info_id IS NOT DISTINCT FROM $1
              AND wgpu_adapter_info_id = $2
              AND webgpu_adapter_info_id IS NOT DISTINCT FROM $3
            "#,
            user_agent_id,
            wgpu_adapter_info_id,
            webgpu_adapter_info_id
        )
        .fetch_one(tx.deref_mut())
        .await?
        .expect("If insertion returns nothing then record must exist and SELECT will get it");

        tx.commit().await?;

        Ok(DataStorePlatform {
            platform_id,
            user_agent: create.user_agent,
            wgpu_adapter_info: create.wgpu_adapter_info,
            webgpu_adapter_info: create.webgpu_adapter_info,
        })
    }

    async fn list_available_hardwares(
        &self,
    ) -> Result<Vec<Hardware>, sqlx::Error> {
        let mut client = self.client().await?;

        sqlx::query_as!(
            Hardware,
            r#"
            SELECT DISTINCT
                vendor as "webgpu_vendor!",
                architecture as "webgpu_architecture!"
            FROM
                webgpu_adapter_info
            WHERE
                vendor IS NOT NULL AND architecture IS NOT NULL;
            "#,
        )
        .fetch_all(client.acquire().await?)
        .await
    }

    async fn list_available_operating_systems(
        &self,
    ) -> Result<Vec<String>, sqlx::Error> {
        let mut client = self.client().await?;

        sqlx::query_scalar!(
            r#"
            SELECT DISTINCT operating_system
            FROM user_agent_os;
            "#,
        )
        .fetch_all(client.acquire().await?)
        .await
    }
}

async fn create_user_agent_string_info(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgentStringInfo,
) -> Result<Uuid, sqlx::Error> {
    let user_agent_id: Option<Uuid> = if let Some(create) = &create.user_agent {
        Some(create_user_agent(tx, create).await?)
    } else {
        None
    };

    let user_agent_device_id: Option<Uuid> =
        if let Some(create) = &create.device {
            Some(create_user_agent_device(tx, create).await?)
        } else {
            None
        };

    let user_agent_os_id: Option<Uuid> =
        if let Some(create) = &create.operating_system {
            Some(create_user_agent_os(tx, create).await?)
        } else {
            None
        };

    let id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO user_agent_string_info (user_agent_id, user_agent_device_id, user_agent_os_id)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_string_info_id
            )
            SELECT user_agent_string_info_id FROM ins
            UNION
            SELECT user_agent_string_info_id FROM user_agent_string_info
            WHERE user_agent_id IS NOT DISTINCT FROM $1
              AND user_agent_device_id IS NOT DISTINCT FROM $2
              AND user_agent_os_id IS NOT DISTINCT FROM $3
            "#,
            user_agent_id,
            user_agent_device_id,
            user_agent_os_id,
        )
        .fetch_one(tx.deref_mut())
        .await?
        .expect("If insertion returns nothing then record must exist and SELECT will get it");

    Ok(id)
}

async fn create_user_agent_os(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgentOs,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO user_agent_os (operating_system, major, minor, patch, patch_minor)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_os_id
            )
            SELECT user_agent_os_id FROM ins
            UNION
            SELECT user_agent_os_id FROM user_agent_os
            WHERE operating_system = $1
              AND major IS NOT DISTINCT FROM $2
              AND minor IS NOT DISTINCT FROM $3
              AND patch IS NOT DISTINCT FROM $4
              AND patch_minor IS NOT DISTINCT FROM $5
            "#,
            &create.operating_system as &NonEmptyString,
            &create.major as &Option<NonEmptyString>,
            &create.minor as &Option<NonEmptyString>,
            &create.patch as &Option<NonEmptyString>,
            &create.patch_minor as &Option<NonEmptyString>,
        )
        .fetch_one(tx.deref_mut())
        .await?
        .expect("If insertion returns nothing then record must exist and SELECT will get it");

    Ok(id)
}

async fn create_user_agent_device(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgentDevice,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO user_agent_device (device, brand, model)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_device_id
            )
            SELECT user_agent_device_id FROM ins
            UNION
            SELECT user_agent_device_id FROM user_agent_device
            WHERE device = $1
              AND brand IS NOT DISTINCT FROM $2
              AND model IS NOT DISTINCT FROM $3
            "#,
        &create.device as &NonEmptyString,
        &create.brand as &Option<NonEmptyString>,
        &create.model as &Option<NonEmptyString>,
    )
    .fetch_one(tx.deref_mut())
    .await?
    .expect("If insertion returns nothing then record must exist and SELECT will get it");

    Ok(id)
}

async fn create_user_agent(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgent,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO user_agent (family, major, minor, patch, patch_minor)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_id
            )
            SELECT user_agent_id FROM ins
            UNION
            SELECT user_agent_id FROM user_agent
            WHERE family = $1
              AND major IS NOT DISTINCT FROM $2
              AND minor IS NOT DISTINCT FROM $3
              AND patch IS NOT DISTINCT FROM $4
              AND patch_minor IS NOT DISTINCT FROM $5
            "#,
            &create.family as &NonEmptyString,
            &create.major as &Option<NonEmptyString>,
            &create.minor as &Option<NonEmptyString>,
            &create.patch as &Option<NonEmptyString>,
            &create.patch_minor as &Option<NonEmptyString>,
        )
        .fetch_one(tx.deref_mut())
        .await?
        .expect("If insertion returns nothing then record must exist and SELECT will get it");

    Ok(id)
}

async fn create_wgpu_adapter_info(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreWgpuAdapterInfo,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO wgpu_adapter_info (name, vendor, device, device_type, driver, driver_info, backend)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT DO NOTHING
                RETURNING wgpu_adapter_info_id
            )
            SELECT wgpu_adapter_info_id FROM ins
            UNION
            SELECT wgpu_adapter_info_id FROM wgpu_adapter_info
            WHERE name IS NOT DISTINCT FROM $1
              AND vendor = $2
              AND device = $3
              AND device_type = $4
              AND driver IS NOT DISTINCT FROM $5
              AND driver_info IS NOT DISTINCT FROM $6
              AND backend = $7
            "#,
            &create.name as &Option<NonEmptyString>,
            create.vendor as i32,
            create.device as i32,
            &create.device_type as &DataStoreWgpuDeviceType,
            &create.driver as &Option<NonEmptyString>,
            &create.driver_info as &Option<NonEmptyString>,
            &create.backend as &DataStoreWgpuBackend,
        )
        .fetch_one(tx.deref_mut())
        .await?
        .expect("If insertion returns nothing then record must exist and SELECT will get it");

    Ok(id)
}

async fn create_webgpu_adapter_info(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreWebGpuAdapterInfo,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO webgpu_adapter_info (architecture, description, device, vendor)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING
                RETURNING webgpu_adapter_info_id
            )
            SELECT webgpu_adapter_info_id FROM ins
            UNION
            SELECT webgpu_adapter_info_id FROM webgpu_adapter_info
            WHERE architecture IS NOT DISTINCT FROM $1
              AND description IS NOT DISTINCT FROM $2
              AND device IS NOT DISTINCT FROM $3
              AND vendor IS NOT DISTINCT FROM $4
            "#,
            &create.architecture as &Option<NonEmptyString>,
            &create.description as &Option<NonEmptyString>,
            &create.device as &Option<NonEmptyString>,
            &create.vendor as &Option<NonEmptyString>,
        )
        .fetch_one(tx.deref_mut())
        .await?
        .expect("If insertion returns nothing then record must exist and SELECT will get it");

    Ok(id)
}
