# Frontend - Temporal-First Design

## Overview

The Frontend uses Temporal workflows for complex UI operations, making multi-step processes reliable and maintainable while providing a modern React-based interface.

```
┌─────────────────────────────────────────────────────────────┐
│              Temporal-First Frontend                       │
├─────────────────┬─────────────────┬─────────────────────────┤
│  UI Workflows   │   Components    │    Platform             │
│                 │                 │    Integration          │
│                 │                 │                         │
│ • File Upload   │ • React 18+     │ • Web (SPA)            │
│ • Data Sync     │ • TypeScript    │ • Desktop (Tauri)      │
│ • Form Wizard   │ • TailwindCSS   │ • Mobile (PWA)         │
│ • Bulk Actions  │ • State Mgmt    │ • Real-time Updates    │
└─────────────────┴─────────────────┴─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Temporal    │    │   React Query │    │   WebSocket   │
│   Workflows   │    │   (Cache)     │    │   (Real-time) │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Temporal-First Frontend Workflows

### 1. File Upload Workflow (Frontend-Initiated)
```typescript
// Frontend initiates Temporal workflow for complex file uploads
export const useFileUploadWorkflow = () => {
  const [uploadState, setUploadState] = useState<UploadState>('idle');
  const [progress, setProgress] = useState(0);
  const [workflowId, setWorkflowId] = useState<string | null>(null);

  const startUpload = async (files: File[], options: UploadOptions) => {
    try {
      setUploadState('starting');
      
      // Start Temporal workflow via API
      const response = await api.post('/workflows/file-upload', {
        files: files.map(f => ({
          name: f.name,
          size: f.size,
          type: f.type,
        })),
        options,
      });
      
      const { workflowId: id } = response.data;
      setWorkflowId(id);
      setUploadState('uploading');
      
      // Monitor workflow progress
      monitorWorkflowProgress(id);
      
      // Upload files in chunks
      await uploadFilesInChunks(files, id);
      
    } catch (error) {
      setUploadState('error');
      console.error('Upload failed:', error);
    }
  };

  const monitorWorkflowProgress = (id: string) => {
    // WebSocket connection to monitor Temporal workflow
    const ws = new WebSocket(`/ws/workflows/${id}/progress`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setProgress(data.progress);
      
      if (data.status === 'completed') {
        setUploadState('completed');
        ws.close();
      } else if (data.status === 'failed') {
        setUploadState('error');
        ws.close();
      }
    };
  };

  return { startUpload, uploadState, progress, workflowId };
};

// Backend Temporal workflow for file processing
#[workflow]
pub async fn frontend_file_upload_workflow(
    upload_request: FileUploadRequest,
) -> WorkflowResult<UploadResult> {
    // Step 1: Validate upload request
    validate_upload_request_activity(upload_request.clone()).await?;
    
    // Step 2: Create upload session
    let session = create_upload_session_activity(upload_request.clone()).await?;
    
    // Step 3: Process files as they arrive
    let mut processed_files = Vec::new();
    
    for file_info in upload_request.files {
        // Wait for file chunks to be uploaded
        let file_data = wait_for_file_upload_activity(
            session.id.clone(),
            file_info.name.clone(),
        ).await?;
        
        // Process file (virus scan, validation, etc.)
        let processed_file = process_uploaded_file_activity(
            file_data,
            upload_request.options.clone(),
        ).await?;
        
        processed_files.push(processed_file);
        
        // Send progress update to frontend
        send_progress_update_activity(
            session.id.clone(),
            processed_files.len() as f32 / upload_request.files.len() as f32,
        ).await?;
    }
    
    // Step 4: Finalize upload
    let result = finalize_upload_activity(session.id, processed_files).await?;
    
    Ok(result)
}
```

### 2. Data Synchronization Workflow
```typescript
// Frontend data sync using Temporal workflows
export const useDataSync = () => {
  const [syncState, setSyncState] = useState<SyncState>('idle');
  const [lastSync, setLastSync] = useState<Date | null>(null);

  const startSync = async (syncType: SyncType) => {
    try {
      setSyncState('syncing');
      
      // Start Temporal sync workflow
      const response = await api.post('/workflows/data-sync', {
        syncType,
        lastSync,
        clientState: getClientState(),
      });
      
      const { workflowId } = response.data;
      
      // Monitor sync progress
      const ws = new WebSocket(`/ws/workflows/${workflowId}/progress`);
      
      ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        if (data.type === 'sync_update') {
          // Apply incremental updates
          applyDataUpdates(data.updates);
        } else if (data.type === 'sync_completed') {
          setSyncState('completed');
          setLastSync(new Date());
          ws.close();
        } else if (data.type === 'sync_failed') {
          setSyncState('error');
          ws.close();
        }
      };
      
    } catch (error) {
      setSyncState('error');
      console.error('Sync failed:', error);
    }
  };

  return { startSync, syncState, lastSync };
};

