import { createRouter, createWebHashHistory } from 'vue-router';
import { guard } from './guard';
import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'LoginPage',
    component: () => import('@renderer/views/Login/index.vue')
  },
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

router.beforeEach((to, from, next) => guard(to, from, next));

export { router };
