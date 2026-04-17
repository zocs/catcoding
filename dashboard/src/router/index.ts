import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/dashboard'
    },
    {
      path: '/dashboard',
      component: () => import('../views/DashboardLayout.vue'),
      children: [
        {
          path: '',
          name: 'dashboard',
          redirect: { name: 'agents' }
        },
        {
          path: 'agents',
          name: 'agents',
          component: () => import('../views/Agents.vue')
        },
        {
          path: 'gantt',
          name: 'gantt',
          component: () => import('../views/Gantt.vue')
        },
        {
          path: 'terminal',
          name: 'terminal',
          component: () => import('../views/TerminalDemo.vue')
        },
        {
          path: 'commands',
          name: 'commands',
          component: () => import('../views/Command.vue')
        },
        {
          path: 'logs',
          name: 'logs',
          component: () => import('../views/Logs.vue')
        },
        {
          path: 'board',
          name: 'board',
          component: () => import('../views/Board.vue')
        },
        {
          path: 'memory',
          name: 'memory',
          component: () => import('../views/Memory.vue')
        }
      ]
    }
  ]
})

export default router
