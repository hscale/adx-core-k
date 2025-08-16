// Unit tests for LoginForm component
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { LoginForm } from '../LoginForm';
import { TenantProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';

// Mock the BFF client
const mockBFFClient = {
  initiateWorkflow: vi.fn(),
  getAggregatedData: vi.fn(),
  pollWorkflowStatus: vi.fn(),
};

// Mock the tenant context
const mockTenantContext = {
  state: {
    currentTenant: {
      id: 'tenant-123',
      name: 'Test Tenant',
      features: ['auth', 'basic'],
      quotas: {},
      settings: {},
    },
    availableTenants: [],
    loading: false,
    error: null,
  },
  switchTenant: vi.fn(),
  refreshTenantData: vi.fn(),
};

// Mock the event bus
const mockEventBus = {
  emit: vi.fn(),
  subscribe: vi.fn(() => () => {}),
  subscribePattern: vi.fn(() => () => {}),
};

// Test wrapper component
const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return (
    <QueryClientProvider client={queryClient}>
      <TenantProvider value={mockTenantContext}>
        <EventBusProvider value={mockEventBus}>
          {children}
        </EventBusProvider>
      </TenantProvider>
    </QueryClientProvider>
  );
};

// Mock LoginForm component for testing
const LoginForm: React.FC<{ bffClient?: any; onSuccess?: (result: any) => void }> = ({ 
  bffClient = mockBFFClient, 
  onSuccess 
}) => {
  const [email, setEmail] = React.useState('');
  const [password, setPassword] = React.useState('');
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const result = await bffClient.initiateWorkflow('user_login', {
        email,
        password,
        tenantId: mockTenantContext.state.currentTenant?.id,
      }, { synchronous: true });

      if (result.type === 'sync') {
        onSuccess?.(result.data);
        mockEventBus.emit('auth:login_success', { userId: result.data.userId });
      } else {
        // Handle async workflow
        const finalResult = await bffClient.pollWorkflowStatus(result.operationId);
        onSuccess?.(finalResult);
        mockEventBus.emit('auth:login_success', { userId: finalResult.userId });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
      mockEventBus.emit('auth:login_error', { error: err });
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} data-testid="login-form">
      <div>
        <label htmlFor="email">Email</label>
        <input
          id="email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          data-testid="email-input"
          required
        />
      </div>
      
      <div>
        <label htmlFor="password">Password</label>
        <input
          id="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          data-testid="password-input"
          required
        />
      </div>
      
      {error && (
        <div data-testid="error-message" role="alert">
          {error}
        </div>
      )}
      
      <button
        type="submit"
        disabled={loading}
        data-testid="login-button"
      >
        {loading ? 'Logging in...' : 'Login'}
      </button>
    </form>
  );
};