// Backend sync workflow
#[workflow]
pub async fn data_sync_workflow(
    sync_request: DataSyncRequest,
) -> WorkflowResult<SyncResult> {
    // Step 1: Determine what needs syncing
    let sync_plan = create_sync_plan_activity(
        sync_request.client_state,
        sync_request.last_sync,
    ).await?;
    
    // Step 2: Sync data in batches
    let mut sync_results = Vec::new();
    
    for batch in sync_plan.batches {
        // Sync batch of data
        let batch_result = sync_data_batch_activity(
            batch.clone(),
            sync_request.user_id,
        ).await?;
        
        // Send incremental update to client
        send_sync_update_activity(
            sync_request.client_id.clone(),
            batch_result.updates.clone(),
        ).await?;
        
        sync_results.push(batch_result);
        
        // Small delay between batches to avoid overwhelming client
        temporal_sdk::sleep(Duration::from_millis(100)).await;
    }
    
    // Step 3: Finalize sync
    let final_result = finalize_sync_activity(sync_results).await?;
    
    Ok(final_result)
}
```

### 3. Multi-Step Form Wizard Workflow
```typescript
// Complex form wizard using Temporal workflows
export const useFormWizard = <T extends FormData>(
  steps: FormStep<T>[],
  onComplete: (data: T) => void
) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState<Partial<T>>({});
  const [workflowId, setWorkflowId] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const startWizard = async () => {
    try {
      // Start Temporal workflow for form processing
      const response = await api.post('/workflows/form-wizard', {
        formType: steps[0].type,
        totalSteps: steps.length,
      });
      
      setWorkflowId(response.data.workflowId);
    } catch (error) {
      console.error('Failed to start wizard:', error);
    }
  };

  const submitStep = async (stepData: Partial<T>) => {
    if (!workflowId) return;
    
    try {
      setIsProcessing(true);
      
      // Submit step data to Temporal workflow
      const response = await api.post(`/workflows/${workflowId}/step`, {
        stepIndex: currentStep,
        data: stepData,
      });
      
      const { nextStep, isComplete, validationErrors } = response.data;
      
      if (validationErrors?.length > 0) {
        // Handle validation errors
        return { success: false, errors: validationErrors };
      }
      
      // Update local state
      setFormData(prev => ({ ...prev, ...stepData }));
      
      if (isComplete) {
        // Workflow completed
        onComplete({ ...formData, ...stepData } as T);
      } else {
        // Move to next step
        setCurrentStep(nextStep);
      }
      
      return { success: true };
      
    } catch (error) {
      console.error('Step submission failed:', error);
      return { success: false, errors: ['Submission failed'] };
    } finally {
      setIsProcessing(false);
    }
  };

  return {
    currentStep,
    formData,
    submitStep,
    startWizard,
    isProcessing,
  };
};

// Backend form wizard workflow
#[workflow]
pub async fn form_wizard_workflow(
    wizard_request: FormWizardRequest,
) -> WorkflowResult<FormResult> {
    // Step 1: Initialize wizard state
    let mut wizard_state = initialize_wizard_activity(wizard_request.clone()).await?;
    
    // Step 2: Process each step as it comes in
    loop {
        // Wait for step submission
        let step_data = wait_for_step_submission_signal().await;
        
        // Validate step data
        let validation_result = validate_step_activity(
            step_data.step_index,
            step_data.data.clone(),
            wizard_state.clone(),
        ).await?;
        
        if !validation_result.is_valid {
            // Send validation errors back to frontend
            send_step_response_activity(StepResponse {
                success: false,
                validation_errors: validation_result.errors,
                next_step: None,
                is_complete: false,
            }).await?;
            continue;
        }
        
        // Update wizard state
        wizard_state = update_wizard_state_activity(
            wizard_state,
            step_data.step_index,
            step_data.data,
        ).await?;
        
        // Check if wizard is complete
        if wizard_state.current_step >= wizard_request.total_steps {
            // Process final form submission
            let final_result = process_final_form_activity(wizard_state.clone()).await?;
            
            send_step_response_activity(StepResponse {
                success: true,
                validation_errors: vec![],
                next_step: None,
                is_complete: true,
            }).await?;
            
            return Ok(final_result);
        } else {
            // Send success response with next step
            send_step_response_activity(StepResponse {
                success: true,
                validation_errors: vec![],
                next_step: Some(wizard_state.current_step),
                is_complete: false,
            }).await?;
        }
    }
}
```

### 4. Bulk Operations Workflow
```typescript
// Bulk operations using Temporal workflows
export const useBulkOperations = () => {
  const [operations, setOperations] = useState<BulkOperation[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);

  const startBulkOperation = async (
    operationType: BulkOperationType,
    items: any[],
    options: BulkOptions
  ) => {
    try {
      setIsProcessing(true);
      
      // Start Temporal bulk operation workflow
      const response = await api.post('/workflows/bulk-operation', {
        operationType,
        items,
        options,
      });
      
      const { workflowId } = response.data;
      
      // Monitor progress
      const ws = new WebSocket(`/ws/workflows/${workflowId}/progress`);
      
      ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        if (data.type === 'operation_progress') {
          setOperations(prev => 
            prev.map(op => 
              op.id === data.operationId 
                ? { ...op, status: data.status, progress: data.progress }
                : op
            )
          );
        } else if (data.type === 'operation_completed') {
          setIsProcessing(false);
          // Handle completion
        }
      };
      
    } catch (error) {
      setIsProcessing(false);
      console.error('Bulk operation failed:', error);
    }
  };

  return { startBulkOperation, operations, isProcessing };
};

