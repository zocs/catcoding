import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/board',
    },
    {
      path: '/board',
      name: 'Board',
      component: () => import('../views/Board.vue'),
    },
    {
      path: '/gantt',
      name: 'Gantt',
      component: () => import('../views/Gantt.vue'),
    },
    {
      path: '/agents',
      name: 'Agents',
      component: () => import('../views/Agents.vue'),
    },
    {
      path: '/logs',
      name: 'Logs',
      component: () => import('../views/Logs.vue'),
    },
    {
      path: '/command',
      name: 'Command',
      component: () => import('../views/Command.vue'),
    },
  ],
})

export default router