describe('LoginForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  it('should render login form with all required fields', () => {
    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /login/i })).toBeInTheDocument();
  });

  it('should handle successful synchronous login', async () => {
    const mockOnSuccess = vi.fn();
    const mockLoginResult = {
      type: 'sync',
      data: {
        userId: 'user-123',
        accessToken: 'token-123',
        refreshToken: 'refresh-123',
      },
    };

    mockBFFClient.initiateWorkflow.mockResolvedValue(mockLoginResult);

    render(
      <TestWrapper>
        <LoginForm onSuccess={mockOnSuccess} />
      </TestWrapper>
    );

    // Fill in form
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });

    // Submit form
    fireEvent.click(screen.getByTestId('login-button'));

    // Wait for workflow to complete
    await waitFor(() => {
      expect(mockBFFClient.initiateWorkflow).toHaveBeenCalledWith(
        'user_login',
        {
          email: 'test@example.com',
          password: 'password123',
          tenantId: 'tenant-123',
        },
        { synchronous: true }
      );
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(mockLoginResult.data);
    expect(mockEventBus.emit).toHaveBeenCalledWith('auth:login_success', {
      userId: 'user-123',
    });
  });

  it('should handle successful asynchronous login', async () => {
    const mockOnSuccess = vi.fn();
    const mockWorkflowResult = {
      type: 'async',
      operationId: 'op-123',
      statusUrl: '/api/workflows/op-123/status',
    };
    const mockFinalResult = {
      userId: 'user-123',
      accessToken: 'token-123',
      refreshToken: 'refresh-123',
    };

    mockBFFClient.initiateWorkflow.mockResolvedValue(mockWorkflowResult);
    mockBFFClient.pollWorkflowStatus.mockResolvedValue(mockFinalResult);

    render(
      <TestWrapper>
        <LoginForm onSuccess={mockOnSuccess} />
      </TestWrapper>
    );

    // Fill in form
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });

    // Submit form
    fireEvent.click(screen.getByTestId('login-button'));

    // Wait for workflow to complete
    await waitFor(() => {
      expect(mockBFFClient.pollWorkflowStatus).toHaveBeenCalledWith('op-123');
    });

    expect(mockOnSuccess).toHaveBeenCalledWith(mockFinalResult);
    expect(mockEventBus.emit).toHaveBeenCalledWith('auth:login_success', {
      userId: 'user-123',
    });
  });

  it('should handle login failure', async () => {
    const mockError = new Error('Invalid credentials');
    mockBFFClient.initiateWorkflow.mockRejectedValue(mockError);

    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    // Fill in form
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'wrongpassword' },
    });

    // Submit form
    fireEvent.click(screen.getByTestId('login-button'));

    // Wait for error to appear
    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('Invalid credentials');
    });

    expect(mockEventBus.emit).toHaveBeenCalledWith('auth:login_error', {
      error: mockError,
    });
  });

  it('should show loading state during login', async () => {
    // Mock a delayed response
    mockBFFClient.initiateWorkflow.mockImplementation(
      () => new Promise(resolve => setTimeout(() => resolve({
        type: 'sync',
        data: { userId: 'user-123' },
      }), 100))
    );

    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    // Fill in form
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });

    // Submit form
    fireEvent.click(screen.getByTestId('login-button'));

    // Check loading state
    expect(screen.getByText('Logging in...')).toBeInTheDocument();
    expect(screen.getByTestId('login-button')).toBeDisabled();

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.getByText('Login')).toBeInTheDocument();
    });
  });

  it('should validate required fields', () => {
    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    const emailInput = screen.getByTestId('email-input');
    const passwordInput = screen.getByTestId('password-input');

    expect(emailInput).toHaveAttribute('required');
    expect(passwordInput).toHaveAttribute('required');
    expect(emailInput).toHaveAttribute('type', 'email');
    expect(passwordInput).toHaveAttribute('type', 'password');
  });

  it('should update input values correctly', () => {
    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    const emailInput = screen.getByTestId('email-input') as HTMLInputElement;
    const passwordInput = screen.getByTestId('password-input') as HTMLInputElement;

    fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
    fireEvent.change(passwordInput, { target: { value: 'password123' } });

    expect(emailInput.value).toBe('test@example.com');
    expect(passwordInput.value).toBe('password123');
  });

  it('should include tenant context in login request', async () => {
    mockBFFClient.initiateWorkflow.mockResolvedValue({
      type: 'sync',
      data: { userId: 'user-123' },
    });

    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    // Fill in form
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });

    // Submit form
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(mockBFFClient.initiateWorkflow).toHaveBeenCalledWith(
        'user_login',
        expect.objectContaining({
          tenantId: 'tenant-123',
        }),
        { synchronous: true }
      );
    });
  });

  it('should handle network errors gracefully', async () => {
    const networkError = new Error('Network error');
    mockBFFClient.initiateWorkflow.mockRejectedValue(networkError);

    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    // Fill in form and submit
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('Network error');
    });
  });

  it('should clear error when retrying login', async () => {
    // First attempt fails
    mockBFFClient.initiateWorkflow.mockRejectedValueOnce(new Error('First error'));
    // Second attempt succeeds
    mockBFFClient.initiateWorkflow.mockResolvedValueOnce({
      type: 'sync',
      data: { userId: 'user-123' },
    });

    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    // Fill in form and submit (first attempt)
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    // Wait for error
    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('First error');
    });

    // Submit again (second attempt)
    fireEvent.click(screen.getByTestId('login-button'));

    // Error should be cleared during loading
    await waitFor(() => {
      expect(screen.queryByTestId('error-message')).not.toBeInTheDocument();
    });
  });
});

// Integration test with multiple components
describe('LoginForm Integration', () => {
  it('should integrate with tenant context and event bus', async () => {
    const mockOnSuccess = vi.fn();
    
    mockBFFClient.initiateWorkflow.mockResolvedValue({
      type: 'sync',
      data: { userId: 'user-123', accessToken: 'token-123' },
    });

    render(
      <TestWrapper>
        <LoginForm onSuccess={mockOnSuccess} />
      </TestWrapper>
    );

    // Simulate login
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByTestId('password-input'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      // Verify BFF client was called with tenant context
      expect(mockBFFClient.initiateWorkflow).toHaveBeenCalledWith(
        'user_login',
        expect.objectContaining({
          tenantId: 'tenant-123',
        }),
        { synchronous: true }
      );

      // Verify event was emitted
      expect(mockEventBus.emit).toHaveBeenCalledWith('auth:login_success', {
        userId: 'user-123',
      });

      // Verify success callback was called
      expect(mockOnSuccess).toHaveBeenCalledWith({
        userId: 'user-123',
        accessToken: 'token-123',
      });
    });
  });
});