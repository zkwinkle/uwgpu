use std::ops::DerefMut;

use sqlx::{types::Uuid, Acquire, Postgres, Transaction};

use crate::data_store::platform::wgpu_adapter_info::DataStoreWgpuBackend;
use crate::data_store::platform::wgpu_adapter_info::DataStoreWgpuDeviceType;
use crate::data_store::{non_empty_string::NonEmptyString, PostgresDataStore};

use super::{
    user_agent_info::{
        DataStoreUserAgent, DataStoreUserAgentDevice, DataStoreUserAgentOs,
        DataStoreUserAgentStringInfo,
    },
    webgpu_adapter_info::DataStoreWebGpuAdapterInfo,
    wgpu_adapter_info::DataStoreWgpuAdapterInfo,
    DataStoreCreatePlatform, DataStorePlatform,
};

pub trait DataStorePlatformInterface {
    async fn create_or_get_platform(
        &self,
        create: DataStoreCreatePlatform,
    ) -> Result<DataStorePlatform, sqlx::Error>;
}

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
                INSERT INTO platform (user_agent_string_info_id, wgpu_adapter_info_id, webgpu_adapter_info_id)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING platform_id
            "#,
            user_agent_id,
            wgpu_adapter_info_id,
            webgpu_adapter_info_id
        )
        .fetch_one(tx.deref_mut())
        .await?;

        Ok(DataStorePlatform {
            platform_id,
            user_agent: create.user_agent,
            wgpu_adapter_info: create.wgpu_adapter_info,
            webgpu_adapter_info: create.webgpu_adapter_info,
        })
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
                INSERT INTO user_agent_string_info (user_agent_id, user_agent_device_id, user_agent_os_id)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_string_info_id
            "#,
            user_agent_id,
            user_agent_device_id,
            user_agent_os_id,
        )
        .fetch_one(tx.deref_mut())
        .await?;

    Ok(id)
}

async fn create_user_agent_os(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgentOs,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
                INSERT INTO user_agent_os (operating_system, major, minor, patch, patch_minor)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_os_id
            "#,
            &create.operating_system as &NonEmptyString,
            &create.major as &Option<NonEmptyString>,
            &create.minor as &Option<NonEmptyString>,
            &create.patch as &Option<NonEmptyString>,
            &create.patch_minor as &Option<NonEmptyString>,
        )
        .fetch_one(tx.deref_mut())
        .await?;

    Ok(id)
}

async fn create_user_agent_device(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgentDevice,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
        r#"
                INSERT INTO user_agent_device (device, brand, model)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_device_id
            "#,
        &create.device as &NonEmptyString,
        &create.brand as &Option<NonEmptyString>,
        &create.model as &Option<NonEmptyString>,
    )
    .fetch_one(tx.deref_mut())
    .await?;

    Ok(id)
}

async fn create_user_agent(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreUserAgent,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
                INSERT INTO user_agent (family, major, minor, patch, patch_minor)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT DO NOTHING
                RETURNING user_agent_id
            "#,
            &create.family as &NonEmptyString,
            &create.major as &Option<NonEmptyString>,
            &create.minor as &Option<NonEmptyString>,
            &create.patch as &Option<NonEmptyString>,
            &create.patch_minor as &Option<NonEmptyString>,
        )
        .fetch_one(tx.deref_mut())
        .await?;

    Ok(id)
}

async fn create_wgpu_adapter_info(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreWgpuAdapterInfo,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
                INSERT INTO wgpu_adapter_info (name, vendor, device, device_type, driver, driver_info, backend)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT DO NOTHING
                RETURNING wgpu_adapter_info_id
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
        .await?;

    Ok(id)
}

async fn create_webgpu_adapter_info(
    tx: &mut Transaction<'_, Postgres>,
    create: &DataStoreWebGpuAdapterInfo,
) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar!(
            r#"
                INSERT INTO webgpu_adapter_info (architecture, description, device, vendor)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING
                RETURNING webgpu_adapter_info_id
            "#,
            &create.architecture as &Option<NonEmptyString>,
            &create.description as &Option<NonEmptyString>,
            &create.device as &Option<NonEmptyString>,
            &create.vendor as &Option<NonEmptyString>,
        )
        .fetch_one(tx.deref_mut())
        .await?;

    Ok(id)
}
