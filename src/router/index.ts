import { createRouter, createWebHashHistory } from 'vue-router'
import CredentialsView from '../views/CredentialsView.vue'
import HomeView from '../views/HomeView.vue'
import CreateDoiView from '../views/CreateDoiView.vue'
import DoiListView from '../views/DoiListView.vue'
import BatchDoiView from '../views/BatchDoiView.vue'

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/home' },
    { path: '/home', component: HomeView },
    { path: '/create-doi', component: CreateDoiView },
    { path: '/doi-list', component: DoiListView },
    { path: '/batch-doi', component: BatchDoiView },
    { path: '/credentials', component: CredentialsView },
  ],
})
