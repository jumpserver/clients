import { createRouter, createWebHashHistory } from 'vue-router'

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'mainPage',
      redirect: '/ssh',
      component: () => import('@/layouts/Main.vue'),
      children: [
        {
          path: '/ssh',
          name: 'sshPage',
          component: () => import('@/pages/Terminal.vue')
        },
        {
          path: '/remote',
          name: 'remotePage',
          component: () => import('@/pages/RemoteDesktop.vue')
        },
        {
          path: '/files',
          name: 'fileTransferPage',
          component: () => import('@/pages/FileTransfer.vue')
        },
        {
          path: '/databases',
          name: 'databasesPage',
          component: () => import('@/pages/Databases.vue')
        },
        {
          path: '/about',
          name: 'aboutPage',
          component: () => import('@/pages/About.vue')
        }
      ]
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ]
})
