import { createApp } from "vue";
import { createPinia } from 'pinia'
import { Quasar } from "quasar";
import quasarLang from "quasar/lang/en-GB";

import '@quasar/extras/material-icons/material-icons.css';
import 'quasar/src/css/index.sass';

import App from "./App.vue";
import router from './router'

const pinia = createPinia();
const myApp = createApp(App);

myApp.use(pinia);

myApp.use(Quasar, {
  plugins: {},
  lang: quasarLang,
  config: { dark: true }
});

myApp.use(router)

myApp.mount("#app");