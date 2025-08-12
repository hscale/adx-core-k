#[cfg(test)]
mod user_management_workflow_tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use std::collections::HashMap;
    use adx_shared::temporal::{WorkflowContext, WorkflowError};

    // Helper function to create test workflow context
    fn create_test_context() -> WorkflowContext {
        use adx_shared::temporal::{
            WorkflowVersion, UserContext, TenantContext, WorkflowMetadata, 
            SubscriptionTier, TenantQuotas, TenantSettings, TenantIsolationLevel,
            WorkflowPriority
        };
        use std::time::Duration;
        
        WorkflowContext {
            workflow_id: "test-workflow-id".to_string(),
            run_id: "test-run-id".to_string(),
            workflow_type: "test-workflow".to_string(),
            version: WorkflowVersion::new(1, 0, 0),
            task_queue: "test-queue".to_string(),
            namespace: "test-namespace".to_string(),
            user_context: UserContext {
                user_id: "test-user-id".to_string(),
                email: "test@example.com".to_string(),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                session_id: Some("test-session-id".to_string()),
                device_info: None,
            },
            tenant_context: TenantContext {
                tenant_id: "test-tenant-id".to_string(),
                tenant_name: "Test Tenant".to_string(),
                subscription_tier: SubscriptionTier::Professional,
                features: vec!["basic_features".to_string()],
                quotas: TenantQuotas {
                    max_users: 100,
                    max_storage_gb: 1000,
                    max_api_calls_per_hour: 10000,
                    max_concurrent_workflows: 50,
                    max_file_upload_size_mb: 100,
                },
                settings: TenantSettings {
                    default_language: "en".to_string(),
                    timezone: "UTC".to_string(),
                    date_format: "YYYY-MM-DD".to_string(),
                    currency: "USD".to_string(),
                    branding: None,
                },
                isolation_level: TenantIsolationLevel::Schema,
            },
            metadata: WorkflowMetadata {
                start_time: Utc::now(),
                timeout: Duration::from_secs(3600),
                retry_policy: None,
                parent_workflow_id: None,
                correlation_id: None,
                business_process: None,
                priority: WorkflowPriority::Normal,
                tags: vec![],
            },
            search_attributes: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_user_profile_sync_workflow_success() {
        let context = create_test_context();
        let request = UserProfileSyncWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            sync_targets: vec!["auth".to_string(), "file".to_string(), "tenant".to_string()],
            sync_type: "full".to_string(),
            force_sync: false,
        };

        let result = user_profile_sync_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.synced_services.len(), 3);
        assert!(response.synced_services.contains(&"auth".to_string()));
        assert!(response.synced_services.contains(&"file".to_string()));
        assert!(response.synced_services.contains(&"tenant".to_string()));
        assert!(response.failed_services.is_empty());
        assert_eq!(response.sync_summary.len(), 3);
    }

    #[tokio::test]
    async fn test_user_profile_sync_workflow_with_unknown_service() {
        let context = create_test_context();
        let request = UserProfileSyncWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            sync_targets: vec!["auth".to_string(), "unknown_service".to_string()],
            sync_type: "incremental".to_string(),
            force_sync: true,
        };

        let result = user_profile_sync_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.synced_services.len(), 1);
        assert!(response.synced_services.contains(&"auth".to_string()));
        assert_eq!(response.failed_services.len(), 1);
        assert!(response.failed_services.contains(&"unknown_service".to_string()));
    }

    #[tokio::test]
    async fn test_user_preference_migration_workflow_success() {
        let context = create_test_context();
        let request = UserPreferenceMigrationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            migration_type: "upgrade".to_string(),
            source_version: Some("1.0".to_string()),
            target_version: Some("2.0".to_string()),
            preference_categories: vec![
                "ui_preferences".to_string(),
                "notification_settings".to_string(),
                "privacy_settings".to_string(),
            ],
            backup_preferences: true,
        };

        let result = user_preference_migration_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.migrated_categories.len(), 3);
        assert!(response.migrated_categories.contains(&"ui_preferences".to_string()));
        assert!(response.migrated_categories.contains(&"notification_settings".to_string()));
        assert!(response.migrated_categories.contains(&"privacy_settings".to_string()));
        assert!(response.failed_categories.is_empty());
        assert!(response.backup_id.is_some());
        assert!(response.rollback_available);
    }

    #[tokio::test]
    async fn test_user_preference_migration_workflow_with_unknown_category() {
        let context = create_test_context();
        let request = UserPreferenceMigrationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            migration_type: "cross_tenant".to_string(),
            source_version: None,
            target_version: None,
            preference_categories: vec![
                "ui_preferences".to_string(),
                "unknown_category".to_string(),
            ],
            backup_preferences: false,
        };

        let result = user_preference_migration_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.migrated_categories.len(), 1);
        assert!(response.migrated_categories.contains(&"ui_preferences".to_string()));
        assert_eq!(response.failed_categories.len(), 1);
        assert!(response.failed_categories.contains(&"unknown_category".to_string()));
        assert!(response.backup_id.is_none());
        assert!(!response.rollback_available);
    }

    #[tokio::test]
    async fn test_user_deactivation_workflow_success() {
        let context = create_test_context();
        let deactivated_by = Uuid::new_v4();
        let new_owner_id = Uuid::new_v4();
        
        let request = UserDeactivationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            deactivation_reason: "User requested account deletion".to_string(),
            deactivated_by,
            retain_data: true,
            data_retention_days: Some(30),
            notify_user: true,
            transfer_ownership: Some(TransferOwnershipRequest {
                new_owner_id,
                resource_types: vec!["files".to_string(), "projects".to_string()],
                notify_new_owner: true,
            }),
        };

        let result = user_deactivation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.data_retention_until.is_some());
        assert_eq!(response.ownership_transfers.len(), 2);
        assert_eq!(response.cleanup_tasks.len(), 4);
        
        // Check ownership transfers
        let file_transfer = response.ownership_transfers.iter()
            .find(|t| t.resource_type == "files")
            .unwrap();
        assert_eq!(file_transfer.new_owner_id, new_owner_id);
        assert_eq!(file_transfer.transferred_count, 10);
        assert_eq!(file_transfer.failed_count, 0);
    }

    #[tokio::test]
    async fn test_user_deactivation_workflow_without_data_retention() {
        let context = create_test_context();
        let request = UserDeactivationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            deactivation_reason: "Policy violation".to_string(),
            deactivated_by: Uuid::new_v4(),
            retain_data: false,
            data_retention_days: None,
            notify_user: false,
            transfer_ownership: None,
        };

        let result = user_deactivation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.data_retention_until.is_none());
        assert!(response.ownership_transfers.is_empty());
        assert_eq!(response.cleanup_tasks.len(), 4);
    }

    #[tokio::test]
    async fn test_user_reactivation_workflow_success() {
        let context = create_test_context();
        let request = UserReactivationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            reactivated_by: Uuid::new_v4(),
            restore_data: true,
            restore_permissions: true,
            send_welcome_back: true,
            reset_password: true,
        };

        let result = user_reactivation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.data_restored);
        assert!(response.permissions_restored);
        assert!(response.temporary_password.is_some());
        assert_eq!(response.restoration_summary.len(), 2);
        
        // Check restoration summary
        assert!(response.restoration_summary.contains_key("data_restored"));
        assert!(response.restoration_summary.contains_key("permissions_restored"));
    }

    #[tokio::test]
    async fn test_user_reactivation_workflow_minimal() {
        let context = create_test_context();
        let request = UserReactivationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            reactivated_by: Uuid::new_v4(),
            restore_data: false,
            restore_permissions: false,
            send_welcome_back: false,
            reset_password: false,
        };

        let result = user_reactivation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.data_restored);
        assert!(!response.permissions_restored);
        assert!(response.temporary_password.is_none());
        assert!(response.restoration_summary.is_empty());
    }

    #[tokio::test]
    async fn test_bulk_user_operation_workflow_create_users() {
        let context = create_test_context();
        let operations = vec![
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: None,
                operation_data: serde_json::json!({
                    "email": "user1@example.com",
                    "first_name": "User",
                    "last_name": "One"
                }),
                priority: Some(1),
            },
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: None,
                operation_data: serde_json::json!({
                    "email": "user2@example.com",
                    "first_name": "User",
                    "last_name": "Two"
                }),
                priority: Some(2),
            },
        ];

        let request = BulkUserOperationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            operation_type: "create".to_string(),
            user_operations: operations,
            batch_size: Some(5),
            continue_on_error: true,
            notify_on_completion: true,
            initiated_by: Uuid::new_v4(),
        };

        let result = bulk_user_operation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_operations, 2);
        assert_eq!(response.successful_operations, 2);
        assert_eq!(response.failed_operations, 0);
        assert_eq!(response.operation_results.len(), 2);
        
        // Check that all operations succeeded
        for result in &response.operation_results {
            assert_eq!(result.status, "success");
            assert!(result.user_id.is_some());
            assert!(result.error_message.is_none());
        }
    }

    #[tokio::test]
    async fn test_bulk_user_operation_workflow_update_users() {
        let context = create_test_context();
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();
        
        let operations = vec![
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: Some(user1_id),
                operation_data: serde_json::json!({
                    "first_name": "Updated User",
                    "last_name": "One"
                }),
                priority: None,
            },
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: Some(user2_id),
                operation_data: serde_json::json!({
                    "status": "inactive"
                }),
                priority: None,
            },
        ];

        let request = BulkUserOperationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            operation_type: "update".to_string(),
            user_operations: operations,
            batch_size: None,
            continue_on_error: false,
            notify_on_completion: false,
            initiated_by: Uuid::new_v4(),
        };

        let result = bulk_user_operation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_operations, 2);
        assert_eq!(response.successful_operations, 2);
        assert_eq!(response.failed_operations, 0);
        
        // Check that user IDs are preserved
        for result in &response.operation_results {
            assert_eq!(result.status, "success");
            assert!(result.user_id.is_some());
            assert!(result.user_id == Some(user1_id) || result.user_id == Some(user2_id));
        }
    }

    #[tokio::test]
    async fn test_bulk_user_operation_workflow_unknown_operation() {
        let context = create_test_context();
        let operations = vec![
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: Some(Uuid::new_v4()),
                operation_data: serde_json::json!({}),
                priority: None,
            },
        ];

        let request = BulkUserOperationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            operation_type: "unknown_operation".to_string(),
            user_operations: operations,
            batch_size: Some(1),
            continue_on_error: true,
            notify_on_completion: false,
            initiated_by: Uuid::new_v4(),
        };

        let result = bulk_user_operation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_operations, 1);
        assert_eq!(response.successful_operations, 0);
        assert_eq!(response.failed_operations, 1);
        assert_eq!(response.operation_results.len(), 1);
        
        let failed_result = &response.operation_results[0];
        assert_eq!(failed_result.status, "failed");
        assert!(failed_result.error_message.is_some());
        assert!(failed_result.error_message.as_ref().unwrap().contains("Unknown operation type"));
    }

    #[tokio::test]
    async fn test_bulk_user_operation_workflow_deactivate_users() {
        let context = create_test_context();
        let operations = vec![
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: Some(Uuid::new_v4()),
                operation_data: serde_json::json!({
                    "reason": "Bulk deactivation for compliance"
                }),
                priority: Some(1),
            },
            BulkUserOperation {
                operation_id: Uuid::new_v4(),
                user_id: Some(Uuid::new_v4()),
                operation_data: serde_json::json!({
                    "reason": "User requested deactivation"
                }),
                priority: Some(2),
            },
        ];

        let request = BulkUserOperationWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            operation_type: "deactivate".to_string(),
            user_operations: operations,
            batch_size: Some(10),
            continue_on_error: true,
            notify_on_completion: true,
            initiated_by: Uuid::new_v4(),
        };

        let result = bulk_user_operation_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_operations, 2);
        assert_eq!(response.successful_operations, 2);
        assert_eq!(response.failed_operations, 0);
        
        // Check completion summary
        assert!(response.completion_summary.contains_key("operation_type"));
        assert_eq!(
            response.completion_summary.get("operation_type").unwrap(),
            &serde_json::json!("deactivate")
        );
    }

    #[tokio::test]
    async fn test_user_data_export_workflow_email_delivery() {
        let context = create_test_context();
        let request = UserDataExportWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            export_format: "json".to_string(),
            include_activity_log: true,
            include_preferences: true,
            delivery_method: "email".to_string(),
            delivery_target: "user@example.com".to_string(),
        };

        let result = user_data_export_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.delivery_status, "email_sent");
        assert!(response.download_url.is_none());
        assert!(response.export_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_user_data_export_workflow_download_delivery() {
        let context = create_test_context();
        let request = UserDataExportWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            export_format: "csv".to_string(),
            include_activity_log: false,
            include_preferences: true,
            delivery_method: "download".to_string(),
            delivery_target: "".to_string(),
        };

        let result = user_data_export_workflow(context, request.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.delivery_status, "download_ready");
        assert!(response.download_url.is_some());
        assert!(response.download_url.as_ref().unwrap().contains(&response.export_id.to_string()));
    }

    #[tokio::test]
    async fn test_user_data_export_workflow_invalid_delivery_method() {
        let context = create_test_context();
        let request = UserDataExportWorkflowRequest {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            export_format: "xml".to_string(),
            include_activity_log: true,
            include_preferences: false,
            delivery_method: "invalid_method".to_string(),
            delivery_target: "target".to_string(),
        };

        let result = user_data_export_workflow(context, request.clone()).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowError::ValidationFailed { errors } => {
                assert_eq!(errors.len(), 1);
                assert!(errors[0].contains("Unknown delivery method"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }
}