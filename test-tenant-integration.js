#!/usr/bin/env node

/**
 * Integration test script for Tenant Micro-Frontend and BFF Service
 * Tests the complete integration between the tenant micro-frontend and BFF service
 */

import fetch from 'node-fetch';

const TENANT_FRONTEND_URL = 'http://localhost:3002';
const TENANT_BFF_URL = 'http://localhost:4002';

async function testService(url, name) {
  try {
    console.log(`🔍 Testing ${name} at ${url}...`);
    const response = await fetch(url);
    
    if (response.ok) {
      console.log(`✅ ${name} is running and accessible`);
      return true;
    } else {
      console.log(`❌ ${name} returned status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`❌ ${name} is not accessible: ${error.message}`);
    return false;
  }
}

async function testBFFHealthCheck() {
  try {
    console.log('🔍 Testing BFF health check...');
    const response = await fetch(`${TENANT_BFF_URL}/health`);
    
    if (response.ok) {
      const data = await response.json();
      console.log('✅ BFF health check passed:', data);
      return true;
    } else {
      console.log(`❌ BFF health check failed with status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`❌ BFF health check failed: ${error.message}`);
    return false;
  }
}

async function testBFFTenantAPI() {
  try {
    console.log('🔍 Testing BFF tenant API...');
    const response = await fetch(`${TENANT_BFF_URL}/api/tenant/current`, {
      headers: {
        'Authorization': 'Bearer mock-token',
        'X-Tenant-ID': 'tenant-1',
      },
    });
    
    if (response.ok) {
      const data = await response.json();
      console.log('✅ BFF tenant API working:', data.name);
      return true;
    } else {
      console.log(`❌ BFF tenant API failed with status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`❌ BFF tenant API failed: ${error.message}`);
    return false;
  }
}

async function runIntegrationTests() {
  console.log('🚀 Starting Tenant Micro-Frontend Integration Tests\n');
  
  const results = {
    frontendAccessible: false,
    bffAccessible: false,
    bffHealthCheck: false,
    bffTenantAPI: false,
  };
  
  // Test frontend accessibility
  results.frontendAccessible = await testService(TENANT_FRONTEND_URL, 'Tenant Frontend');
  
  // Test BFF accessibility
  results.bffAccessible = await testService(TENANT_BFF_URL, 'Tenant BFF');
  
  // Test BFF health check
  if (results.bffAccessible) {
    results.bffHealthCheck = await testBFFHealthCheck();
  }
  
  // Test BFF tenant API
  if (results.bffHealthCheck) {
    results.bffTenantAPI = await testBFFTenantAPI();
  }
  
  // Summary
  console.log('\n📊 Integration Test Results:');
  console.log('================================');
  console.log(`Frontend Accessible: ${results.frontendAccessible ? '✅' : '❌'}`);
  console.log(`BFF Service Accessible: ${results.bffAccessible ? '✅' : '❌'}`);
  console.log(`BFF Health Check: ${results.bffHealthCheck ? '✅' : '❌'}`);
  console.log(`BFF Tenant API: ${results.bffTenantAPI ? '✅' : '❌'}`);
  
  const allPassed = Object.values(results).every(result => result === true);
  
  if (allPassed) {
    console.log('\n🎉 All integration tests passed!');
    console.log('✅ Tenant Micro-Frontend is properly set up and integrated with BFF service');
  } else {
    console.log('\n⚠️  Some integration tests failed');
    console.log('💡 Make sure both services are running:');
    console.log('   - Tenant Frontend: npm run dev (in apps/tenant)');
    console.log('   - Tenant BFF: npm run dev (in bff-services/tenant-bff)');
  }
  
  return allPassed;
}

// Run the tests
runIntegrationTests()
  .then(success => {
    process.exit(success ? 0 : 1);
  })
  .catch(error => {
    console.error('❌ Integration test failed:', error);
    process.exit(1);
  });