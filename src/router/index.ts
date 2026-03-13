import { createRouter, createWebHashHistory } from 'vue-router'
import CredentialsView from '../views/CredentialsView.vue'
import HomeView from '../views/HomeView.vue'
import CreateDoiView from '../views/CreateDoiView.vue'
import DoiListView from '../views/DoiListView.vue'

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/home' },
    { path: '/home', component: HomeView },
    { path: '/create-doi', component: CreateDoiView },
    { path: '/doi-list', component: DoiListView },
    { path: '/credentials', component: CredentialsView },
  ],
})
