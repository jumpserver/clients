import './styles/main.css';
import App from './App.vue';
import { createApp } from 'vue';
import { i18n } from '@renderer/lang';
import { pinia } from '@renderer/store';
import { router } from '@renderer/router';

const app = createApp(App);

app.use(i18n);
app.use(pinia);
app.use(router);

app.mount('#app');
