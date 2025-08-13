/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_AUTH_BFF_URL: string;
  readonly VITE_API_GATEWAY_URL: string;
  readonly VITE_NODE_ENV: string;
  readonly VITE_DEBUG: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
  readonly hot?: {
    accept(): void;
  };
}