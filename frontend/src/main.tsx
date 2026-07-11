import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

const root = document.getElementById('root');
if (!root) throw new Error('找不到 #root 元素');
ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
