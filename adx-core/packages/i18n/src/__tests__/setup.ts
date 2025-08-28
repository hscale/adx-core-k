import '@testing-library/jest-dom';

// Mock fetch for translation loading
global.fetch = jest.fn(() =>
  Promise.resolve({
    ok: true,
    json: () => Promise.resolve({
      loading: 'Loading...',
      save: 'Save',
      title: 'Authentication',
    }),
  })
) as jest.Mock;