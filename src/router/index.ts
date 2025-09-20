import { createMemoryHistory, createRouter } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/home',
    children: [
      {
        path: 'home',
        name: 'home',
        component: () => import('@/pages/home.vue'),
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@/pages/settings.vue'),
      },
      {
        path: 'receive',
        name: 'receive',
        component: () => import('@/pages/Receive.vue'),
      },
      {
        path: 'send',
        name: 'send',
        component: () => import('@/pages/SendFile.vue'),
      },
    ],
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

export default router
