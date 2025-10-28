import { createApp } from "vue";
import { Quasar } from "quasar";
import quasarLang from "quasar/lang/en-GB";

import '@quasar/extras/material-icons/material-icons.css';
import 'quasar/src/css/index.sass';

import App from "./App.vue";
import router from './router'

const myApp = createApp(App);

myApp.use(Quasar, {
  plugins: {},
  lang: quasarLang,
  config: { dark: true }
});

myApp.use(router)

myApp.mount("#app");