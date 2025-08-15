import { UpdateUserRequest, UpdateUserProfileRequest, CreateUserRequest } from '../types';

export interface ValidationError {
  field: string;
  message: string;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
}

// Email validation
export const isValidEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

// Phone validation
export const isValidPhone = (phone: string): boolean => {
  const phoneRegex = /^\+?[\d\s\-\(\)]+$/;
  return phoneRegex.test(phone) && phone.replace(/\D/g, '').length >= 10;
};

// URL validation
export const isValidUrl = (url: string): boolean => {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
};

// Validate user creation request
export const validateCreateUserRequest = (data: CreateUserRequest): ValidationResult => {
  const errors: ValidationError[] = [];

  if (!data.email?.trim()) {
    errors.push({ field: 'email', message: 'Email is required' });
  } else if (!isValidEmail(data.email)) {
    errors.push({ field: 'email', message: 'Please enter a valid email address' });
  }

  if (!data.firstName?.trim()) {
    errors.push({ field: 'firstName', message: 'First name is required' });
  } else if (data.firstName.length < 2) {
    errors.push({ field: 'firstName', message: 'First name must be at least 2 characters' });
  }

  if (!data.lastName?.trim()) {
    errors.push({ field: 'lastName', message: 'Last name is required' });
  } else if (data.lastName.length < 2) {
    errors.push({ field: 'lastName', message: 'Last name must be at least 2 characters' });
  }

  if (!data.roles || data.roles.length === 0) {
    errors.push({ field: 'roles', message: 'At least one role is required' });
  }

  if (!data.tenantId?.trim()) {
    errors.push({ field: 'tenantId', message: 'Tenant ID is required' });
  }

  return {
    isValid: errors.length === 0,
    errors,
  };
};

// Validate user update request
export const validateUpdateUserRequest = (data: UpdateUserRequest): ValidationResult => {
  const errors: ValidationError[] = [];

  if (data.firstName !== undefined) {
    if (!data.firstName?.trim()) {
      errors.push({ field: 'firstName', message: 'First name cannot be empty' });
    } else if (data.firstName.length < 2) {
      errors.push({ field: 'firstName', message: 'First name must be at least 2 characters' });
    }
  }

  if (data.lastName !== undefined) {
    if (!data.lastName?.trim()) {
      errors.push({ field: 'lastName', message: 'Last name cannot be empty' });
    } else if (data.lastName.length < 2) {
      errors.push({ field: 'lastName', message: 'Last name must be at least 2 characters' });
    }
  }

  if (data.phone !== undefined && data.phone && !isValidPhone(data.phone)) {
    errors.push({ field: 'phone', message: 'Please enter a valid phone number' });
  }

  if (data.roles !== undefined && (!data.roles || data.roles.length === 0)) {
    errors.push({ field: 'roles', message: 'At least one role is required' });
  }

  return {
    isValid: errors.length === 0,
    errors,
  };
};

// Validate user profile update request
export const validateUpdateUserProfileRequest = (data: UpdateUserProfileRequest): ValidationResult => {
  const errors: ValidationError[] = [];

  if (data.website !== undefined && data.website && !isValidUrl(data.website)) {
    errors.push({ field: 'website', message: 'Please enter a valid website URL' });
  }

  if (data.socialLinks) {
    if (data.socialLinks.linkedin && !isValidUrl(data.socialLinks.linkedin)) {
      errors.push({ field: 'socialLinks.linkedin', message: 'Please enter a valid LinkedIn URL' });
    }
    if (data.socialLinks.twitter && !isValidUrl(data.socialLinks.twitter)) {
      errors.push({ field: 'socialLinks.twitter', message: 'Please enter a valid Twitter URL' });
    }
    if (data.socialLinks.github && !isValidUrl(data.socialLinks.github)) {
      errors.push({ field: 'socialLinks.github', message: 'Please enter a valid GitHub URL' });
    }
  }

  if (data.bio !== undefined && data.bio && data.bio.length > 500) {
    errors.push({ field: 'bio', message: 'Bio must be 500 characters or less' });
  }

  return {
    isValid: errors.length === 0,
    errors,
  };
};

// Get validation error for a specific field
export const getFieldError = (errors: ValidationError[], field: string): string | undefined => {
  const error = errors.find(e => e.field === field);
  return error?.message;
};

// Check if field has error
export const hasFieldError = (errors: ValidationError[], field: string): boolean => {
  return errors.some(e => e.field === field);
};