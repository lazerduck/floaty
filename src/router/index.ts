import { createRouter, createWebHistory } from 'vue-router'
import Home from '../pages/Home.vue'
import About from '../pages/About.vue'
import Store from '../pages/JsonStore.vue'
import Music from '../pages/Music.vue'

const routes = [
  { path: '/', name: 'Home', component: Home },
  { path: '/store', name: 'Store', component: Store},
  { path: '/about', name: 'About', component: About },
  { path: '/music', name: 'Music', component: Music },
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
