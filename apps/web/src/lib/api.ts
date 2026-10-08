@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap');

:root {
  --primary: #0066cc;
  --primary-dark: #0052a3;
  --neutral-50: #f8fafc;
  --neutral-100: #f1f5f9;
  --neutral-200: #e2e8f0;
  --neutral-300: #cbd5e1;
  --neutral-400: #94a3b8;
  --neutral-500: #64748b;
  --neutral-600: #475569;
  --neutral-700: #334155;
  --neutral-800: #1e293b;
  --neutral-900: #0f172a;
  --success: #16a34a;
  --warning: #f59e0b;
  --error: #dc2626;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--neutral-50);
  color: var(--neutral-900);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  min-height: 100vh;
  line-height: 1.5;
}

a {
  color: var(--primary);
  text-decoration: none;
}

button, input, select {
  font: inherit;
}

button {
  cursor: pointer;
  border: none;
}

input, select {
  color: var(--neutral-900);
}

input:focus,
select:focus,
button:focus {
  outline: 2px solid rgba(0, 102, 204, 0.16);
  outline-offset: 2px;
}
