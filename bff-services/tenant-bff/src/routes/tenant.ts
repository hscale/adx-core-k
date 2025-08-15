import { Router, Request, Response } from 'express';
import { RedisClient } from '../services/redis.js';
import { apiGatewayClient, tenantServiceClient } from '../services/apiClient.js';
import { Tenant, TenantMember, TenantInvitation, SubscriptionTier, TenantStatus, TenantRole, MemberStatus, InvitationStatus } from '../types/tenant.js';

export const createTenantRoutes = (redisClient: RedisClient) => {
  const router = Router();

  // Get current tenant (with caching)
  router.get('/tenant/current', async (req: Request, res: Response) => {
    try {
      const tenantId = req.tenant?.tenantId;
      if (!tenantId) {
        return res.status(400).json({
          error: { code: 'TENANT_ID_REQUIRED', message: 'Tenant ID is required' }
        });
      }

      // Check cache first
      const cacheKey = `tenant:${tenantId}`;
      const cached = await redisClient.get(cacheKey);
      
      if (cached) {
        return res.json(JSON.parse(cached));
      }

      // Mock tenant data (in real implementation, this would call the tenant service)
      const tenant: Tenant = {
        id: tenantId,
        name: 'Demo Tenant',
        slug: 'demo-tenant',
        description: 'Demo tenant for development',
        adminEmail: 'admin@demo.com',
        subscriptionTier: SubscriptionTier.PROFESSIONAL,
        status: TenantStatus.ACTIVE,
        features: ['basic_features', 'advanced_analytics'],
        quotas: {
          maxUsers: 10,
          maxStorageGB: 100,
          maxApiCallsPerHour: 1000,
          maxWorkflowsPerHour: 50,
          currentUsers: 3,
          currentStorageGB: 25.5,
          currentApiCallsThisHour: 150,
          currentWorkflowsThisHour: 5,
        },
        settings: {
          timezone: 'UTC',
          dateFormat: 'MM/DD/YYYY',
          language: 'en',
          theme: 'system',
          notifications: {
            email: true,
            push: true,
            sms: false,
          },
          security: {
            mfaRequired: false,
            sessionTimeout: 60,
            passwordPolicy: {
              minLength: 8,
              requireUppercase: true,
              requireLowercase: true,
              requireNumbers: true,
              requireSpecialChars: false,
              maxAge: 90,
            },
          },
          branding: {
            primaryColor: '#3B82F6',
            secondaryColor: '#6B7280',
            customDomain: 'https://demo.adxcore.com',
          },
        },
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-15T10:30:00Z',
      };

      // Cache for 5 minutes
      await redisClient.set(cacheKey, JSON.stringify(tenant), 300);

      res.json(tenant);
    } catch (error) {
      console.error('Error fetching current tenant:', error);
      res.status(500).json({
        error: { code: 'FETCH_TENANT_ERROR', message: 'Failed to fetch tenant' }
      });
    }
  });

  // Get user's tenants (with caching)
  router.get('/tenants', async (req: Request, res: Response) => {
    try {
      const userId = req.auth?.userId;
      if (!userId) {
        return res.status(401).json({
          error: { code: 'USER_ID_REQUIRED', message: 'User ID is required' }
        });
      }

      // Check cache first
      const cacheKey = `user_tenants:${userId}`;
      const cached = await redisClient.get(cacheKey);
      
      if (cached) {
        return res.json(JSON.parse(cached));
      }

      // Mock tenants data (in real implementation, this would call the tenant service)
      const tenants: Tenant[] = [
        {
          id: 'tenant-1',
          name: 'Demo Tenant',
          slug: 'demo-tenant',
          description: 'Demo tenant for development',
          adminEmail: 'admin@demo.com',
          subscriptionTier: SubscriptionTier.PROFESSIONAL,
          status: TenantStatus.ACTIVE,
          features: ['basic_features', 'advanced_analytics'],
          quotas: {
            maxUsers: 10,
            maxStorageGB: 100,
            maxApiCallsPerHour: 1000,
            maxWorkflowsPerHour: 50,
            currentUsers: 3,
            currentStorageGB: 25.5,
            currentApiCallsThisHour: 150,
            currentWorkflowsThisHour: 5,
          },
          settings: {
            timezone: 'UTC',
            dateFormat: 'MM/DD/YYYY',
            language: 'en',
            theme: 'system',
            notifications: { email: true, push: true, sms: false },
            security: {
              mfaRequired: false,
              sessionTimeout: 60,
              passwordPolicy: {
                minLength: 8,
                requireUppercase: true,
                requireLowercase: true,
                requireNumbers: true,
                requireSpecialChars: false,
                maxAge: 90,
              },
            },
            branding: { primaryColor: '#3B82F6', secondaryColor: '#6B7280', customDomain: 'https://demo.adxcore.com' },
          },
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-15T10:30:00Z',
        },
        {
          id: 'tenant-2',
          name: 'Test Tenant',
          slug: 'test-tenant',
          description: 'Test tenant for development',
          adminEmail: 'admin@test.com',
          subscriptionTier: SubscriptionTier.FREE,
          status: TenantStatus.ACTIVE,
          features: ['basic_features'],
          quotas: {
            maxUsers: 5,
            maxStorageGB: 10,
            maxApiCallsPerHour: 100,
            maxWorkflowsPerHour: 10,
            currentUsers: 1,
            currentStorageGB: 2.1,
            currentApiCallsThisHour: 25,
            currentWorkflowsThisHour: 2,
          },
          settings: {
            timezone: 'UTC',
            dateFormat: 'MM/DD/YYYY',
            language: 'en',
            theme: 'light',
            notifications: { email: true, push: false, sms: false },
            security: {
              mfaRequired: false,
              sessionTimeout: 30,
              passwordPolicy: {
                minLength: 8,
                requireUppercase: false,
                requireLowercase: true,
                requireNumbers: true,
                requireSpecialChars: false,
                maxAge: 180,
              },
            },
            branding: { primaryColor: '#10B981', secondaryColor: '#6B7280' },
          },
          createdAt: '2024-01-10T00:00:00Z',
          updatedAt: '2024-01-15T10:30:00Z',
        },
      ];

      // Cache for 5 minutes
      await redisClient.set(cacheKey, JSON.stringify(tenants), 300);

      res.json(tenants);
    } catch (error) {
      console.error('Error fetching user tenants:', error);
      res.status(500).json({
        error: { code: 'FETCH_TENANTS_ERROR', message: 'Failed to fetch tenants' }
      });
    }
  });

  // Get specific tenant
  router.get('/tenants/:id', async (req: Request, res: Response) => {
    try {
      const tenantId = req.params.id;
      
      // Check cache first
      const cacheKey = `tenant:${tenantId}`;
      const cached = await redisClient.get(cacheKey);
      
      if (cached) {
        return res.json(JSON.parse(cached));
      }

      // In real implementation, call tenant service
      // For now, return mock data
      const tenant: Tenant = {
        id: tenantId,
        name: 'Demo Tenant',
        slug: 'demo-tenant',
        description: 'Demo tenant for development',
        adminEmail: 'admin@demo.com',
        subscriptionTier: SubscriptionTier.PROFESSIONAL,
        status: TenantStatus.ACTIVE,
        features: ['basic_features', 'advanced_analytics'],
        quotas: {
          maxUsers: 10,
          maxStorageGB: 100,
          maxApiCallsPerHour: 1000,
          maxWorkflowsPerHour: 50,
          currentUsers: 3,
          currentStorageGB: 25.5,
          currentApiCallsThisHour: 150,
          currentWorkflowsThisHour: 5,
        },
        settings: {
          timezone: 'UTC',
          dateFormat: 'MM/DD/YYYY',
          language: 'en',
          theme: 'system',
          notifications: { email: true, push: true, sms: false },
          security: {
            mfaRequired: false,
            sessionTimeout: 60,
            passwordPolicy: {
              minLength: 8,
              requireUppercase: true,
              requireLowercase: true,
              requireNumbers: true,
              requireSpecialChars: false,
              maxAge: 90,
            },
          },
          branding: { primaryColor: '#3B82F6', secondaryColor: '#6B7280', customDomain: 'https://demo.adxcore.com' },
        },
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-15T10:30:00Z',
      };

      // Cache for 5 minutes
      await redisClient.set(cacheKey, JSON.stringify(tenant), 300);

      res.json(tenant);
    } catch (error) {
      console.error('Error fetching tenant:', error);
      res.status(500).json({
        error: { code: 'FETCH_TENANT_ERROR', message: 'Failed to fetch tenant' }
      });
    }
  });

  // Update tenant
  router.put('/tenants/:id', async (req: Request, res: Response) => {
    try {
      const tenantId = req.params.id;
      const updateData = req.body;

      // In real implementation, call tenant service
      // For now, return updated mock data
      const updatedTenant: Tenant = {
        id: tenantId,
        name: updateData.name || 'Demo Tenant',
        slug: 'demo-tenant',
        description: updateData.description || 'Demo tenant for development',
        adminEmail: 'admin@demo.com',
        subscriptionTier: SubscriptionTier.PROFESSIONAL,
        status: TenantStatus.ACTIVE,
        features: ['basic_features', 'advanced_analytics'],
        quotas: {
          maxUsers: 10,
          maxStorageGB: 100,
          maxApiCallsPerHour: 1000,
          maxWorkflowsPerHour: 50,
          currentUsers: 3,
          currentStorageGB: 25.5,
          currentApiCallsThisHour: 150,
          currentWorkflowsThisHour: 5,
        },
        settings: updateData.settings || {
          timezone: 'UTC',
          dateFormat: 'MM/DD/YYYY',
          language: 'en',
          theme: 'system',
          notifications: { email: true, push: true, sms: false },
          security: {
            mfaRequired: false,
            sessionTimeout: 60,
            passwordPolicy: {
              minLength: 8,
              requireUppercase: true,
              requireLowercase: true,
              requireNumbers: true,
              requireSpecialChars: false,
              maxAge: 90,
            },
          },
          branding: { primaryColor: '#3B82F6', secondaryColor: '#6B7280', customDomain: 'https://demo.adxcore.com' },
        },
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: new Date().toISOString(),
      };

      // Invalidate cache
      await redisClient.del(`tenant:${tenantId}`);
      await redisClient.del(`user_tenants:${req.auth?.userId}`);

      res.json(updatedTenant);
    } catch (error) {
      console.error('Error updating tenant:', error);
      res.status(500).json({
        error: { code: 'UPDATE_TENANT_ERROR', message: 'Failed to update tenant' }
      });
    }
  });

  // Get tenant members
  router.get('/tenants/:id/members', async (req: Request, res: Response) => {
    try {
      const tenantId = req.params.id;
      
      // Check cache first
      const cacheKey = `tenant_members:${tenantId}`;
      const cached = await redisClient.get(cacheKey);
      
      if (cached) {
        return res.json(JSON.parse(cached));
      }

      // Mock members data
      const members: TenantMember[] = [
        {
          id: 'member-1',
          userId: 'user-1',
          tenantId,
          email: 'admin@demo.com',
          name: 'Admin User',
          role: TenantRole.OWNER,
          status: MemberStatus.ACTIVE,
          joinedAt: '2024-01-01T00:00:00Z',
          lastActiveAt: '2024-01-15T10:00:00Z',
        },
        {
          id: 'member-2',
          userId: 'user-2',
          tenantId,
          email: 'john@demo.com',
          name: 'John Doe',
          role: TenantRole.ADMIN,
          status: MemberStatus.ACTIVE,
          joinedAt: '2024-01-05T00:00:00Z',
          lastActiveAt: '2024-01-15T09:30:00Z',
        },
        {
          id: 'member-3',
          userId: 'user-3',
          tenantId,
          email: 'jane@demo.com',
          name: 'Jane Smith',
          role: TenantRole.MEMBER,
          status: MemberStatus.ACTIVE,
          joinedAt: '2024-01-10T00:00:00Z',
          lastActiveAt: '2024-01-14T16:45:00Z',
        },
      ];

      // Cache for 2 minutes
      await redisClient.set(cacheKey, JSON.stringify(members), 120);

      res.json(members);
    } catch (error) {
      console.error('Error fetching tenant members:', error);
      res.status(500).json({
        error: { code: 'FETCH_MEMBERS_ERROR', message: 'Failed to fetch members' }
      });
    }
  });

  // Get tenant invitations
  router.get('/tenants/:id/invitations', async (req: Request, res: Response) => {
    try {
      const tenantId = req.params.id;
      
      // Check cache first
      const cacheKey = `tenant_invitations:${tenantId}`;
      const cached = await redisClient.get(cacheKey);
      
      if (cached) {
        return res.json(JSON.parse(cached));
      }

      // Mock invitations data
      const invitations: TenantInvitation[] = [
        {
          id: 'invitation-1',
          tenantId,
          email: 'newuser@demo.com',
          role: TenantRole.MEMBER,
          invitedBy: 'user-1',
          invitedAt: '2024-01-14T10:00:00Z',
          expiresAt: '2024-01-21T10:00:00Z',
          status: InvitationStatus.PENDING,
          token: 'invite-token-123',
        },
        {
          id: 'invitation-2',
          tenantId,
          email: 'expired@demo.com',
          role: TenantRole.VIEWER,
          invitedBy: 'user-1',
          invitedAt: '2024-01-01T10:00:00Z',
          expiresAt: '2024-01-08T10:00:00Z',
          status: InvitationStatus.EXPIRED,
          token: 'invite-token-456',
        },
      ];

      // Cache for 2 minutes
      await redisClient.set(cacheKey, JSON.stringify(invitations), 120);

      res.json(invitations);
    } catch (error) {
      console.error('Error fetching tenant invitations:', error);
      res.status(500).json({
        error: { code: 'FETCH_INVITATIONS_ERROR', message: 'Failed to fetch invitations' }
      });
    }
  });

  return router;
};