// Backend bulk operation workflow
#[workflow]
pub async fn bulk_operation_workflow(
    bulk_request: BulkOperationRequest,
) -> WorkflowResult<BulkResult> {
    // Step 1: Validate bulk operation
    validate_bulk_operation_activity(bulk_request.clone()).await?;
    
    // Step 2: Process items in batches
    let batch_size = determine_batch_size_activity(
        bulk_request.operation_type.clone(),
        bulk_request.items.len(),
    ).await?;
    
    let mut results = Vec::new();
    let total_items = bulk_request.items.len();
    
    for (batch_index, batch) in bulk_request.items.chunks(batch_size).enumerate() {
        // Process batch
        let batch_result = process_batch_activity(
            bulk_request.operation_type.clone(),
            batch.to_vec(),
            bulk_request.options.clone(),
        ).await?;
        
        results.extend(batch_result.results);
        
        // Send progress update
        let progress = ((batch_index + 1) * batch_size).min(total_items) as f32 / total_items as f32;
        send_bulk_progress_activity(
            bulk_request.client_id.clone(),
            progress,
            batch_result.summary.clone(),
        ).await?;
        
        // Small delay between batches
        temporal_sdk::sleep(Duration::from_millis(500)).await;
    }
    
    // Step 3: Generate final report
    let final_report = generate_bulk_report_activity(results.clone()).await?;
    
    Ok(BulkResult {
        total_processed: results.len(),
        successful: results.iter().filter(|r| r.success).count(),
        failed: results.iter().filter(|r| !r.success).count(),
        results,
        report: final_report,
    })
}
```

## Frontend Architecture

### Component Structure
```typescript
// Temporal-aware React components
export const TemporalWorkflowProvider: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const [activeWorkflows, setActiveWorkflows] = useState<Map<string, WorkflowState>>(new Map());
  
  const startWorkflow = async (type: string, data: any) => {
    const response = await api.post(`/workflows/${type}`, data);
    const { workflowId } = response.data;
    
    setActiveWorkflows(prev => new Map(prev.set(workflowId, {
      id: workflowId,
      type,
      status: 'running',
      progress: 0,
    })));
    
    return workflowId;
  };
  
  const monitorWorkflow = (workflowId: string) => {
    const ws = new WebSocket(`/ws/workflows/${workflowId}/progress`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      
      setActiveWorkflows(prev => {
        const updated = new Map(prev);
        const workflow = updated.get(workflowId);
        if (workflow) {
          updated.set(workflowId, {
            ...workflow,
            status: data.status,
            progress: data.progress,
          });
        }
        return updated;
      });
    };
    
    return () => ws.close();
  };
  
  return (
    <WorkflowContext.Provider value={{
      activeWorkflows,
      startWorkflow,
      monitorWorkflow,
    }}>
      {children}
    </WorkflowContext.Provider>
  );
};

// Hook for using Temporal workflows in components
export const useTemporalWorkflow = (type: string) => {
  const { startWorkflow, monitorWorkflow } = useContext(WorkflowContext);
  
  const execute = async (data: any) => {
    const workflowId = await startWorkflow(type, data);
    const cleanup = monitorWorkflow(workflowId);
    
    return { workflowId, cleanup };
  };
  
  return { execute };
};
```

## Key Benefits of Temporal-First Frontend

### 1. Reliable Complex Operations
- **File uploads** with automatic retry and resume
- **Data synchronization** with conflict resolution
- **Multi-step forms** with state persistence
- **Bulk operations** with progress tracking

### 2. Better User Experience
- **Real-time progress** updates via WebSocket
- **Error recovery** with automatic retries
- **State persistence** across browser refreshes
- **Offline support** with sync when reconnected

### 3. Simplified Development
- **Workflow patterns** for complex UI operations
- **Built-in error handling** and retry logic
- **Visual debugging** using Temporal UI
- **Easy testing** of complex user flows

### 4. Scalable Architecture
- **Backend workflows** handle heavy processing
- **Frontend focuses** on UI and user interaction
- **Clear separation** between presentation and business logic
- **Easy maintenance** and feature additions

This Temporal-first approach makes the frontend **reliable, maintainable, and user-friendly** while handling complex operations through proven workflow patterns.