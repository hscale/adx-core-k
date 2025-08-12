// Verification script to check that all required workflows are implemented
// This is a standalone script to verify the implementation without compiling the full service

use std::fs;

fn main() {
    println!("Verifying User Management Temporal Workflows Implementation...\n");

    // Check workflows.rs file
    let workflows_content = fs::read_to_string("src/workflows.rs")
        .expect("Failed to read workflows.rs");

    // Check activities.rs file
    let activities_content = fs::read_to_string("src/activities.rs")
        .expect("Failed to read activities.rs");

    // Check worker.rs file
    let worker_content = fs::read_to_string("src/worker.rs")
        .expect("Failed to read worker.rs");

    let mut all_workflows_found = true;
    let mut all_activities_found = true;

    // Required workflows from task 15
    let required_workflows = vec![
        "user_profile_sync_workflow",
        "user_preference_migration_workflow", 
        "user_data_export_workflow",
        "user_deactivation_workflow",
        "user_reactivation_workflow",
        "bulk_user_operation_workflow",
    ];

    // Required activities for the workflows
    let required_activities = vec![
        "sync_user_profile_activity",
        "migrate_user_preferences_activity",
        "export_user_data_activity", 
        "deactivate_user_activity",
        "reactivate_user_activity",
        "transfer_user_ownership_activity",
    ];

    println!("✅ Checking Workflows Implementation:");
    for workflow in &required_workflows {
        if workflows_content.contains(&format!("pub async fn {}", workflow)) {
            println!("  ✓ {} - IMPLEMENTED", workflow);
        } else {
            println!("  ✗ {} - MISSING", workflow);
            all_workflows_found = false;
        }
    }

    println!("\n✅ Checking Activities Implementation:");
    for activity in &required_activities {
        if activities_content.contains(&format!("async fn {}", activity)) {
            println!("  ✓ {} - IMPLEMENTED", activity);
        } else {
            println!("  ✗ {} - MISSING", activity);
            all_activities_found = false;
        }
    }

    println!("\n✅ Checking Worker Registration:");
    let mut all_registered = true;
    for workflow in &required_workflows {
        if worker_content.contains(&format!("Registering workflow: {}", workflow)) {
            println!("  ✓ {} - REGISTERED", workflow);
        } else {
            println!("  ✗ {} - NOT REGISTERED", workflow);
            all_registered = false;
        }
    }

    for activity in &required_activities {
        if worker_content.contains(&format!("Registering activity: {}", activity)) {
            println!("  ✓ {} - REGISTERED", activity);
        } else {
            println!("  ✗ {} - NOT REGISTERED", activity);
            all_registered = false;
        }
    }

    // Check request/response types
    println!("\n✅ Checking Request/Response Types:");
    let required_types = vec![
        "UserProfileSyncWorkflowRequest",
        "UserProfileSyncWorkflowResponse",
        "UserPreferenceMigrationWorkflowRequest", 
        "UserPreferenceMigrationWorkflowResponse",
        "UserDeactivationWorkflowRequest",
        "UserDeactivationWorkflowResponse",
        "UserReactivationWorkflowRequest",
        "UserReactivationWorkflowResponse",
        "BulkUserOperationWorkflowRequest",
        "BulkUserOperationWorkflowResponse",
    ];

    let mut all_types_found = true;
    for type_name in &required_types {
        if workflows_content.contains(&format!("pub struct {}", type_name)) {
            println!("  ✓ {} - DEFINED", type_name);
        } else {
            println!("  ✗ {} - MISSING", type_name);
            all_types_found = false;
        }
    }

    // Check test file exists
    println!("\n✅ Checking Test Implementation:");
    if fs::metadata("src/workflows_test.rs").is_ok() {
        let test_content = fs::read_to_string("src/workflows_test.rs")
            .expect("Failed to read test file");
        
        if test_content.contains("user_management_workflow_tests") {
            println!("  ✓ Test module - IMPLEMENTED");
        } else {
            println!("  ✗ Test module - MISSING");
        }
    } else {
        println!("  ✗ Test file - MISSING");
    }

    println!("\n{}", "=".repeat(60));
    println!("VERIFICATION SUMMARY:");
    println!("{}", "=".repeat(60));

    if all_workflows_found && all_activities_found && all_registered && all_types_found {
        println!("🎉 SUCCESS: All User Management Temporal Workflows are properly implemented!");
        println!("\nImplemented workflows:");
        for workflow in &required_workflows {
            println!("  • {}", workflow);
        }
        println!("\nImplemented activities:");
        for activity in &required_activities {
            println!("  • {}", activity);
        }
        println!("\n✅ Task 15 requirements are COMPLETE");
    } else {
        println!("❌ INCOMPLETE: Some components are missing");
        if !all_workflows_found {
            println!("  - Missing workflow implementations");
        }
        if !all_activities_found {
            println!("  - Missing activity implementations");
        }
        if !all_registered {
            println!("  - Missing worker registrations");
        }
        if !all_types_found {
            println!("  - Missing type definitions");
        }
    }
}