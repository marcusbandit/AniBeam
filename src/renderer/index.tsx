import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './styles/index.css';
import App from './App';
import { initLiquidGlass } from './utils/liquidGlass';

const rootElement = document.getElementById('root');
if (!rootElement) {
  throw new Error('Root element not found');
}

// Refracting glass for [data-liquid-glass] surfaces (Chromium-only tech,
// which Electron guarantees). Safe to start before React mounts.
initLiquidGlass();

try {
  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
} catch (error) {
  console.error('Failed to render app:', error);
  rootElement.innerHTML = `
    <div style="padding: 2rem; color: #f43f5e; font-family: sans-serif;">
      <h2>Failed to load application</h2>
      <p>${error instanceof Error ? error.message : String(error)}</p>
      <pre style="background: #1a1a24; padding: 1rem; border-radius: 8px; overflow: auto;">
        ${error instanceof Error ? error.stack : String(error)}
      </pre>
    </div>
  `;
}
