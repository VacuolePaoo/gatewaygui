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
      {
        path: 'select-device',
        name: 'select-device',
        component: () => import('@/pages/SelectDevice.vue'),
      },
      {
        path: 'mount',
        name: 'mount',
        component: () => import('@/pages/MountDirectory.vue'),
      },
    ],
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

export default router
