import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from './App';
import './index.css';

// Mount the app when running standalone
if (!(window as any).__POWERED_BY_QIANKUN__) {
  const root = ReactDOM.createRoot(document.getElementById('root')!);
  root.render(
    <React.StrictMode>
      <BrowserRouter basename="/users">
        <App />
      </BrowserRouter>
    </React.StrictMode>
  );
}

// Export for Module Federation
export default App;