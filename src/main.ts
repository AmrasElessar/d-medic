import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { i18n } from './i18n';
import { installDevLogger } from './utils/devlog';
import './assets/styles/main.css';

const app = createApp(App);
app.use(createPinia());
app.use(i18n);
// Dev modda frontend hatalarını dev terminaline köprüle (no-op release'de).
installDevLogger(app);
app.mount('#app');
