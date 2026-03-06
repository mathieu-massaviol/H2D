import { createRouter, createWebHashHistory } from 'vue-router'
import CredentialsView from '../views/CredentialsView.vue'

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/credentials' },
    { path: '/credentials', component: CredentialsView },
  ],
})
