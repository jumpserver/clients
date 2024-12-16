import { createRouter, createWebHashHistory } from 'vue-router';
import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'homePage',
    component: () => import('../layouts/index.vue'),
    children: [
      {
        path: 'linux',
        name: 'Linux',
        component: () => import('@renderer/views/Linux/index.vue')
      },
      {
        path: 'windows',
        name: 'Windows',
        component: () => import('@renderer/views/Windows/index.vue')
      },
      {
        path: 'database',
        name: 'Database',
        component: () => import('@renderer/views/Database/index.vue')
      },
      {
        path: 'history',
        name: 'History',
        component: () => import('@renderer/views/History/index.vue')
      },
      {
        path: 'favorite',
        name: 'Favorite',
        component: () => import('@renderer/views/Favorite/index.vue')
      },
      {
        path: 'setting',
        name: 'Setting',
        component: () => import('@renderer/views/Setting/index.vue')
      }
    ]
  },
  {
    path: '/:pathMatch(.*)*',
    name: '404',
    component: () => import('@renderer/views/NotFound/index.vue')
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes
});

export { router };
