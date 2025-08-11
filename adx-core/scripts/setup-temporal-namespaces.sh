#!/bin/bash

# ADX Core Temporal Namespace Setup Script
# This script creates and configures Temporal namespaces for different environments

set -e

TEMPORAL_ADDRESS=${TEMPORAL_ADDRESS:-"localhost:7233"}
TEMPORAL_CLI=${TEMPORAL_CLI:-"tctl"}

echo "Setting up ADX Core Temporal namespaces..."
echo "Temporal Address: $TEMPORAL_ADDRESS"

# Function to create namespace with retry logic
create_namespace() {
    local namespace=$1
    local description=$2
    local retention=$3
    local max_retries=5
    local retry_count=0

    echo "Creating namespace: $namespace"
    
    while [ $retry_count -lt $max_retries ]; do
        if $TEMPORAL_CLI --address $TEMPORAL_ADDRESS namespace register \
            --namespace $namespace \
            --description "$description" \
            --retention $retention \
            --history_archival_state disabled \
            --visibility_archival_state disabled; then
            echo "✓ Successfully created namespace: $namespace"
            return 0
        else
            retry_count=$((retry_count + 1))
            echo "⚠ Failed to create namespace $namespace (attempt $retry_count/$max_retries)"
            if [ $retry_count -lt $max_retries ]; then
                echo "Retrying in 5 seconds..."
                sleep 5
            fi
        fi
    done
    
    echo "✗ Failed to create namespace $namespace after $max_retries attempts"
    return 1
}

# Function to configure search attributes
configure_search_attributes() {
    local namespace=$1
    
    echo "Configuring search attributes for namespace: $namespace"
    
    # Add ADX Core specific search attributes
    $TEMPORAL_CLI --address $TEMPORAL_ADDRESS admin cluster add-search-attributes \
        --namespace $namespace \
        --name TenantId --type Keyword \
        --name UserId --type Keyword \
        --name WorkflowType --type Keyword \
        --name BusinessProcess --type Keyword \
        --name Priority --type Int \
        --name Environment --type Keyword \
        --name Version --type Keyword \
        --name ModuleId --type Keyword \
        --name CorrelationId --type Keyword \
        --name ParentWorkflowId --type Keyword || echo "Search attributes may already exist"
    
    echo "✓ Search attributes configured for namespace: $namespace"
}

# Wait for Temporal server to be ready
echo "Waiting for Temporal server to be ready..."
max_wait=60
wait_count=0

while [ $wait_count -lt $max_wait ]; do
    if $TEMPORAL_CLI --address $TEMPORAL_ADDRESS cluster health; then
        echo "✓ Temporal server is ready"
        break
    else
        wait_count=$((wait_count + 1))
        echo "Waiting for Temporal server... ($wait_count/$max_wait)"
        sleep 2
    fi
done

if [ $wait_count -eq $max_wait ]; then
    echo "✗ Temporal server is not ready after $max_wait attempts"
    exit 1
fi

# Create development namespace
create_namespace "adx-core-development" \
    "ADX Core Development Environment - For local development and testing" \
    "72h"

# Create staging namespace
create_namespace "adx-core-staging" \
    "ADX Core Staging Environment - For integration testing and pre-production validation" \
    "168h"

# Create production namespace
create_namespace "adx-core-production" \
    "ADX Core Production Environment - For live production workloads" \
    "8760h"

# Configure search attributes for each namespace
configure_search_attributes "adx-core-development"
configure_search_attributes "adx-core-staging"
configure_search_attributes "adx-core-production"

echo ""
echo "🎉 ADX Core Temporal namespaces setup completed successfully!"
echo ""
echo "Available namespaces:"
echo "  • adx-core-development (retention: 72h)"
echo "  • adx-core-staging (retention: 168h)"
echo "  • adx-core-production (retention: 8760h)"
echo ""
echo "You can view the namespaces in Temporal Web UI at: http://localhost:8088"
echo ""
echo "To list namespaces: $TEMPORAL_CLI --address $TEMPORAL_ADDRESS namespace list"
echo "To describe a namespace: $TEMPORAL_CLI --address $TEMPORAL_ADDRESS namespace describe --namespace <namespace-name>"