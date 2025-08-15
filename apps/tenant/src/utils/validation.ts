import { z } from 'zod';
import { SubscriptionTier, TenantRole } from '../types';

// Tenant validation schemas
export const createTenantSchema = z.object({
    name: z
        .string()
        .min(2, 'Tenant name must be at least 2 characters')
        .max(50, 'Tenant name must be less than 50 characters')
        .regex(/^[a-zA-Z0-9\s\-_]+$/, 'Tenant name can only contain letters, numbers, spaces, hyphens, and underscores'),
    slug: z
        .string()
        .min(2, 'Slug must be at least 2 characters')
        .max(30, 'Slug must be less than 30 characters')
        .regex(/^[a-z0-9\-]+$/, 'Slug can only contain lowercase letters, numbers, and hyphens')
        .optional(),
    description: z
        .string()
        .max(500, 'Description must be less than 500 characters')
        .optional(),
    subscriptionTier: z.nativeEnum(SubscriptionTier),
    adminEmail: z
        .string()
        .email('Please enter a valid email address'),
});

export const updateTenantSchema = z.object({
    name: z
        .string()
        .min(2, 'Tenant name must be at least 2 characters')
        .max(50, 'Tenant name must be less than 50 characters')
        .regex(/^[a-zA-Z0-9\s\-_]+$/, 'Tenant name can only contain letters, numbers, spaces, hyphens, and underscores')
        .optional(),
    description: z
        .string()
        .max(500, 'Description must be less than 500 characters')
        .optional(),
    settings: z.object({
        timezone: z.string().optional(),
        dateFormat: z.string().optional(),
        language: z.string().optional(),
        theme: z.enum(['light', 'dark', 'system']).optional(),
        notifications: z.object({
            email: z.boolean().optional(),
            push: z.boolean().optional(),
            sms: z.boolean().optional(),
        }).optional(),
        security: z.object({
            mfaRequired: z.boolean().optional(),
            sessionTimeout: z.number().min(5).max(1440).optional(), // 5 minutes to 24 hours
            passwordPolicy: z.object({
                minLength: z.number().min(8).max(128).optional(),
                requireUppercase: z.boolean().optional(),
                requireLowercase: z.boolean().optional(),
                requireNumbers: z.boolean().optional(),
                requireSpecialChars: z.boolean().optional(),
                maxAge: z.number().min(30).max(365).optional(), // 30 days to 1 year
            }).optional(),
        }).optional(),
        branding: z.object({
            logo: z.string().url().optional(),
            primaryColor: z.string().regex(/^#[0-9A-F]{6}$/i, 'Primary color must be a valid hex color').optional(),
            secondaryColor: z.string().regex(/^#[0-9A-F]{6}$/i, 'Secondary color must be a valid hex color').optional(),
            customDomain: z.string().url().optional(),
        }).optional(),
    }).optional(),
});

export const inviteMemberSchema = z.object({
    email: z
        .string()
        .email('Please enter a valid email address'),
    role: z.nativeEnum(TenantRole),
    message: z
        .string()
        .max(500, 'Message must be less than 500 characters')
        .optional(),
});

export const updateMemberSchema = z.object({
    role: z.nativeEnum(TenantRole).optional(),
});

// Validation helper functions
export const validateTenantName = (name: string): string | null => {
    if (!name || name.trim().length === 0) {
        return 'Tenant name is required';
    }

    if (name.length < 2) {
        return 'Tenant name must be at least 2 characters';
    }

    if (name.length > 50) {
        return 'Tenant name must be less than 50 characters';
    }

    if (!/^[a-zA-Z0-9\s\-_]+$/.test(name)) {
        return 'Tenant name can only contain letters, numbers, spaces, hyphens, and underscores';
    }

    return null;
};

export const validateEmail = (email: string): string | null => {
    if (!email || email.trim().length === 0) {
        return 'Email is required';
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
        return 'Please enter a valid email address';
    }

    return null;
};

export const validateSlug = (slug: string): string | null => {
    if (!slug || slug.trim().length === 0) {
        return null; // Slug is optional
    }

    if (slug.length < 2) {
        return 'Slug must be at least 2 characters';
    }

    if (slug.length > 30) {
        return 'Slug must be less than 30 characters';
    }

    if (!/^[a-z0-9\-]+$/.test(slug)) {
        return 'Slug can only contain lowercase letters, numbers, and hyphens';
    }

    if (slug.startsWith('-') || slug.endsWith('-')) {
        return 'Slug cannot start or end with a hyphen';
    }

    if (slug.includes('--')) {
        return 'Slug cannot contain consecutive hyphens';
    }

    return null;
};

export const generateSlugFromName = (name: string): string => {
    return name
        .toLowerCase()
        .trim()
        .replace(/[^a-z0-9\s\-]/g, '') // Remove invalid characters
        .replace(/\s+/g, '-') // Replace spaces with hyphens
        .replace(/-+/g, '-') // Replace multiple hyphens with single hyphen
        .replace(/^-|-$/g, ''); // Remove leading/trailing hyphens
};

export const validateHexColor = (color: string): string | null => {
    if (!color || color.trim().length === 0) {
        return null; // Color is optional
    }

    if (!/^#[0-9A-F]{6}$/i.test(color)) {
        return 'Color must be a valid hex color (e.g., #FF0000)';
    }

    return null;
};

export const validateUrl = (url: string): string | null => {
    if (!url || url.trim().length === 0) {
        return null; // URL is optional
    }

    try {
        new URL(url);
        return null;
    } catch {
        return 'Please enter a valid URL';
    }
};

// Form validation helpers
export type ValidationErrors<T> = Partial<Record<keyof T, string>>;

export const validateForm = <T extends Record<string, any>>(
    data: T,
    schema: z.ZodSchema<T>
): { isValid: boolean; errors: ValidationErrors<T> } => {
    try {
        schema.parse(data);
        return { isValid: true, errors: {} };
    } catch (error) {
        if (error instanceof z.ZodError) {
            const errors: ValidationErrors<T> = {};
            error.errors.forEach((err) => {
                if (err.path.length > 0) {
                    const field = err.path[0] as keyof T;
                    errors[field] = err.message;
                }
            });
            return { isValid: false, errors };
        }
        return { isValid: false, errors: {} };
    }
};